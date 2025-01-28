use std::sync::Arc;

use solana_sdk::{signature::Keypair, signer::{EncodableKey, Signer}};
use solana_zk_sdk::encryption::elgamal::ElGamalKeypair;

pub mod utils;

#[tokio::test]
async fn test_confidential_transfer_basic() {
    common::utils::init_log("trace", "");
    let airdrop_amount = spl_token_2022::ui_amount_to_amount(1000.0, 9);

    let sender = Keypair::new();
    let recipient = Keypair::new();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();
    let auditor_keypair = ElGamalKeypair::new_rand();

    let mut tc = utils::TestClient::new(None);
    tc.airdrop(
        &[
            (sender.pubkey(), airdrop_amount),
            (recipient.pubkey(), airdrop_amount),
            (mint_authority.pubkey(), airdrop_amount),
        ]
    ).await;
    log::info!("{:#?}", tc.svm().get_account(&"ZkE1Gama1Proof11111111111111111111111111111".parse().unwrap()).unwrap());
    tc.create_no_auditor_mint(&mint_authority, &mint, &auditor_keypair).await.unwrap();
    tc.create_token_accounts(&sender, &mint.pubkey()).await;
}
#[tokio::test]
async fn test_confidential_transfer_basic_devnet() {
    common::utils::init_log("info", "");
    let airdrop_amount = spl_token_2022::ui_amount_to_amount(1000.0, 9);
    let rpc = solana_client::nonblocking::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());
    //let rpc = solana_client::nonblocking::rpc_client::RpcClient::new("http://localhost:8899".to_string());
    let mut tc = utils::TestClient::new(Some(Arc::new(rpc)));


    let sender = Keypair::read_from_file("sender.json").unwrap();
    let recipient = Keypair::new();
    let mint_authority = Keypair::read_from_file("mint_authority.json").unwrap();
    let mint = Keypair::new();
    let auditor_keypair = ElGamalKeypair::new_rand();
    //tc.airdrop(
    //    &[
    //        (sender.pubkey(), airdrop_amount),
    //        (recipient.pubkey(), airdrop_amount),
    //        (mint_authority.pubkey(), airdrop_amount),
    //    ]
    //).await;

    tc.create_no_auditor_mint(&sender, &mint, &auditor_keypair).await.unwrap();
    tc.create_token_accounts(&sender, &mint.pubkey()).await;
    tc.mint_confidential_tokens(&mint_authority, mint.pubkey(), sender.pubkey(), 100_000, &auditor_keypair).await;
}