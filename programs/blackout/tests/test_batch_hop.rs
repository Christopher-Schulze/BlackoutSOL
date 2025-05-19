use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    transport::TransportError,
    pubkey::Pubkey,
};
use solana_program::system_program;

use blackout::{
    state::*,
    instructions::*,
};

mod test_framework;
use test_framework::BlackoutTestFramework;

#[tokio::test]
async fn test_batch_hop_basic() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 100_000_000; // 0.1 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. PDAs für den ersten Batch erzeugen (2 Hops)
    let mut pdas = Vec::new();
    
    // Erstes Set: Hop 0, Split 0
    let (split_pda_0_0, _) = framework.derive_split_pda(&seed, 0, 0, false);
    pdas.push(split_pda_0_0);
    
    // Zweites Set: Hop 1, Split 0
    let (split_pda_1_0, _) = framework.derive_split_pda(&seed, 1, 0, false);
    pdas.push(split_pda_1_0);
    
    // 3. Batch-Hop ausführen
    framework.execute_batch_hop(&transfer_pda, 0, &pdas).await?;
    
    // 4. Verify dass 2 Hops ausgeführt wurden
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.current_hop, 2);
    assert_eq!(transfer_state.batch_count, 1);
    
    // 5. Verify dass die PDAs erzeugt wurden
    for pda in &pdas {
        let account = framework.context.banks_client
            .get_account(*pda)
            .await?;
        
        assert!(account.is_some(), "PDA sollte existieren");
        assert!(account.unwrap().lamports > 0, "PDA sollte Lamports erhalten haben");
    }
    
    Ok(())
}

#[tokio::test]
async fn test_batch_hop_multi_batch() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 500_000_000; // 0.5 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Ersten Batch ausführen (2 Hops)
    let mut pdas_batch_1 = Vec::new();
    let (split_pda_0_0, _) = framework.derive_split_pda(&seed, 0, 0, false);
    let (split_pda_1_0, _) = framework.derive_split_pda(&seed, 1, 0, false);
    pdas_batch_1.push(split_pda_0_0);
    pdas_batch_1.push(split_pda_1_0);
    
    framework.execute_batch_hop(&transfer_pda, 0, &pdas_batch_1).await?;
    
    // 3. Verify erster Batch
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.current_hop, 2);
    assert_eq!(transfer_state.batch_count, 1);
    
    // 4. Zweiten Batch ausführen (2 Hops)
    let mut pdas_batch_2 = Vec::new();
    let (split_pda_2_0, _) = framework.derive_split_pda(&seed, 2, 0, false);
    let (split_pda_3_0, _) = framework.derive_split_pda(&seed, 3, 0, false);
    pdas_batch_2.push(split_pda_2_0);
    pdas_batch_2.push(split_pda_3_0);
    
    framework.execute_batch_hop(&transfer_pda, 1, &pdas_batch_2).await?;
    
    // 5. Verify zweiter Batch
    let final_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let final_state = TransferState::try_deserialize(
        &mut final_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(final_state.current_hop, 4);
    assert_eq!(final_state.batch_count, 2);
    
    // 6. Alle PDAs prüfen
    for pda in pdas_batch_1.iter().chain(pdas_batch_2.iter()) {
        let account = framework.context.banks_client
            .get_account(*pda)
            .await?;
        
        assert!(account.is_some(), "PDA sollte existieren");
        assert!(account.unwrap().lamports > 0, "PDA sollte Lamports erhalten haben");
    }
    
    // 7. Transfer finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 8. Verify Finalisierung
    let finalized_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let finalized_state = TransferState::try_deserialize(
        &mut finalized_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(finalized_state.completed, true);
    
    Ok(())
}

#[tokio::test]
async fn test_batch_hop_fake_splits() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 200_000_000; // 0.2 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Batch mit echten und Fake-Splits
    let mut pdas = Vec::new();
    
    // Real-Split für Hop 0
    let (real_pda_0, _) = framework.derive_split_pda(&seed, 0, 0, false);
    pdas.push(real_pda_0);
    
    // Fake-Split für Hop 0
    let (fake_pda_0, _) = framework.derive_split_pda(&seed, 0, 1, true);
    pdas.push(fake_pda_0);
    
    // Real-Split für Hop 1
    let (real_pda_1, _) = framework.derive_split_pda(&seed, 1, 0, false);
    pdas.push(real_pda_1);
    
    // Fake-Split für Hop 1
    let (fake_pda_1, _) = framework.derive_split_pda(&seed, 1, 1, true);
    pdas.push(fake_pda_1);
    
    // 3. Batch-Hop mit Fake-Split-Optimierung ausführen
    framework.execute_batch_hop(&transfer_pda, 0, &pdas).await?;
    
    // 4. Verify Hops wurden ausgeführt
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.current_hop, 2);
    
    // 5. Verify Real-Splits haben Lamports, Fake-Splits haben weniger
    let real_account_0 = framework.context.banks_client
        .get_account(real_pda_0)
        .await?
        .expect("Real-Split PDA sollte existieren");
    
    let fake_account_0 = framework.context.banks_client
        .get_account(fake_pda_0)
        .await?;
    
    // Bei primären Fake-Splits sollte Account existieren, aber mit weniger Lamports
    if let Some(fake_acc) = fake_account_0 {
        assert!(fake_acc.lamports < real_account_0.lamports, 
                "Fake-Split sollte weniger Lamports haben als Real-Split");
    }
    
    // 6. Rest der Hops und Finalisierung
    let mut final_pdas = Vec::new();
    let (split_pda_2_0, _) = framework.derive_split_pda(&seed, 2, 0, false);
    let (split_pda_3_0, _) = framework.derive_split_pda(&seed, 3, 0, false);
    final_pdas.push(split_pda_2_0);
    final_pdas.push(split_pda_3_0);
    
    framework.execute_batch_hop(&transfer_pda, 1, &final_pdas).await?;
    
    // 7. Finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    Ok(())
}

#[tokio::test]
async fn test_batch_hop_error_conditions() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 50_000_000; // 0.05 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. PDAs vorbereiten
    let mut pdas = Vec::new();
    let (split_pda_0_0, _) = framework.derive_split_pda(&seed, 0, 0, false);
    pdas.push(split_pda_0_0);
    
    // 3. Falschen Batch-Index versuchen
    let result = framework.execute_batch_hop(&transfer_pda, 1, &pdas).await;
    assert!(result.is_err(), "Ungültiger Batch-Index sollte fehlschlagen");
    
    // 4. Korrekten Batch ausführen
    framework.execute_batch_hop(&transfer_pda, 0, &pdas).await?;
    
    // 5. Gleichen Batch-Index erneut versuchen
    let result = framework.execute_batch_hop(&transfer_pda, 0, &pdas).await;
    assert!(result.is_err(), "Wiederholter Batch-Index sollte fehlschlagen");
    
    // 6. Leere PDA-Liste testen
    let empty_pdas: Vec<Pubkey> = Vec::new();
    let result = framework.execute_batch_hop(&transfer_pda, 1, &empty_pdas).await;
    assert!(result.is_err(), "Leere PDA-Liste sollte fehlschlagen");
    
    // 7. Restlichen Hops ausführen
    let mut final_pdas = Vec::new();
    let (split_pda_1_0, _) = framework.derive_split_pda(&seed, 1, 0, false);
    let (split_pda_2_0, _) = framework.derive_split_pda(&seed, 2, 0, false);
    let (split_pda_3_0, _) = framework.derive_split_pda(&seed, 3, 0, false);
    final_pdas.push(split_pda_1_0);
    final_pdas.push(split_pda_2_0);
    final_pdas.push(split_pda_3_0);
    
    framework.execute_batch_hop(&transfer_pda, 1, &final_pdas).await?;
    
    // 8. Finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 9. Versuch nach Finalisierung
    let result = framework.execute_batch_hop(&transfer_pda, 2, &pdas).await;
    assert!(result.is_err(), "Batch-Hop nach Finalisierung sollte fehlschlagen");
    
    Ok(())
}

#[tokio::test]
async fn test_batch_hop_maximum_capacity() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 1_000_000_000; // 1 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. PDAs für alle 4 Hops vorbereiten
    let mut pdas = Vec::new();
    for hop_index in 0..4 {
        let (split_pda, _) = framework.derive_split_pda(&seed, hop_index, 0, false);
        pdas.push(split_pda);
    }
    
    // 3. Versuchen, alle 4 Hops in einem Batch auszuführen
    framework.execute_batch_hop(&transfer_pda, 0, &pdas).await?;
    
    // 4. Verify dass alle Hops ausgeführt wurden
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.current_hop, 4, "Alle 4 Hops sollten ausgeführt sein");
    assert_eq!(transfer_state.batch_count, 1, "Batch-Count sollte 1 sein");
    
    // 5. Alle PDAs überprüfen
    for pda in &pdas {
        let account = framework.context.banks_client
            .get_account(*pda)
            .await?;
        
        assert!(account.is_some(), "PDA sollte existieren");
        assert!(account.unwrap().lamports > 0, "PDA sollte Lamports haben");
    }
    
    // 6. Finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 7. Verify Finalisierung
    let finalized_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let finalized_state = TransferState::try_deserialize(
        &mut finalized_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(finalized_state.completed, true, "Transfer sollte abgeschlossen sein");
    
    Ok(())
}
