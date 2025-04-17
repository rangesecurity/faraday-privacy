use {
    anyhow::{Context, Result},
    penumbra_sdk_proto::{
        box_grpc_svc::{self, BoxGrpcService},
        view::v1::view_service_client::ViewServiceClient,
    },
    penumbra_sdk_view::ViewClient,
    tonic::transport::Channel,
};

pub struct DisclosureClient {
    view: ViewServiceClient<Channel>,
}

impl DisclosureClient {
    pub async fn new(url: &str) -> Result<Self> {
        let view_service = ViewServiceClient::connect(url.to_string())
            .await
            .with_context(|| "failed to connect ViewServiceClient")?;
        Ok(Self { view: view_service })
    }
}

impl AsMut<ViewServiceClient<Channel>> for DisclosureClient {
    fn as_mut(&mut self) -> &mut ViewServiceClient<Channel> {
        &mut self.view
    }
}

#[cfg(test)]
mod test {
    use penumbra_sdk_proto::view::v1::StatusRequest;

    use super::*;
    #[tokio::test]
    async fn test_disclosure_client_new() {
        let mut dc = DisclosureClient::new("http://localhost:8080")
            .await
            .unwrap();
        let view_service: &mut ViewServiceClient<Channel> = dc.as_mut();
        let res = view_service.status(StatusRequest {}).await.unwrap();
        println!("{res:#?}");
    }
}
