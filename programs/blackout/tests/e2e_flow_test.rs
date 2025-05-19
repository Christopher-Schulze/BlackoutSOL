/*
 * End-to-End Test für den BlackoutSOL-Workflow
 * 
 * Dieser Test führt den vollständigen Workflow von BlackoutSOL durch:
 * - Initialisierung eines anonymen Transfers
 * - Ausführung von Hops
 * - Batch-Hop-Ausführung
 * - Finalisierung des Transfers
 */

use anchor_lang::prelude::*;
use blackout::instructions::*;
use blackout::state::*;
use blackout::utils::*;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::account::Account;
use solana_sdk::signature::{Keypair, Signer};
use solana_sdk::transaction::Transaction;
use solana_sdk::system_instruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::account_info::AccountInfo;
use std::str::FromStr;

mod common {
    use super::*;
    
    // Test-Konstanten
    pub const TRANSFER_AMOUNT: u64 = 1_000_000_000; // 1 SOL
    pub const NUM_HOPS: u8 = 3;
    pub const NUM_SPLITS: u8 = 4;
    
    // Generiert Testdaten
    pub fn generate_test_data() -> ([u8; 128], [u8; 128], [u8; 32], Vec<u8>) {
        let hyperplonk_proof = [42u8; 128];
        let range_proof = [43u8; 128];
        let challenge = [44u8; 32];
        let merkle_proof = vec![45u8; 64];
        
        (hyperplonk_proof, range_proof, challenge, merkle_proof)
    }
    
    // Erstellt einen TransferState mit zufälligen Werten
    pub async fn create_transfer_state(
        program_test: &mut ProgramTest,
        payer: &Keypair,
    ) -> (Keypair, Pubkey) {
        let transfer_state = Keypair::new();
        let seed = [42u8; 32];
        let fake_bloom = [0u8; 16];

        let state = transfer::TransferState {
            owner: payer.pubkey(),
            amount: TRANSFER_AMOUNT,
            current_hop: 0,
            seed,
            fake_bloom,
            refund_triggered: false,
            completed: false,
            bump: 255,
            timestamp: 0,
        };
        
        let mut account_data = Vec::new();
        state.try_serialize(&mut account_data).unwrap();
        
        program_test.add_account(
            transfer_state.pubkey(),
            Account {
                lamports: TRANSFER_AMOUNT + 1_000_000, // Für Rent Exemption
                data: account_data,
                owner: blackout::ID,
                executable: false,
                rent_epoch: 0,
            },
        );
        
        // Berechne PDA für den ersten Hop
        let (pda, _) = derive_stealth_pda(
            &blackout::ID,
            &seed,
            0,  // Hop-Index
            0,  // Split-Index
            false, // Nicht ein Fake-Split
        );
        
        (transfer_state, pda)
    }
}

// End-to-End Test des BlackoutSOL-Workflows
#[tokio::test]
async fn test_e2e_blackout_flow() {
    use common::*;
    
    // 1. Test-Umgebung aufsetzen
    let program_id = blackout::ID;
    let mut program_test = ProgramTest::new(
        "blackout",
        program_id,
        processor!(blackout::process_instruction),
    );
    
    // 2. Test-Accounts erstellen
    let payer = Keypair::new();
    program_test.add_account(
        payer.pubkey(),
        Account {
            lamports: 10_000_000_000, // 10 SOL
            data: vec![],
            owner: solana_program::system_program::ID,
            executable: false,
            rent_epoch: 0,
        },
    );
    
    // 3. TransferState erstellen und PDAs vorbereiten
    let (transfer_state, first_pda) = create_transfer_state(&mut program_test, &payer).await;
    
    // 4. Test starten
    let (mut banks_client, payer, recent_blockhash) = program_test.start().await;
    
    // 5. Testdaten generieren
    let (hyperplonk_proof, range_proof, challenge, merkle_proof) = generate_test_data();
    
    // ----------------------------------------
    // Schritt 1: Initialisierung
    // ----------------------------------------
    println!("🚀 Initialisierung des Transfers...");
    
    let initialize_ix = initialize::initialize(
        &program_id,
        &payer.pubkey(),
        &transfer_state.pubkey(),
        TRANSFER_AMOUNT,
        hyperplonk_proof,
        range_proof,
        challenge,
        merkle_proof,
    );
    
    let initialize_tx = Transaction::new_signed_with_payer(
        &[initialize_ix],
        Some(&payer.pubkey()),
        &[&payer, &transfer_state],
        recent_blockhash,
    );
    
    banks_client.process_transaction(initialize_tx).await.expect("Initialisierung fehlgeschlagen");
    
    // Transfer-Zustand überprüfen
    let transfer_account = banks_client
        .get_account(transfer_state.pubkey())
        .await
        .expect("Fehler beim Abrufen des Accounts")
        .expect("Account nicht gefunden");
    
    let state = transfer::TransferState::try_deserialize(&mut transfer_account.data.as_slice()).unwrap();
    assert_eq!(state.amount, TRANSFER_AMOUNT, "Übertragsbetrag stimmt nicht überein");
    assert_eq!(state.current_hop, 0, "Hop-Index sollte 0 sein");
    
    // ----------------------------------------
    // Schritt 2: Ersten Hop ausführen
    // ----------------------------------------
    println!("🔄 Führe ersten Hop aus...");
    
    let hop_data = [42u8; 128];
    
    let execute_hop_ix = execute_hop::execute_hop(
        &program_id,
        &payer.pubkey(),
        &transfer_state.pubkey(),
        &first_pda,
        0, // Hop-Index
        hop_data,
        range_proof,
    );
    
    let execute_hop_tx = Transaction::new_signed_with_payer(
        &[execute_hop_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    banks_client.process_transaction(execute_hop_tx).await.expect("Hop-Ausführung fehlgeschlagen");
    
    // Transfer-Zustand überprüfen
    let transfer_account = banks_client
        .get_account(transfer_state.pubkey())
        .await
        .expect("Fehler beim Abrufen des Accounts")
        .expect("Account nicht gefunden");
    
    let state = transfer::TransferState::try_deserialize(&mut transfer_account.data.as_slice()).unwrap();
    assert_eq!(state.current_hop, 1, "Hop-Index sollte nach Ausführung 1 sein");
    
    // ----------------------------------------
    // Schritt 3: Batch-Hop ausführen
    // ----------------------------------------
    println!("🔄 Führe Batch-Hop aus...");
    
    // PDAs für den Batch-Hop ableiten
    let seed = [42u8; 32];
    let (pda1, _) = derive_stealth_pda(&program_id, &seed, 1, 0, false);
    let (pda2, _) = derive_stealth_pda(&program_id, &seed, 2, 0, false);
    
    // Batch-Hop-Ausführung
    let batch_hop_ix = batch_hop::process_batch_hop(
        &program_id,
        &payer.pubkey(),
        &transfer_state.pubkey(),
        &pda1,
        Some(&pda2),
        None, // pda3
        None, // pda4
        None, // pda5
        None, // pda6
        None, // pda7
        &solana_program::system_program::ID,
        0, // Batch-Index
    );
    
    let batch_hop_tx = Transaction::new_signed_with_payer(
        &[batch_hop_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    banks_client.process_transaction(batch_hop_tx).await.expect("Batch-Hop-Ausführung fehlgeschlagen");
    
    // Transfer-Zustand überprüfen
    let transfer_account = banks_client
        .get_account(transfer_state.pubkey())
        .await
        .expect("Fehler beim Abrufen des Accounts")
        .expect("Account nicht gefunden");
    
    let state = transfer::TransferState::try_deserialize(&mut transfer_account.data.as_slice()).unwrap();
    assert_eq!(state.current_hop, 3, "Hop-Index sollte nach Batch-Ausführung 3 sein");
    
    // ----------------------------------------
    // Schritt 4: Transfer finalisieren
    // ----------------------------------------
    println!("✅ Finalisiere den Transfer...");
    
    // Finale Ausführung
    let recipient = Keypair::new();
    let finalize_ix = finalize::finalize_transfer(
        &program_id,
        &payer.pubkey(),
        &transfer_state.pubkey(),
        &recipient.pubkey(),
        hyperplonk_proof,
    );
    
    let finalize_tx = Transaction::new_signed_with_payer(
        &[finalize_ix],
        Some(&payer.pubkey()),
        &[&payer],
        recent_blockhash,
    );
    
    banks_client.process_transaction(finalize_tx).await.expect("Finalisierung fehlgeschlagen");
    
    // Überprüfe, ob der Transfer abgeschlossen ist
    let transfer_account = banks_client
        .get_account(transfer_state.pubkey())
        .await
        .expect("Fehler beim Abrufen des Accounts")
        .expect("Account nicht gefunden");
    
    let state = transfer::TransferState::try_deserialize(&mut transfer_account.data.as_slice()).unwrap();
    assert!(state.completed, "Transfer sollte als abgeschlossen markiert sein");
    
    // Überprüfe, ob der Empfänger die Gelder erhalten hat
    let recipient_account = banks_client
        .get_account(recipient.pubkey())
        .await
        .expect("Fehler beim Abrufen des Empfänger-Accounts")
        .expect("Empfänger-Account nicht gefunden");
    
    // Empfänger sollte den Transferbetrag abzüglich Gebühren erhalten haben
    // Die genaue Gebührenberechnung kann variieren, daher prüfen wir nur auf einen groben Bereich
    let expected_min = TRANSFER_AMOUNT - (TRANSFER_AMOUNT / 20); // 95% des Betrags
    assert!(recipient_account.lamports >= expected_min, "Empfänger hat nicht genug Lamports erhalten");
    
    println!("🎉 End-to-End Test erfolgreich abgeschlossen!");
}
