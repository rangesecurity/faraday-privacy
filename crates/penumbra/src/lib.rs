use {
    anyhow::{Context, Result},
    camino::Utf8PathBuf,
    penumbra_sdk_keys::FullViewingKey,
    penumbra_sdk_proto::{
        box_grpc_svc::{self, BoxGrpcService},
        view::v1::{
            view_service_client::ViewServiceClient, view_service_server::ViewServiceServer,
        },
    },
    penumbra_sdk_view::{ViewClient, ViewServer},
    std::str::FromStr,
    tonic::transport::Channel,
    futures::StreamExt
};

pub struct DisclosureClient {
    view: ViewServiceClient<BoxGrpcService>,
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
        Ok(Self { view: view_service })
    }
    pub async fn sync(&mut self) -> Result<()> {
        let mut stream = self.view().status_stream().await?;
        while let Some(Ok(res)) = stream.next().await {}
        Ok(())
    }
    pub fn view(&mut self) -> &mut impl ViewClient {
        &mut self.view
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
    use penumbra_sdk_proto::{
        core::txhash::v1::TransactionId,
        util::tendermint_proxy::v1::{
            tendermint_proxy_service_client::TendermintProxyServiceClient, AbciQueryRequest,
            GetBlockByHeightRequest, GetStatusRequest, GetTxRequest,
        },
        view::v1::{
            BalancesRequest, IndexByAddressRequest, StatusRequest, TransactionInfoByHashRequest,
            TransparentAddressRequest,
        },
    };

    use super::*;
    #[tokio::test]
    async fn test_disclosure_client_new() {
        let fvk = FullViewingKey::from_str("penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq").unwrap();
        let idx = fvk.payment_address(AddressIndex::new(0));

        let mut dc = DisclosureClient::new("mydb", "http://localhost:8080/", &fvk)
            .await
            .unwrap();

        dc.sync().await.unwrap();

        let vc = dc.view();

        let res = vc.balances(AddressIndex::new(0), None).await.unwrap();
        println!("{res:#?}");

        let res = vc
            .transaction_info_by_hash(
                "74f2f7e67137ccbbe14636180acc22e1d1ce2987dcc0ec6f9c1e8a88b0738b88"
                  //  .to_uppercase()
                    .parse()
                    .unwrap(),
            )
            .await
            .unwrap();
        println!("{res:#?}");
    }
}
