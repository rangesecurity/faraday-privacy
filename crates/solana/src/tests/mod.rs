use solana_sdk::{signature::Keypair, signer::Signer};
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

    let mut tc = utils::TestClient::new();
    tc.svm().get_account(&"NativeLoader1111111111111111111111111111111".parse().unwrap()).unwrap();
    tc.airdrop(
        &[
            (sender.pubkey(), airdrop_amount),
            (recipient.pubkey(), airdrop_amount),
            (mint_authority.pubkey(), airdrop_amount),
        ]
    );
    log::info!("{:#?}", tc.svm().get_account(&"ZkE1Gama1Proof11111111111111111111111111111".parse().unwrap()).unwrap());
    tc.create_no_auditor_mint(&mint_authority, &mint, &auditor_keypair).unwrap();
    tc.create_token_accounts(&sender, &mint.pubkey());
}