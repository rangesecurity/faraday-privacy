use {
    crate::{api::server::router::AppState, client::DisclosureClient},
    axum::{extract::State, response::IntoResponse, Json},
    common::{
        apis::default_api::DiscloseSingleTransactionError,
        models::{
            error::Error as CommonError, DisclosedTransactionResult,
            DisclosedTransactionResultDisclosureTransactions, DisclosureRequestSingle,
        },
    },
    penumbra_sdk_keys::FullViewingKey,
    reqwest::StatusCode,
    std::{str::FromStr, sync::Arc},
};

pub async fn disclose_transaction(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DisclosureRequestSingle>,
) -> impl IntoResponse {
    let state = state.clone();
    let fvk = match FullViewingKey::from_str(&payload.full_viewing_key) {
        Ok(fvk) => fvk,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DiscloseSingleTransactionError::Status400(CommonError {
                    code: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("{err:#?}"),
                })),
            )
                .into_response()
        }
    };
    let dc = match DisclosureClient::new(&state.url, &fvk).await {
        Ok(dc) => dc,
        Err(err) => {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DiscloseSingleTransactionError::Status500(CommonError {
                    code: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                    message: format!("failed to initialize disclosure client {err:#?}"),
                })),
            )
                .into_response()
        }
    };

    {
        let dc = dc.lock().await;

        if let Err(err) = dc.sync().await {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(DiscloseSingleTransactionError::Status500(CommonError {
                    code: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                    message: format!("failed to synchronize disclosure client {err:#?}"),
                })),
            )
                .into_response();
        }
    }

    let txn = {
        let dc = dc.lock().await;
        dc.transaction(&payload.transaction_hash).await
    };
    match txn {
        Ok(tx_info) => (
            StatusCode::OK,
            Json(DisclosedTransactionResult {
                disclosure_transactions: Some(DisclosedTransactionResultDisclosureTransactions {
                    transactions: vec![tx_info],
                }),
                disclosure_errors: None,
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(DiscloseSingleTransactionError::Status500(CommonError {
                code: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                message: format!("failed to generate disclosure bundle {err:#?}"),
            })),
        )
            .into_response(),
    }
}
