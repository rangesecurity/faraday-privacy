use {
    crate::api::server::handlers,
    axum::{
        routing::{get, post},
        Router,
    },
    std::sync::Arc,
    tower_http::cors::CorsLayer,
};

#[derive(Clone)]
pub struct AppState {
    pub url: String,
}

pub fn new(url: String) -> Router {
    Router::new()
        .route(
            "/disclose/transaction",
            post(handlers::disclose_transaction),
        )
        .route(
            "/disclose/transactions",
            post(handlers::disclose_transactions),
        )
        .route("health", get(handlers::health))
        .with_state(Arc::new(AppState { url }))
        .layer(
            CorsLayer::new()
                .allow_methods(tower_http::cors::Any)
                .allow_origin(tower_http::cors::Any)
                .allow_headers([
                    http::header::CONTENT_TYPE,
                    http::header::UPGRADE,
                    http::header::CONNECTION,
                    http::header::SEC_WEBSOCKET_KEY,
                    http::header::SEC_WEBSOCKET_VERSION,
                    http::header::SEC_WEBSOCKET_PROTOCOL,
                ]),
        )
}

#[cfg(test)]
mod test {
    use {
        super::*,
        axum::{body::Body, http::Request},
        common::models::{
            counterparty::Role, transaction::Protocol, Asset, Counterparty,
            DisclosedTransactionResult, DisclosureRequestMultiple, DisclosureRequestSingle,
        },
        http::StatusCode,
        http_body_util::BodyExt,
        serde_json::Value,
        tower::{Service, ServiceExt},
    };

    #[tokio::test]
    async fn test_disclose_transaction() {
        let mut router = new("http://localhost:8080/".to_string());

        let request = Request::builder().method("POST").uri("/disclose/transaction").header("Content-Type", "application/json").body(Body::from(serde_json::to_string(&DisclosureRequestSingle {
            full_viewing_key: "penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq".to_string(),
            transaction_hash: "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf".to_string()
        }).unwrap())).unwrap();
        let res = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        let status = res.status();

        let res: DisclosedTransactionResult =
            serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
        assert!(res.disclosure_errors.is_none());

        let Some(txns) = res.disclosure_transactions else {
            panic!("disclosure_transactions should not be None");
        };
        let tx_info = &txns.transactions[0];
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

        assert_eq!(status, StatusCode::OK);
    }

    #[tokio::test]
    async fn test_disclose_transactions() {
        let mut router = new("http://localhost:8080/".to_string());

        let request = Request::builder().method("POST").uri("/disclose/transactions").header("Content-Type", "application/json").body(Body::from(serde_json::to_string(&DisclosureRequestMultiple {
            full_viewing_key: "penumbrafullviewingkey1jzwnl8k7hhqnvf06m4hfdwtsyc9ucce4nq6slpvxm8l9jgse0gg676654ea865dz4mn9ez33q3ysnedcplxey5g589cx4xl0duqkzrc0gqscq".to_string(),
            transaction_hashes: vec!["c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b25177d5d9d300ccaf".to_string(), "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b2517fffffffffccff".to_string()]
        }).unwrap())).unwrap();
        let res = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        let status = res.status();

        let res: DisclosedTransactionResult =
            serde_json::from_slice(&res.into_body().collect().await.unwrap().to_bytes()).unwrap();
        let Some(disclosure_errors) = res.disclosure_errors else {
            panic!("there should be at least one disclosure error");
        };
        assert_eq!(disclosure_errors.errors.len(), 1);
        assert_eq!(
            disclosure_errors.errors[0].transaction_hash,
            "c888fe430188c9a83aa450ab7f647c51f6224caf16e3b8b2517fffffffffccff"
        );

        let Some(txns) = res.disclosure_transactions else {
            panic!("disclosure_transactions should not be None");
        };
        let tx_info = &txns.transactions[0];
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

        assert_eq!(status, StatusCode::OK);
    }
    #[tokio::test]
    async fn test_health() {
        let mut router = new("http://localhost:8080/".to_string());
        let request = Request::builder()
            .method("GET")
            .uri("/health")
            .body(Body::from(""))
            .unwrap();
        let res = ServiceExt::<Request<Body>>::ready(&mut router)
            .await
            .unwrap()
            .call(request)
            .await
            .unwrap();
        let status = res.status();
        assert_eq!(status, StatusCode::OK);

        let body_bytes = res.into_body().collect().await.unwrap().to_bytes();
        let json: Value = serde_json::from_slice(&body_bytes).unwrap();

        assert_eq!(json["status"], "ok");
        assert_eq!(json["version"], env!("CARGO_PKG_VERSION"));
        assert!(json["timestamp"].is_string());
    }
}
