use {
    crate::types::TransactionType,
    anyhow::{Context, Result},
    camino::Utf8PathBuf,
    common::{
        self,
        models::{counterparty::Role, transaction::Protocol, Counterparty, Transaction},
    },
    futures::StreamExt,
    penumbra_sdk_keys::{AddressView, FullViewingKey},
    penumbra_sdk_proto::{
        box_grpc_svc::{self, BoxGrpcService},
        util::tendermint_proxy::v1::{
            tendermint_proxy_service_client::TendermintProxyServiceClient, GetBlockByHeightRequest,
        },
        view::v1::{
            view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
        },
    },
    penumbra_sdk_view::{ViewClient, ViewServer},
    sha3::{Digest, Sha3_256},
    std::{collections::HashMap, str::FromStr, sync::Arc},
    tokio::sync::Mutex,
    tonic::transport::Channel,
};

#[derive(Clone)]
pub struct DisclosureClient {
    view: Arc<Mutex<ViewServiceClient<BoxGrpcService>>>,
    tpc: Arc<Mutex<TendermintProxyServiceClient<Channel>>>,
    fvk: FullViewingKey,
}

impl DisclosureClient {
    /// We need to wrap in Arc<RwLock<T>> because ViewServiceClient is not Sync
    pub async fn new(url: &str, fvk: &FullViewingKey) -> Result<Arc<Mutex<Self>>> {
        // store the db on disk using the filename as the sha3 hash of the fvk
        // this way we arent storing the actual fvk on disk in plaintext
        // this also allows the DisclosureClient to be reused within the api service
        // and not have conflicts

        let mut hasher = Sha3_256::new();
        hasher.update(fvk.to_string());
        let storage_path = Utf8PathBuf::from_str(&hex::encode(hasher.finalize()))?;
        let registry_path = storage_path.join("registry.json");
        let registry_path = if registry_path.exists() {
            Some(registry_path)
        } else {
            None
        };
        let view_server =
            ViewServer::load_or_initialize(Some(storage_path), registry_path, fvk, url.parse()?)
                .await
                .with_context(|| "failed to create view server")?;

        let svc: ViewServiceServer<ViewServer> = ViewServiceServer::new(view_server);
        let view_service = ViewServiceClient::new(box_grpc_svc::local(svc));

        Ok(Arc::new(Mutex::new(Self {
            view: Arc::new(Mutex::new(view_service)),
            tpc: Arc::new(Mutex::new(
                TendermintProxyServiceClient::connect(url.to_string())
                    .await
                    .with_context(|| "failed to connect to proxy")?,
            )),
            fvk: fvk.clone(),
        })))
    }

    pub async fn sync(&self) -> Result<()> {
        let view = self.view.clone();
        let mut view = view.lock().await;
        let view: &mut dyn ViewClient = &mut *view;
        let mut stream = view.status_stream().await?;
        while let Some(Ok(_)) = stream.next().await {}
        Ok(())
    }

    pub async fn transaction(&self, hash: &str) -> Result<Transaction> {
        let txn = {
            let view = self.view.clone();
            let mut view = view.lock().await;
            let view: &mut dyn ViewClient = &mut *view;
            view.transaction_info_by_hash(hash.parse().with_context(|| "failed to parse hash")?)
                .await
                .with_context(|| "failed to get tx hash")?
        };
        let time = {
            let tpc = self.tpc.clone();
            let mut tpc = tpc.lock().await;
            tpc.get_block_by_height(GetBlockByHeightRequest {
                height: txn.height as i64,
            })
            .await
            .with_context(|| "failed to query blockheight")?
            .into_inner()
            .block
            .with_context(|| "block is None")?
            .header
            .with_context(|| "header is None")?
            .time
            .with_context(|| "time is None")?
        };
        let mut assets: HashMap<&penumbra_sdk_asset::asset::Id, common::models::Asset> =
            Default::default();
        for (asset_id, denom_metadata) in txn.perspective.denoms.iter() {
            assets.insert(
                asset_id,
                common::models::Asset {
                    identifier: denom_metadata.base_denom().denom.clone(),
                    // we're just using the assets map to aggregate metadata information
                    // of all denoms in the transaction, so we dont need to store teh amount
                    amount: "".to_string(),
                    decimals: Some(denom_metadata.default_unit().exponent() as u32),
                },
            );
        }

        // we want additional metadata to describe the effects of the transaction so
        // we can skip including the various *Output* actions
        // a transfer from A->B would have two actions Spend and Output
        // the Output metadata is not relevant for disclosure, as we can
        // simply disclose that this transaction includes a spend
        let metadata = txn
            .view
            .body_view
            .action_views
            .iter()
            .filter_map(|action| {
                let transaction_type = TransactionType::from(action);
                let transaction_type_str = AsRef::<String>::as_ref(&transaction_type);

                if transaction_type_str.eq("Output")
                    || transaction_type_str.eq("CommunityPoolOutput")
                {
                    return None;
                }

                Some(common::models::Metadata {
                    transaction_type: Some(transaction_type.to_string()),
                    tags: None,
                    notes: None,
                })
            })
            .collect::<Vec<_>>();

        let mut tx = Transaction {
            transaction_hash: hash.to_string(),
            protocol: Protocol::Penumbra,
            chain_id: txn.view.body_view.transaction_parameters.chain_id,
            counterparties: vec![],
            // todo: should we foramt the timestamp into a human readable value?
            timestamp: format!("{}", time.seconds),
            metadata: if metadata.is_empty() {
                None
            } else {
                Some(metadata)
            },
        };

        for effect in txn.summary.effects {
            let role = if let AddressView::Decoded { .. } =
                self.fvk.view_address(effect.address.address())
            {
                Role::Sender
            } else {
                Role::Receiver
            };

            let balance_info = match role {
                Role::Receiver => effect
                    .balance
                    .required()
                    .next()
                    .with_context(|| "failed to get receiver balance info")?,
                Role::Sender => effect
                    .balance
                    .provided()
                    .next()
                    .with_context(|| "failed to get sender balance info")?,
            };

            let denom_metadata = assets
                .get(&balance_info.asset_id)
                .with_context(|| format!("failed to get metadata for {}", balance_info.asset_id))?;

            tx.counterparties.push(Counterparty {
                // this is not correct, need a better way to determine if it is the recipient or receiver
                role,
                address: effect.address.address().to_string(),
                name: None,
                assets: vec![common::models::Asset {
                    identifier: denom_metadata.identifier.clone(),
                    amount: balance_info.amount.to_string(),
                    decimals: denom_metadata.decimals,
                }],
            })
        }

        Ok(tx)
    }
}

#[cfg(test)]
mod test {
    use common::models::Asset;

    use super::*;
    #[tokio::test]
    async fn test_disclosure_client_new() {
        let fvk = FullViewingKey::from_str("penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq").unwrap();

        let dc = DisclosureClient::new("http://localhost:8080/", &fvk)
            .await
            .unwrap();
        let dc = dc.lock().await;
        dc.sync().await.unwrap();

        let tx_info = dc
            .transaction("c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf")
            .await
            .unwrap();
        println!("{tx_info:#?}");
        assert_eq!(
            tx_info.transaction_hash,
            "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf"
        );
        assert_eq!(tx_info.protocol, Protocol::Penumbra);
        assert_eq!(tx_info.chain_id, "penumbra-testnet-phobos-x3b26d34a");
        assert!(tx_info.counterparties.contains(&Counterparty {
            role: Role::Receiver,
            address: "penumbra147mfall0zr6am5r45qkwht7xqqrdsp50czde7empv7yq2nk3z8yyfh9k9520ddgswkmzar22vhz9dwtuem7uxw0qytfpv7lk3q9dp8ccaw2fn5c838rfackazmgf3ahh09cxmz".to_string(),
            name: None,
            assets: vec![Asset {
                identifier: "wtest_usd".to_string(),
                amount: "100000000000000000000".to_string(),
                decimals: Some(18)
            }]
        }));
        assert!(tx_info.counterparties.contains(&Counterparty {
            role: Role::Sender,
            address: "penumbra1alp9a75s438d33rs5nt245ue2wctfne7x4c3v7afyslmwefltgpzm7r0jgmxphrcva6h44v9pe3esstnkw5fsha54rcp7xpmaphxx76scql92mefzg366ckwcy425s3y5657ll".to_string(),
            name: None,
            assets: vec![Asset {
                identifier: "wtest_usd".to_string(),
                amount: "100000000000000000000".to_string(),
                decimals: Some(18)
            }]
        }));
        println!("{}", serde_json::to_string(&tx_info).unwrap());
    }
}
