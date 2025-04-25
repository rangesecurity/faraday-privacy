use {
    anyhow::{Context, Result},
    camino::Utf8PathBuf,
    common::{
        self,
        models::{counterparty::Role, transaction::Protocol, Counterparty, Transaction},
    },
    futures::StreamExt,
    penumbra_sdk_keys::{keys::AddressIndex, Address, AddressView, FullViewingKey},
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
    std::str::FromStr,
    tonic::transport::Channel,
};

pub struct DisclosureClient {
    view: ViewServiceClient<BoxGrpcService>,
    tpc: TendermintProxyServiceClient<Channel>,
    // probably need a better way to do this, can we just store the FullViewingKey ?
    payment_keys: Vec<Address>,
}

impl DisclosureClient {
    pub async fn new(storage_path: &str, url: &str, fvk: &FullViewingKey) -> Result<Self> {
        let storage_path = Utf8PathBuf::from_str(storage_path)?;
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
        Ok(Self {
            view: view_service,
            tpc: TendermintProxyServiceClient::connect(url.to_string())
                .await
                .with_context(|| "failed to connect to proxy")?,
            payment_keys: (0..=3)
                .into_iter()
                .map(|idx| fvk.payment_address(AddressIndex::new(idx)).0)
                .collect::<Vec<_>>(),
        })
    }
    pub async fn sync(&mut self) -> Result<()> {
        let mut stream = self.view().status_stream().await?;
        while let Some(Ok(_)) = stream.next().await {}
        Ok(())
    }
    pub fn view(&mut self) -> &mut impl ViewClient {
        &mut self.view
    }
    pub async fn transaction(&mut self, hash: &str) -> Result<Transaction> {
        let txn = self
            .view()
            .transaction_info_by_hash(hash.parse().with_context(|| "failed to parse hash")?)
            .await?;

        let time = self
            .tpc
            .get_block_by_height(GetBlockByHeightRequest {
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
            .with_context(|| "time is None")?;

        let mut tx = Transaction {
            transaction_hash: hash.to_string(),
            protocol: Protocol::Penumbra,
            chain_id: txn.view.body_view.transaction_parameters.chain_id,
            counterparties: vec![],
            // todo: convert timestamp
            timestamp: format!("{}", time.seconds),
            metadata: None,
        };

        for effect in txn.summary.effects {
            let role = if self.payment_keys.contains(&effect.address.address()) {
                Role::Sender
            } else {
                Role::Receiver
            };
            tx.counterparties.push(Counterparty {
                // this is not correct, need a better way to determine if it is the recipient or receiver
                role,
                address: effect.address.address().to_string(),
                name: None,
                assets: vec![],
            })
        }
        Ok(tx)
    }
}

impl AsMut<ViewServiceClient<BoxGrpcService>> for DisclosureClient {
    fn as_mut(&mut self) -> &mut ViewServiceClient<BoxGrpcService> {
        &mut self.view
    }
}

#[cfg(test)]
mod test {
    use penumbra_sdk_keys::{keys::AddressIndex, Address};

    use super::*;
    #[tokio::test]
    async fn test_disclosure_client_new() {
        let fvk = FullViewingKey::from_str("penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq").unwrap();
        let idx = fvk.payment_address(AddressIndex::new(0));

        let mut dc = DisclosureClient::new("mydb", "http://localhost:8080/", &fvk)
            .await
            .unwrap();

        dc.sync().await.unwrap();

        let tx_info = dc
            .transaction("c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf")
            .await
            .unwrap();
        println!("{tx_info:#?}");
        return;

        let vc = dc.view();

        let res = vc.balances(AddressIndex::new(0), None).await.unwrap();

        let res = vc
            .transaction_info_by_hash(
                "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf"
                    //  .to_uppercase()
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{:#?}", res.summary);
        // print memo view (contains address information)
        // println!("{:#?}", res.view.body_view.memo_view);

        // print transaction parameters (contains chain information)
        // println!("{:#?}", res.view.body_view.transaction_parameters);

        // print action views (contains tx summaries, etc...)
        // println!("{:#?}", res.view.body_view.action_views);
    }
}
