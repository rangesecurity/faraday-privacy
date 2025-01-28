use {
    anyhow::Result, litesvm::LiteSVM, solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcRequestAirdropConfig}, solana_feature_set::FeatureSet, solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey::Pubkey, signature::Keypair, signer::Signer, system_program, transaction::Transaction}, solana_zk_sdk::{encryption::{auth_encryption::AeKey, elgamal::ElGamalKeypair, pod::elgamal::{PodElGamalCiphertext, PodElGamalPubkey}}, zk_elgamal_proof_program::proof_data::PubkeyValidityProofData}, spl_token_2022::{extension::{confidential_transfer::instruction::configure_account, ExtensionType}, state::Mint}, spl_token_client::token::ExtensionInitializationParams, spl_token_confidential_transfer_proof_extraction::instruction::{ProofData, ProofLocation}, std::sync::Arc
};
pub struct TestClient {
    svm: LiteSVM,
    rpc: Option<Arc<RpcClient>>,
}

impl TestClient {
    pub fn new(
        rpc: Option<Arc<RpcClient>>,
    ) -> Self {
        let mut svm = LiteSVM::new()
        // need to enable build ints
        .with_builtins(Some(
            FeatureSet::all_enabled()
        ))
        // need to enable precompiles
        .with_precompiles(Some(
            FeatureSet::all_enabled()
        ))
        .with_log_bytes_limit(Some(1_000_000_000));
        svm.add_program_from_file(
            "TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb".parse().unwrap(),
            "../../token2022.so"
        ).unwrap();

        svm.set_account(
            "ZkE1Gama1Proof11111111111111111111111111111".parse().unwrap(),
            Account {
                lamports: 1,
                data: base64::decode("emtfZWxnYW1hbF9wcm9vZl9wcm9ncmFt").unwrap(),
                owner: "NativeLoader1111111111111111111111111111111".parse().unwrap(),
                executable: true,
                rent_epoch: 18446744073709551615
            }
        ).unwrap();
        Self {
            svm: svm,
            rpc,
        }
    }
    pub fn svm(&mut self) -> &mut LiteSVM {
        &mut self.svm 
    }
    pub async fn airdrop(&mut self, keys: &[(Pubkey, u64)]) {
        for (key, amount) in keys.iter() {
            if let Some(rpc) = &self.rpc {
                rpc.request_airdrop_with_config(
                    key,
                    *amount,
                    RpcRequestAirdropConfig {
                        commitment: Some(CommitmentConfig::finalized()),
                        ..Default::default()
                    }
                ).await.unwrap();
            } else {
                self.svm.airdrop(key, *amount).unwrap();

            }
        }
    }
    pub async fn create_no_auditor_mint(
        &mut self,
        mint_authority: &Keypair,
        mint: &Keypair,
        auditor: &ElGamalKeypair,
    ) -> Result<()> {
        let space = ExtensionType::try_calculate_account_len::<Mint>(&[
            ExtensionType::ConfidentialTransferMint,
            ExtensionType::ConfidentialMintBurn,
        ])?;
        let rent = self.svm.minimum_balance_for_rent_exemption(space);
        let mut ixs = vec![solana_sdk::system_instruction::create_account(
            &mint_authority.pubkey(),
            &mint.pubkey(),
            rent,
            space as u64,
            &spl_token_2022::id()
        )];
        ixs.push(ExtensionInitializationParams::ConfidentialTransferMint {
            authority: Some(mint_authority.pubkey()),
            auto_approve_new_accounts: true, // If `true`, no approval is required and new accounts may be used immediately
            auditor_elgamal_pubkey: Some(
                (*auditor.pubkey()).into()
            ),
        }.instruction(&spl_token_2022::id(), &mint.pubkey())?);
        ixs.push(spl_token_2022::extension::confidential_mint_burn::instruction::initialize_mint(
            &spl_token_2022::id(),
            &mint.pubkey(),
            auditor.pubkey_owned().into(),
            AeKey::new_rand().encrypt(0).into(),
        )?);
        ixs.push(spl_token_2022::instruction::initialize_mint(
            &spl_token_2022::id(),
            &mint.pubkey(),
            &mint_authority.pubkey(),
            Some(&mint_authority.pubkey()),
            6
        )?);
        let block_hash = self.latest_block_hash().await;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&mint_authority.pubkey()),
            &[
                &mint_authority,
                &mint
            ],
            block_hash
        );
        self.send_transaction(tx).await;

        Ok(())
    }
    pub async fn create_token_accounts(
        &mut self,
        sender: &Keypair,
        mint: &Pubkey
    ) {
        let token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
            &sender.pubkey(),
            mint,
            &spl_token_2022::id()
        );
        let mut ixs = vec![spl_associated_token_account::instruction::create_associated_token_account(
            &sender.pubkey(),
            &sender.pubkey(),
            mint,
            &spl_token_2022::id()
        )];

        // realloc to aadd room for the confidential transfer account
        ixs.push(spl_token_2022::instruction::reallocate(
            &spl_token_2022::id(),
            &token_account,
            &sender.pubkey(),
            &sender.pubkey(),
            &[&sender.pubkey()],
            &[ExtensionType::ConfidentialTransferAccount]
        ).unwrap());
        // create elgamal keypair + aes key
        let elgamal_kp = ElGamalKeypair::new_from_signer(sender, &token_account.to_bytes()).unwrap();
        let aes_kp = AeKey::new_from_signer(sender, &token_account.to_bytes()).unwrap();
        
        let decryptable_balance = aes_kp.encrypt(0);

        let proof_data = PubkeyValidityProofData::new(
            &elgamal_kp
        ).unwrap();

        let proof_location = ProofLocation::InstructionOffset(
            1.try_into().unwrap(),
            ProofData::InstructionData(&proof_data)
        );

        ixs.extend(configure_account(
            &spl_token_2022::id(),
            &token_account,
            mint,
            decryptable_balance.into(),
            65536,
            &sender.pubkey(),
            &[],
            proof_location
        ).unwrap());
        log::info!("{ixs:#?}");
        let block_hash = self.latest_block_hash().await;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&sender.pubkey()),
            &[
                &sender,
            ],
            block_hash
        );
        self.send_transaction(tx).await;


    }

    pub async fn mint_confidential_tokens(
        &mut self,
        mint_authority: &Keypair,
        mint: Pubkey,
        recipient: Pubkey,
        mint_amount: u64,
        auditor: &ElGamalKeypair,
    ) {
        let receiving_token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
            &recipient,
            &mint,
            &spl_token_2022::id(),
        );
        let context_state_dummy = Pubkey::new_unique();
        let ixs = spl_token_2022::extension::confidential_mint_burn::instruction::confidential_mint_with_split_proofs(
            &spl_token_2022::id(),
            &receiving_token_account,
            &mint,
            Some(auditor.pubkey_owned()),
            &PodElGamalCiphertext::default(),
            &PodElGamalCiphertext::default(),
            &mint_authority.pubkey(),
            &[&mint_authority.pubkey()],
            ProofLocation::ContextStateAccount(&context_state_dummy),
            ProofLocation::ContextStateAccount(&context_state_dummy),
            ProofLocation::ContextStateAccount(&context_state_dummy),
            AeKey::new_rand().encrypt(mint_amount).into()            
        ).unwrap();
        let block_hash = self.latest_block_hash().await;
        let tx = Transaction::new_signed_with_payer(
            &ixs,
            Some(&mint_authority.pubkey()),
            &[&mint_authority],
            block_hash
        );
        self.send_transaction(tx).await;
    }
    pub async fn mint_tokens(
        &mut self,
        mint_authority: &Keypair,
        mint: Pubkey,
        recipient: Pubkey,
        mint_amount: u64
    ) {
        let receiving_token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
            &recipient,
            &mint,
            &spl_token_2022::id(),
        );

        let ix = spl_token_2022::instruction::mint_to(
            &spl_token_2022::id(),
            &mint,
            &receiving_token_account,
            &mint_authority.pubkey(),
            &[],
            mint_amount
        ).unwrap();
        let block_hash = self.latest_block_hash().await;
        let tx = Transaction::new_signed_with_payer(
            &[ix],
            Some(&mint_authority.pubkey()),
            &[&mint_authority],
            block_hash
        );
        self.send_transaction(tx).await;
    }
    pub async fn deposit_to_confidential_balance(
        &mut self,
        depositor: &Keypair,
        mint: Pubkey,
        amount: u64
    ) {
        let depositor_token_account = spl_associated_token_account::get_associated_token_address_with_program_id(
            &depositor.pubkey(),
            &mint,
            &spl_token_2022::id(),
        );
        let ix = spl_token_2022::extension::confidential_transfer::instruction::deposit(
            &spl_token_2022::id(),
            &depositor_token_account,
            &mint,
            amount,
            6,
            &depositor.pubkey(),
            &[]
        ).unwrap();
    }
    pub async fn send_transaction(&mut self, tx: Transaction) {
        if let Some(rpc) = &self.rpc {
            log::info!("sent tx {}", rpc.send_and_confirm_transaction(&tx).await.unwrap());
        } else {
            let tx = self.svm.send_transaction(tx).unwrap();
            log::info!("{tx:#?}");
        }
    }
    pub async fn latest_block_hash(&mut self) ->  solana_sdk::hash::Hash {
        if let Some(rpc) = &self.rpc {
            rpc.get_latest_blockhash().await.unwrap()
        } else {
            self.svm.latest_blockhash()

        }
    }
}