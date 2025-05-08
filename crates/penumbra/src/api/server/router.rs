use {
    crate::api::server::handlers, axum::{routing::post, Router}, std::sync::Arc, tower_http::cors::CorsLayer
};

#[derive(Clone)]
pub struct AppState {
    pub url: String,
}

pub fn new(url: String) -> Router {
    Router::new()
    .route("/disclosure/transaction", post(handlers::disclose_transaction))
    .with_state(Arc::new(AppState {
        url,
    }))
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