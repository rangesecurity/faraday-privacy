use {
    anyhow::{Context, Result},
    common::{
        apis::{
            configuration::Configuration,
            default_api::{disclose_multiple_transactions, disclose_single_transaction},
        },
        models::{
            DisclosedTransactionResult, DisclosureRequestMultiple, DisclosureRequestSingle,
            Transaction,
        },
    },
    std::sync::Arc,
};

#[derive(Clone)]
pub struct ApiClient(Configuration);

impl ApiClient {
    pub fn new(cfg: Configuration) -> Arc<Self> {
        Arc::new(Self(cfg))
    }
    pub async fn disclose_transaction(&self, tx_hash: String, fvk: String) -> Result<Transaction> {
        Ok(disclose_single_transaction(
            &self.0,
            DisclosureRequestSingle {
                full_viewing_key: fvk,
                transaction_hash: tx_hash,
            },
        )
        .await
        .with_context(|| "failed to send request")?)
    }
    pub async fn disclose_transactions(
        &self,
        tx_hashes: Vec<String>,
        fvk: String,
    ) -> Result<Vec<DisclosedTransactionResult>> {
        Ok(disclose_multiple_transactions(
            &self.0,
            DisclosureRequestMultiple {
                full_viewing_key: fvk,
                transaction_hashes: tx_hashes,
            },
        )
        .await
        .with_context(|| "failed to send request")?)
    }
}
