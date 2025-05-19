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
async fn test_single_hop_execution() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 10_000_000; // 0.01 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // Verify Transfer wurde korrekt initialisiert
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.amount, amount);
    assert_eq!(transfer_state.current_hop, 0);
    assert_eq!(transfer_state.owner, framework.user.pubkey());
    assert_eq!(transfer_state.completed, false);
    
    // 2. Ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // Verify Hop wurde korrekt ausgeführt
    let updated_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let updated_state = TransferState::try_deserialize(
        &mut updated_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(updated_state.current_hop, 1);
    assert_eq!(updated_state.completed, false);
    
    // 3. Verifizieren, dass das PDA erstellt wurde
    let (split_pda, _) = framework.derive_split_pda(&seed, 0, 0, false);
    let split_account = framework.context.banks_client
        .get_account(split_pda)
        .await?;
    
    assert!(split_account.is_some(), "Split-PDA sollte existieren");
    
    // 4. Lamport-Transfer verifizieren
    let split_account = split_account.unwrap();
    assert!(split_account.lamports > 0, "Split-PDA sollte Lamports haben");
    
    Ok(())
}

#[tokio::test]
async fn test_complete_hop_sequence() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 100_000_000; // 0.1 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Alle 4 Hops nacheinander ausführen
    for hop_index in 0..4 {
        framework.execute_hop(&transfer_pda, &seed, hop_index, 0, false).await?;
        
        // Verify Hop wurde korrekt ausgeführt
        let transfer_account = framework.context.banks_client
            .get_account(transfer_pda)
            .await?
            .expect("Transfer-Account sollte existieren");
        
        let transfer_state = TransferState::try_deserialize(
            &mut transfer_account.data.as_ref()
        ).expect("Transfer-State sollte deserialisierbar sein");
        
        assert_eq!(transfer_state.current_hop, hop_index + 1);
        
        // Verify Split-PDA wurde erstellt
        let (split_pda, _) = framework.derive_split_pda(&seed, hop_index, 0, false);
        let split_account = framework.context.banks_client
            .get_account(split_pda)
            .await?
            .expect("Split-PDA sollte existieren");
        
        assert!(split_account.lamports > 0, "Split-PDA sollte Lamports haben");
    }
    
    // 3. Verify dass alle Hops abgeschlossen wurden
    let final_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let final_state = TransferState::try_deserialize(
        &mut final_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(final_state.current_hop, 4);
    assert_eq!(final_state.completed, false); // Noch nicht finalisiert
    
    // 4. Transfer finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 5. Verify dass Transfer abgeschlossen ist
    let finalized_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let finalized_state = TransferState::try_deserialize(
        &mut finalized_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(finalized_state.completed, true);
    assert_eq!(finalized_state.recipient, recipient.pubkey());
    
    Ok(())
}

#[tokio::test]
async fn test_hop_error_conditions() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 5_000_000; // 0.005 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Versuch, einen ungültigen Hop-Index auszuführen (nicht sequentiell)
    let result = framework.execute_hop(&transfer_pda, &seed, 1, 0, false).await;
    assert!(result.is_err(), "Falscher Hop-Index sollte fehlschlagen");
    
    // 3. Korrekten ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 4. Versuch, den gleichen Hop erneut auszuführen
    let result = framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await;
    assert!(result.is_err(), "Wiederholter Hop sollte fehlschlagen");
    
    // 5. Zu großen Hop-Index versuchen
    let result = framework.execute_hop(&transfer_pda, &seed, 10, 0, false).await;
    assert!(result.is_err(), "Zu großer Hop-Index sollte fehlschlagen");
    
    // 6. Transfer abschließen
    framework.execute_hop(&transfer_pda, &seed, 1, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 2, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 3, 0, false).await?;
    
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 7. Versuch, nach Finalisierung weiteren Hop auszuführen
    let result = framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await;
    assert!(result.is_err(), "Hop nach Finalisierung sollte fehlschlagen");
    
    Ok(())
}

#[tokio::test]
async fn test_fake_split_handling() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 50_000_000; // 0.05 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Fake-Split für eine PDA ausführen
    let fake_result = framework.execute_hop(&transfer_pda, &seed, 0, 0, true).await;
    assert!(fake_result.is_ok(), "Fake-Split sollte funktionieren");
    
    // 3. Status nach Fake-Split prüfen
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.current_hop, 1);
    
    // 4. Fake-Split enthüllen
    framework.reveal_fake(&transfer_pda, 0, 0).await?;
    
    // 5. Rest der Hops ausführen
    framework.execute_hop(&transfer_pda, &seed, 1, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 2, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 3, 0, false).await?;
    
    // 6. Transfer finalisieren
    let recipient = Keypair::new();
    let finalize_result = framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await;
    assert!(finalize_result.is_ok(), "Finalisierung sollte trotz Fake-Split funktionieren");
    
    Ok(())
}

#[tokio::test]
async fn test_refund_functionality() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 20_000_000; // 0.02 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 3. Refund durchführen
    let initial_balance = framework.context.banks_client
        .get_balance(framework.user.pubkey())
        .await?;
    
    framework.refund_transfer(&transfer_pda).await?;
    
    // 4. Prüfen, ob Refund erfolgreich war
    let refunded_transfer = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let refunded_state = TransferState::try_deserialize(
        &mut refunded_transfer.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(refunded_state.refund_triggered, true);
    
    // 5. Prüfen, ob Lamports zurückgegeben wurden
    let final_balance = framework.context.banks_client
        .get_balance(framework.user.pubkey())
        .await?;
    
    assert!(final_balance > initial_balance, "Refund sollte Balance erhöhen");
    
    // 6. Prüfen, ob kein weiterer Hop möglich ist
    let hop_result = framework.execute_hop(&transfer_pda, &seed, 1, 0, false).await;
    assert!(hop_result.is_err(), "Hop nach Refund sollte fehlschlagen");
    
    Ok(())
}
