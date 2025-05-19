use anchor_lang::prelude::*;
use anchor_lang::solana_program::{
    instruction::Instruction,
    pubkey::Pubkey,
    hash::Hash,
    system_program,
};
use std::str::FromStr;
use solana_program_test::{
    ProgramTest, ProgramTestBanksClient, ProgramTestContext,
    processor,
};
use solana_sdk::{
    account::Account,
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
};

use blackout::{
    instructions::*,
    state::*,
};

/// Test framework for BlackoutSOL tests
/// 
/// Provides helper functions for common test operations and test fixtures
pub struct BlackoutTestFramework {
    /// Program context for tests
    pub context: ProgramTestContext,
    /// Program ID of BlackoutSOL
    pub program_id: Pubkey,
    /// User keypair for tests
    pub user: Keypair,
    /// Available configuration presets
    pub config: BlackoutConfig,
    /// Current transaction (for vectorized tests)
    current_tx: Option<Transaction>,
}

impl BlackoutTestFramework {
    /// Initializes the test framework with a fresh environment
    pub async fn new() -> Self {
        // Load Blackout program
        let program_id = Pubkey::from_str("B1acKoutso111111111111111111111111111111111").unwrap();
        let mut program_test = ProgramTest::new(
            "blackout",
            program_id,
            processor!(blackout::entry),
        );
        
        // Create test context
        let mut context = program_test.start_with_context().await;
        
        // Create test user with balance
        let user = Keypair::new();
        context.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[system_instruction::transfer(
                    &context.payer.pubkey(),
                    &user.pubkey(),
                    1_000_000_000, // 1 SOL
                )],
                Some(&context.payer.pubkey()),
                &[&context.payer],
                context.last_blockhash,
            ))
            .await
            .expect("Error transferring SOL to test wallet");
        
        // Create default configuration
        let config = BlackoutConfig::new();
        
        Self {
            context,
            program_id,
            user,
            config,
            current_tx: None,
        }
    }
    
    /// Executes a transaction with the given instructions
    pub async fn execute_transaction(&mut self, 
                                    instructions: &[Instruction], 
                                    signers: &[&Keypair]) 
        -> std::result::Result<(), TransportError> 
    {
        let mut tx_signers = Vec::with_capacity(signers.len() + 1);
        tx_signers.push(&self.context.payer);
        tx_signers.extend(signers);
        
        let transaction = Transaction::new_signed_with_payer(
            instructions,
            Some(&self.context.payer.pubkey()),
            &tx_signers,
            self.context.last_blockhash,
        );
        
        self.context.banks_client.process_transaction(transaction).await.map_err(|e| e.into())
    }
    
    /// Generates a new seed for an anonymous transfer
    pub fn generate_transfer_seed() -> [u8; 32] {
        let mut seed = [0u8; 32];
        for i in 0..32 {
            seed[i] = rand::random::<u8>();
        }
        seed
    }
    
    /// Creates a mock HyperPlonk proof for tests
    pub fn create_test_hyperplonk_proof(challenge: &[u8; 32], splits: &[u64]) -> [u8; 128] {
        let mut proof = [0u8; 128];
        
        // 'PS' signature for Poseidon-Schnorr
        proof[0] = 0x50;
        proof[1] = 0x53;
        
        // Public Inputs
        for i in 0..std::cmp::min(32, challenge.len()) {
            proof[i + 2] = challenge[i];
        }
        
        // Commitments
        for i in 0..std::cmp::min(4, splits.len()) {
            let bytes = splits[i].to_le_bytes();
            for j in 0..8 {
                proof[34 + i*8 + j] = bytes[j];
            }
        }
        
        proof
    }
    
    /// Creates a mock Plonky2 Range-Proof for tests
    pub fn create_test_range_proof(challenge: &[u8; 32], 
                               commitments: &[[u8; 32]; 8], 
                               sum_check: bool) -> [u8; 128] {
        let mut proof = [0u8; 128];
        
        // 'P2R1' signature for Plonky2 Range-Proof v1
        proof[0] = 0x50;
        proof[1] = 0x32;
        proof[2] = 0x52;
        proof[3] = 0x31;
        
        // Inner Verification Key
        for i in 0..16 {
            proof[4 + i] = i as u8;
        }
        
        // Wire Commitments
        for i in 0..32 {
            proof[20 + i] = commitments[0][i];
        }
        
        // Proof Polynomials
        for i in 0..32 {
            proof[52 + i] = (i as u8).wrapping_add(commitments[1][0]);
        }
        
        // Opening Proof
        for i in 0..32 {
            proof[84 + i] = challenge[i];
        }
        
        // Public Values
        proof[116] = 0x1;  // Range check bit für Split 0 und 1
        proof[117] = 0x1;  // Range check bit für Split 2 und 3
        proof[118] = 0x0A; // Min-max flag
        
        // Sum-Check (PSMC - Poseidon Sum Check)
        if sum_check {
            proof[124] = 0x50; // 'P'
            proof[125] = 0x53; // 'S'
            proof[126] = 0x4D; // 'M'
            proof[127] = 0x43; // 'C'
        }
        
        proof
    }
    
    /// Creates a Bloom filter for fake splits
    pub fn create_fake_bloom_filter(fake_indices: &[u8]) -> [u8; 16] {
        let mut bloom = [0u8; 16];
        
        for &index in fake_indices {
            let position = index as usize % 128;
            let byte_index = position / 8;
            let bit_index = position % 8;
            
            bloom[byte_index] |= 1 << bit_index;
        }
        
        bloom
    }
    
    /// Calculates the expected PDA for a split
    pub fn derive_split_pda(&self, 
                          seed: &[u8; 32], 
                          hop_index: u8, 
                          split_index: u8, 
                          is_fake: bool) -> (Pubkey, u8) {
        let prefix = if is_fake { b"fake" } else { b"split" };
        
        Pubkey::find_program_address(
            &[
                prefix,
                &hop_index.to_le_bytes(),
                &split_index.to_le_bytes(),
                seed,
            ],
            &self.program_id,
        )
    }
    
    /// Finds the PDA for a transfer state
    pub fn find_transfer_pda(&self, owner: &Pubkey) -> (Pubkey, u8) {
        Pubkey::find_program_address(
            &[b"transfer", owner.as_ref()],
            &self.program_id,
        )
    }
    
    /// Initializes a new transfer for testing
    pub async fn initialize_transfer(&mut self, 
                                  amount: u64, 
                                  reserve_percent: u8) 
        -> Result<(Pubkey, [u8; 32]), TransportError> 
    {
        let seed = Self::generate_transfer_seed();
        let (transfer_pda, bump) = self.find_transfer_pda(&self.user.pubkey());
        
        let ix = initialize_instruction(
            initialize::InitializeParams {
                amount,
                reserve_percent,
                seed,
            },
            initialize::InitializeAccounts {
                authority: self.user.pubkey(),
                transfer_state: transfer_pda,
                system_program: system_program::id(),
            },
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await?;
        
        Ok((transfer_pda, seed))
    }
    
    /// Executes a hop
    pub async fn execute_hop(&mut self, 
                         transfer_pda: &Pubkey, 
                         seed: &[u8; 32], 
                         hop_index: u8,
                         split_index: u8,
                         is_fake: bool) 
        -> Result<(), TransportError> 
    {
        let (split_pda, _) = self.derive_split_pda(seed, hop_index, split_index, is_fake);
        
        // Retrieve the current transfer status
        let transfer_account = self.context.banks_client
            .get_account(*transfer_pda)
            .await
            .expect("Error retrieving transfer account")
            .expect("Transfer account not found");
            
        let transfer_state = TransferState::try_deserialize(
            &mut transfer_account.data.as_ref()
        ).expect("Fehler beim Deserialisieren des Transfer-States");
        
        // Generiere Proofs
        let challenge = [0u8; 32]; // Simplified challenge for tests
        let splits = [
            transfer_state.amount / 4,
            transfer_state.amount / 4,
            transfer_state.amount / 4,
            transfer_state.amount / 4,
        ];
        
        let proof_data = Self::create_test_hyperplonk_proof(&challenge, &splits);
        let range_proof_data = Self::create_test_range_proof(
            &challenge, 
            &transfer_state.commitments, 
            true
        );
        
        // Execute the hop
        let ix = execute_hop_instruction(
            execute_hop::ExecuteHopParams {
                hop_index,
                proof_data,
                range_proof_data,
            },
            execute_hop::ExecuteHopAccounts {
                authority: self.user.pubkey(),
                transfer_state: *transfer_pda,
                split_pda,
                compute_budget: self.program_id, // Dummy for tests
                system_program: system_program::id(),
                clock: solana_program::sysvar::clock::id(),
            },
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await
    }
    
    /// Executes a complete transfer (all 4 hops)
    pub async fn execute_complete_transfer(&mut self, 
                                        amount: u64) 
        -> Result<(Pubkey, [u8; 32]), TransportError> 
    {
        // 1. Initialize transfer
        let (transfer_pda, seed) = self.initialize_transfer(amount, 5).await?;
        
        // 2. Execute all 4 hops (1 per hop)
        for hop_index in 0..4 {
            self.execute_hop(&transfer_pda, &seed, hop_index, 0, false).await?;
        }
        
        Ok((transfer_pda, seed))
    }
    
    /// Executes a batch hop
    pub async fn execute_batch_hop(&mut self, 
                                transfer_pda: &Pubkey,
                                batch_index: u8,
                                pdas: &[Pubkey]) 
        -> Result<(), TransportError> 
    {
        // Construct all account metas for the PDAs
        let mut accounts = batch_hop::BatchHopAccounts {
            authority: self.user.pubkey(),
            transfer_state: *transfer_pda,
            pda_0: pdas[0],
            pda_1: if pdas.len() > 1 { Some(pdas[1]) } else { None },
            pda_2: if pdas.len() > 2 { Some(pdas[2]) } else { None },
            pda_3: if pdas.len() > 3 { Some(pdas[3]) } else { None },
            pda_4: if pdas.len() > 4 { Some(pdas[4]) } else { None },
            pda_5: if pdas.len() > 5 { Some(pdas[5]) } else { None },
            pda_6: if pdas.len() > 6 { Some(pdas[6]) } else { None },
            pda_7: if pdas.len() > 7 { Some(pdas[7]) } else { None },
            system_program: system_program::id(),
        };
        
        let ix = batch_hop_instruction(
            batch_hop::BatchHopParams { batch_index },
            accounts,
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await
    }
    
    /// Finalizes a transfer
    pub async fn finalize_transfer(&mut self, 
                                transfer_pda: &Pubkey,
                                recipient: &Pubkey) 
        -> Result<(), TransportError> 
    {
        let ix = finalize_instruction(
            finalize::FinalizeParams {},
            finalize::FinalizeAccounts {
                authority: self.user.pubkey(),
                transfer_state: *transfer_pda,
                recipient: *recipient,
                system_program: system_program::id(),
            },
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await
    }
    
    /// Processes a refund for an incomplete transfer
    pub async fn refund_transfer(&mut self, 
                              transfer_pda: &Pubkey) 
        -> Result<(), TransportError> 
    {
        let ix = refund_instruction(
            refund::RefundParams {},
            refund::RefundAccounts {
                authority: self.user.pubkey(),
                transfer_state: *transfer_pda,
                system_program: system_program::id(),
            },
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await
    }
    
    /// Reveals a fake split
    pub async fn reveal_fake(&mut self, 
                          transfer_pda: &Pubkey,
                          hop_index: u8,
                          split_index: u8) 
        -> Result<(), TransportError> 
    {
        let ix = reveal_fake_instruction(
            reveal_fake::RevealFakeParams {
                hop_index,
                split_index,
            },
            reveal_fake::RevealFakeAccounts {
                authority: self.user.pubkey(),
                transfer_state: *transfer_pda,
            },
        );
        
        self.execute_transaction(&[ix], &[&self.user]).await
    }
}
