use solana_sdk::{signature::Keypair, signer::{EncodableKey, Signer}};
use solana_zk_sdk::encryption::elgamal::ElGamalKeypair;

pub mod utils;

#[tokio::test]
async fn test_confidential_transfer_basic() {
    common::utils::init_log("trace", "");
    let airdrop_amount = spl_token_2022::ui_amount_to_amount(1000.0, 9);
    let rpc = solana_client::nonblocking::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());


    let sender = Keypair::new();
    let recipient = Keypair::new();
    let mint_authority = Keypair::new();
    let mint = Keypair::new();
    let auditor_keypair = ElGamalKeypair::new_rand();

    let mut tc = utils::TestClient::new();
    tc.airdrop(
        &[
            (sender.pubkey(), airdrop_amount),
            (recipient.pubkey(), airdrop_amount),
            (mint_authority.pubkey(), airdrop_amount),
        ]
    );
    log::info!("{:#?}", tc.svm().get_account(&"ZkE1Gama1Proof11111111111111111111111111111".parse().unwrap()).unwrap());
    tc.create_no_auditor_mint(None, &mint_authority, &mint, &auditor_keypair).await.unwrap();
    tc.create_token_accounts(None, &sender, &mint.pubkey()).await;
}
#[tokio::test]
async fn test_confidential_transfer_basic_devnet() {
    common::utils::init_log("trace", "");
    let airdrop_amount = spl_token_2022::ui_amount_to_amount(1000.0, 9);
    let rpc = solana_client::nonblocking::rpc_client::RpcClient::new("https://api.devnet.solana.com".to_string());


    let sender = Keypair::read_from_file("sender.json").unwrap();
    //let recipient = Keypair::new();
    //let mint_authority = Keypair::new();
    let mint = Keypair::new();
    let auditor_keypair = ElGamalKeypair::new_rand();

    let mut tc = utils::TestClient::new();
    tc.create_no_auditor_mint(Some(&rpc), &sender, &mint, &auditor_keypair).await.unwrap();
    tc.create_token_accounts(Some(&rpc), &sender, &mint.pubkey()).await;
}