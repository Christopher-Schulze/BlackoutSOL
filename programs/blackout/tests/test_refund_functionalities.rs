/// Tests for the optimized refund functionality
///
/// These tests validate that the refund mechanics work correctly,
/// including the calculation of fees, reserves, and the proper
/// transfer of Lamports back to the owner.

use anchor_lang::prelude::*;
use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentLevel,
};

mod test_framework;
use test_framework::BlackoutTestFramework;
use blackout::{
    state::*,
    errors::BlackoutError,
};

// Integration test for the complete refund process
// Verifies:
// - Correct initialization of a transfer
// - Partial execution of a transfer (1-2 hops)
// - Refund process with correct amount calculation
// - Verification of protection against duplicate refunds
#[tokio::test]
async fn test_refund_full_process() {
    // 1. Test-Framework initialisieren
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Transfer initialisieren mit 1 SOL
    let amount = 1_000_000_000; // 1 SOL in Lamports
    let reserve_percent = 15; // 15% Reserve
    
    // Benutzer-Saldo vor der Transaktion speichern
    let initial_balance = framework.get_account_balance(&framework.user.pubkey()).await
        .expect("Konnte Benutzer-Saldo nicht abrufen");
    
    // Initialize transfer with custom reserves
    let (transfer_pda, seed) = framework.initialize_transfer(amount, reserve_percent)
        .await
        .expect("Transfer-Initialisierung fehlgeschlagen");
    
    println!("Transfer initialized: 1 SOL with 15% reserve");
    
    // 3. Partially execute transaction (only 2 of 4 hops)
    // Execute first hop
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Execution of first hop failed");
    
    // Execute second hop
    framework.execute_hop(&transfer_pda, &seed, 1, 0, false)
        .await
        .expect("Execution of second hop failed");
    
    println!("2 of 4 hops successfully executed (50% completed)");
    
    // 4. Check transfer state
    let transfer_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state");
    
    assert_eq!(transfer_state.current_hop, 2, "Incorrect hop index after 2 hops");
    assert_eq!(transfer_state.progress_percent(), 50, "Incorrect progress after 2 hops");
    assert!(!transfer_state.completed, "Transfer should not be completed");
    assert!(!transfer_state.refund_triggered, "Refund should not be triggered");
    assert_eq!(transfer_state.current_hop, 2, "Incorrect hop index after 2 hops");
    assert_eq!(transfer_state.progress_percent(), 50, "Incorrect progress after 2 hops");
    assert!(!transfer_state.completed, "Transfer should not be completed");
    assert!(!transfer_state.refund_triggered, "Refund should not be triggered");
    
    // 5. Request refund
    framework.refund_transfer(&transfer_pda)
        .await
        .expect("Refund failed");
    
    println!("Refund requested");
    
    // 6. Check refund status
    let refunded_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state after refund");
    
    assert!(refunded_state.refund_triggered, "Refund flag was not set");
    
    // 7. Verify that the user received ~95% of the amount back
    // (minus transaction fees for tests)
    let final_balance = framework.get_account_balance(&framework.user.pubkey()).await
        .expect("Could not retrieve final user balance");
    
    // Refund should be approximately 95% of the original amount
    let expected_refund = (amount as f64 * 0.95) as u64;
    
    // Allow a small tolerance for transaction fees (~0.01 SOL)
    let tolerance = 10_000_000; // 0.01 SOL
    
    let balance_diff = if final_balance > initial_balance {
        final_balance - initial_balance
    } else {
        // In tests, transaction fees can exceed the refund
        0
    };
    
    // Check with tolerance
    assert!(
        balance_diff + tolerance >= expected_refund || 
        expected_refund <= balance_diff + tolerance,
        "Unexpected refund amount: {:?}, expected approximately: {:?}",
        balance_diff, expected_refund
    );
    
    // 8. Perform second refund attempt (should fail)
    let result = framework.refund_transfer(&transfer_pda).await;
    
    assert!(
        result.is_err(),
        "Second refund should fail, but was successful"
    );
    
    println!("Test successful: Double refund was prevented");
}

// Test for the refund process without prior hop execution
// Verifies: 
// - Full refund of a transfer that has not started
// - Correct calculation of DEV fees
#[tokio::test]
async fn test_refund_immediately_after_init() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Initialize transfer with 0.5 SOL
    let amount = 500_000_000; // 0.5 SOL
    let reserve_percent = 20; // 20% Reserve
    
    // Store user balance before transaction
    let initial_balance = framework.get_account_balance(&framework.user.pubkey()).await
        .expect("Could not retrieve user balance");
    
    // Initialize transfer
    let (transfer_pda, _) = framework.initialize_transfer(amount, reserve_percent)
        .await
        .expect("Transfer initialization failed");
    
    println!("Transfer initialized: 0.5 SOL with 20% reserve");
    
    // 3. Perform immediate refund without hops
    framework.refund_transfer(&transfer_pda)
        .await
        .expect("Refund failed");
    
    // 4. Check refund status
    let refunded_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state after refund");
    
    assert!(refunded_state.refund_triggered, "Refund flag was not set");
    assert_eq!(refunded_state.current_hop, 0, "Current hop should be 0");
    assert_eq!(refunded_state.progress_percent(), 0, "Progress should be 0%");
    
    // 5. Verify correct refund amounts
    let final_balance = framework.get_account_balance(&framework.user.pubkey()).await
        .expect("Could not retrieve final user balance");
    
    // Refund should be approximately 95% of the original amount
    let expected_refund = (amount as f64 * 0.95) as u64;
    
    // Tolerance for transaction fees
    let tolerance = 10_000_000; // 0.01 SOL
    
    let balance_diff = if final_balance > initial_balance {
        final_balance - initial_balance
    } else {
        // In tests, transaction fees can exceed the refund
        0
    };
    
    // Check with tolerance
    assert!(
        balance_diff + tolerance >= expected_refund || 
        expected_refund <= balance_diff + tolerance,
        "Unexpected refund amount: {:?}, expected approximately: {:?}",
        balance_diff, expected_refund
    );
    
    println!("Test successful: Immediate refund after initialization works");
}

// Test for the finalization process with reserve calculation 
// Verifies:
// - Full execution of a transfer (all 4 hops)
// - Correct calculation and distribution of the reserve amount
// - Correct transfer to the recipient
#[tokio::test]
async fn test_finalize_with_reserve() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Initialize transfer with 1 SOL
    let amount = 1_000_000_000; // 1 SOL in Lamports
    let reserve_percent = 25; // 25% Reserve
    
    // Create recipient
    let recipient = Keypair::new();
    
    // Transfer SOL to recipient for account creation
    framework.fund_account(&recipient.pubkey(), 100_000_000).await
        .expect("Could not fund recipient account");
    
    // Store recipient balance before transfer
    let initial_recipient_balance = framework.get_account_balance(&recipient.pubkey()).await
        .expect("Could not retrieve recipient balance");
    
    // Initialize transfer with defined recipient
    let (transfer_pda, seed) = framework.initialize_transfer_with_recipient(
        amount, 
        reserve_percent, 
        &recipient.pubkey()
    ).await.expect("Transfer initialization failed");
    
    println!("Transfer initialized: 1 SOL with 25% reserve");
    
    // 3. Execute all 4 hops
    for hop in 0..4 {
        framework.execute_hop(&transfer_pda, &seed, hop, 0, false)
            .await
            .expect(&format!("Execution of hop {} failed", hop));
        
        println!("Hop {} successfully executed", hop);
    }
    
    // 4. Check transfer state
    let transfer_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state");
    
    assert_eq!(transfer_state.current_hop, 4, "Incorrect hop index after 4 hops");
    assert_eq!(transfer_state.progress_percent(), 100, "Incorrect progress after 4 hops");
    assert!(!transfer_state.completed, "Transfer should not be finalized");
    
    // 5. Finalize transfer
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey())
        .await
        .expect("Finalization failed");
    
    println!("Transfer successfully finalized");
    
    // 6. Check final recipient balance
    let final_recipient_balance = framework.get_account_balance(&recipient.pubkey()).await
        .expect("Could not retrieve final recipient balance");
    
    // Expected amount: Original amount minus 25% reserve
    let expected_amount = (amount as f64 * 0.75) as u64;
    let received_amount = final_recipient_balance - initial_recipient_balance;
    
    // Toleranz für Rundungsunterschiede
    let tolerance = 1000; // 0.000001 SOL Toleranz
    
    assert!(
        (received_amount as i64 - expected_amount as i64).abs() <= tolerance as i64,
        "Unerwarteter Empfängerbetrag: {}, erwartet: {}", 
        received_amount, expected_amount
    );
    
    println!("Test erfolgreich: Korrekter Betrag mit Reserveabzug an Empfänger übertragen");
}

// Erweitere die BlackoutTestFramework um spezifische Hilfsmethoden
impl BlackoutTestFramework {
    // Abrufen eines Kontostandards
    pub async fn get_account_balance(&mut self, pubkey: &Pubkey) -> Result<u64, BanksClientError> {
        let account = self.context.banks_client
            .get_account(*pubkey)
            .await?
            .expect(&format!("Konto existiert nicht: {}", pubkey));
        
        Ok(account.lamports)
    }
    
    // Abrufen des Transfer-States
    pub async fn get_transfer_state(&mut self, pda: &Pubkey) -> Result<TransferState, BanksClientError> {
        let account = self.context.banks_client
            .get_account(*pda)
            .await?
            .expect(&format!("Transfer-State existiert nicht: {}", pda));
        
        let transfer_state: TransferState = anchor_lang::AccountDeserialize::try_deserialize(
            &mut account.data.as_ref()
        ).expect("Konnte Transfer-State nicht deserialisieren");
        
        Ok(transfer_state)
    }
    
    // Ein Konto mit SOL finanzieren
    pub async fn fund_account(&mut self, recipient: &Pubkey, amount: u64) -> Result<(), BanksClientError> {
        self.context.banks_client
            .process_transaction(Transaction::new_signed_with_payer(
                &[system_instruction::transfer(
                    &self.context.payer.pubkey(),
                    recipient,
                    amount,
                )],
                Some(&self.context.payer.pubkey()),
                &[&self.context.payer],
                self.context.last_blockhash,
            ))
            .await
    }
    
    // Transfer mit spezifischem Empfänger initialisieren
    pub async fn initialize_transfer_with_recipient(
        &mut self,
        amount: u64, 
        reserve_percent: u8,
        recipient: &Pubkey,
    ) -> Result<(Pubkey, [u8; 32]), BanksClientError> {
        // Konfiguration mit angepasster Reserve erstellen
        let mut config = BlackoutConfig::new();
        config.reserve_percent = reserve_percent;
        
        // Challenge und Seed generieren
        let challenge = [7u8; 32]; // Fester Wert für einfacheres Testen
        let seed = self.generate_transfer_seed();
        
        // Proof-Daten für Tests erstellen
        let hyperplonk_proof = self.create_test_hyperplonk_proof(&challenge, &[amount / 4; 4]);
        let range_proof = self.create_test_range_proof(&challenge, &[[0; 32]; 8], true);
        
        // Merkle-Root erstellen
        let merkle_root = Keypair::new();
        
        // Transfer-PDA berechnen
        let (transfer_pda, _) = self.find_transfer_pda(&self.user.pubkey());
        
        // Fake-Bloom-Filter erstellen
        let fake_bloom = self.create_fake_bloom_filter(&[5, 10, 15, 20]);
        
        // Initialisierungsinstruktion erstellen
        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new(transfer_pda, false),
                AccountMeta::new_readonly(*recipient, false),
                AccountMeta::new_readonly(merkle_root.pubkey(), false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_program::sysvar::clock::ID, false),
            ],
            data: blackout::instruction::Initialize {
                amount,
                hyperplonk_proof,
                range_proof,
                challenge,
                merkle_proof: vec![],
            }.data(),
        };
        
        // Transaktion ausführen
        self.execute_transaction(&[ix], &[&self.user, &merkle_root]).await?;
        
        Ok((transfer_pda, seed))
    }
}
