/*
 * Privacy Transaction Disclosure API
 *
 * API specification for selective disclosure of privacy-preserving transactions across various protocols including Penumbra and Solana Confidential Transactions. 
 *
 * The version of the OpenAPI document: 1.0.0
 * Contact: support@example.com
 * Generated by: https://openapi-generator.tech
 */

use crate::models;
use serde::{Deserialize, Serialize};

#[derive(Clone, Default, Debug, PartialEq, Serialize, Deserialize)]
pub struct DisclosureRequestMultiple {
    /// Full viewing key used to disclose transactions
    #[serde(rename = "fullViewingKey")]
    pub full_viewing_key: String,
    #[serde(rename = "transactionHashes")]
    pub transaction_hashes: Vec<String>,
}

impl DisclosureRequestMultiple {
    pub fn new(full_viewing_key: String, transaction_hashes: Vec<String>) -> DisclosureRequestMultiple {
        DisclosureRequestMultiple {
            full_viewing_key,
            transaction_hashes,
        }
    }
}

