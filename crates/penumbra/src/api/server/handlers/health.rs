use axum::{response::IntoResponse, Json};
use chrono::Utc;
use serde_json::json;

pub async fn health() -> impl IntoResponse {
    Json(json!({
        "status": "ok",
        "timestamp": Utc::now(),
        "version": env!("CARGO_PKG_VERSION")
    }))
}
