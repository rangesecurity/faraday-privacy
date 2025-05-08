use {
    crate::{api::server::router::AppState, client::DisclosureClient},
    axum::{extract::State, response::IntoResponse, Json},
    common::{
        apis::default_api::DiscloseMultipleTransactionsError,
        models::{
            disclosure_error::ResultType, error::Error as CommonError, DisclosedTransactionResult,
            DisclosedTransactionResultDisclosureErrors,
            DisclosedTransactionResultDisclosureTransactions, DisclosureError,
            DisclosureRequestMultiple, Transaction,
        },
    },
    penumbra_sdk_keys::FullViewingKey,
    reqwest::StatusCode,
    std::{str::FromStr, sync::Arc},
};

pub async fn disclose_transactions(
    State(state): State<Arc<AppState>>,
    Json(payload): Json<DisclosureRequestMultiple>,
) -> impl IntoResponse {
    let state = state.clone();
    let fvk = match FullViewingKey::from_str(&payload.full_viewing_key) {
        Ok(fvk) => fvk,
        Err(err) => {
            return (
                StatusCode::BAD_REQUEST,
                Json(DiscloseMultipleTransactionsError::Status400(CommonError {
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
                Json(DiscloseMultipleTransactionsError::Status500(CommonError {
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
                Json(DiscloseMultipleTransactionsError::Status500(CommonError {
                    code: StatusCode::INTERNAL_SERVER_ERROR.to_string(),
                    message: format!("failed to synchronize disclosure client {err:#?}"),
                })),
            )
                .into_response();
        }
    }

    let mut disclosed_transactions: Vec<Transaction> =
        Vec::with_capacity(payload.transaction_hashes.len());
    let mut disclosure_errors: Vec<DisclosureError> =
        Vec::with_capacity(payload.transaction_hashes.len());

    for tx_hash in payload.transaction_hashes {
        let dc = dc.lock().await;
        match dc.transaction(&tx_hash).await {
            Ok(tx_info) => disclosed_transactions.push(tx_info),
            Err(err) => disclosure_errors.push(DisclosureError {
                result_type: ResultType::Error,
                transaction_hash: tx_hash,
                error: CommonError {
                    code: StatusCode::BAD_REQUEST.to_string(),
                    message: format!("{err:#?}"),
                },
                error_types: vec![],
            }),
        }
    }
    let disclosure_errors = if disclosure_errors.is_empty() {
        None
    } else {
        Some(DisclosedTransactionResultDisclosureErrors {
            errors: disclosure_errors,
        })
    };
    let disclosure_transactions = if disclosed_transactions.is_empty() {
        None
    } else {
        Some(DisclosedTransactionResultDisclosureTransactions {
            transactions: disclosed_transactions,
        })
    };

    (
        StatusCode::OK,
        Json(DisclosedTransactionResult {
            disclosure_errors,
            disclosure_transactions,
        }),
    )
        .into_response()
}
