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
async fn test_initialize_basic() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer mit verschiedenen Parametern initialisieren
    let amount = 100_000_000; // 0.1 SOL
    let reserve_percent = 10;
    let seed = BlackoutTestFramework::generate_transfer_seed();
    
    let (transfer_pda, bump) = framework.find_transfer_pda(&framework.user.pubkey());
    
    let ix = initialize_instruction(
        initialize::InitializeParams {
            amount,
            reserve_percent,
            seed,
        },
        initialize::InitializeAccounts {
            authority: framework.user.pubkey(),
            transfer_state: transfer_pda,
            system_program: system_program::id(),
        },
    );
    
    framework.execute_transaction(&[ix], &[&framework.user]).await?;
    
    // 2. Verify Transfer wurde korrekt initialisiert
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    // 3. Deserialize und prüfe Felder
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(transfer_state.owner, framework.user.pubkey());
    assert_eq!(transfer_state.amount, amount);
    assert_eq!(transfer_state.current_hop, 0);
    assert_eq!(transfer_state.seed, seed);
    assert_eq!(transfer_state.completed, false);
    assert_eq!(transfer_state.bump, bump);
    assert_eq!(transfer_state.config.reserve_percent, reserve_percent);
    assert_eq!(transfer_state.refund_triggered, false);
    
    // 4. Verifiziere Lamport-Übertragung
    assert!(transfer_account.lamports >= amount, 
           "Transfer-Account sollte mindestens {} Lamports haben", amount);
    
    Ok(())
}

#[tokio::test]
async fn test_initialize_edge_cases() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Minimaler Betrag
    let min_amount = 1_000; // 0.000001 SOL
    let (min_pda, min_seed) = framework.initialize_transfer(min_amount, 5).await?;
    
    let min_account = framework.context.banks_client
        .get_account(min_pda)
        .await?
        .expect("Min-Transfer-Account sollte existieren");
    
    let min_state = TransferState::try_deserialize(
        &mut min_account.data.as_ref()
    ).expect("Min-Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(min_state.amount, min_amount);
    
    // 2. Maximaler Reserveprozentsatz
    let max_reserve = 50; // 50%
    let max_amount = 10_000_000;
    
    let seed = BlackoutTestFramework::generate_transfer_seed();
    let (max_pda, _) = framework.find_transfer_pda(&framework.user.pubkey());
    
    let ix = initialize_instruction(
        initialize::InitializeParams {
            amount: max_amount,
            reserve_percent: max_reserve,
            seed,
        },
        initialize::InitializeAccounts {
            authority: framework.user.pubkey(),
            transfer_state: max_pda,
            system_program: system_program::id(),
        },
    );
    
    // Da wir bereits einen Transfer initialisiert haben, sollte
    // dieser fehlschlagen, da die PDA bereits existiert
    let result = framework.execute_transaction(&[ix], &[&framework.user]).await;
    assert!(result.is_err(), "Zweiter Transfer mit gleicher PDA sollte fehlschlagen");
    
    // 3. Zu hoher Reserveprozentsatz (> 50)
    let invalid_reserve = 51;
    let invalid_seed = BlackoutTestFramework::generate_transfer_seed();
    
    // Neuen User erstellen für separaten Test
    let new_user = Keypair::new();
    let transfer_amount = 10_000;
    
    // Funds für den neuen User bereitstellen
    let fund_ix = system_instruction::transfer(
        &framework.context.payer.pubkey(),
        &new_user.pubkey(),
        100_000_000,
    );
    
    framework.execute_transaction(&[fund_ix], &[&framework.context.payer]).await?;
    
    // Invalid Reserve Prozent Transfer
    let (invalid_pda, _) = framework.find_transfer_pda(&new_user.pubkey());
    
    let invalid_ix = initialize_instruction(
        initialize::InitializeParams {
            amount: transfer_amount,
            reserve_percent: invalid_reserve,
            seed: invalid_seed,
        },
        initialize::InitializeAccounts {
            authority: new_user.pubkey(),
            transfer_state: invalid_pda,
            system_program: system_program::id(),
        },
    );
    
    let result = framework.execute_transaction(&[invalid_ix], &[&new_user]).await;
    assert!(result.is_err(), "Transfer mit zu hohem Reserve-Prozent sollte fehlschlagen");
    
    Ok(())
}

#[tokio::test]
async fn test_finalize_after_hop() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 50_000_000; // 0.05 SOL
    let reserve_percent = 5;
    let (transfer_pda, seed) = framework.initialize_transfer(amount, reserve_percent).await?;
    
    // 2. Einen Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 3. Versuchen zu finalisieren (sollte fehlschlagen, da nicht alle Hops abgeschlossen)
    let recipient = Keypair::new();
    let result = framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await;
    assert!(result.is_err(), "Finalisierung vor Abschluss aller Hops sollte fehlschlagen");
    
    // 4. Restliche Hops ausführen
    framework.execute_hop(&transfer_pda, &seed, 1, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 2, 0, false).await?;
    framework.execute_hop(&transfer_pda, &seed, 3, 0, false).await?;
    
    // 5. Jetzt finalisieren
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 6. Verify Finalisierung
    let finalized_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let finalized_state = TransferState::try_deserialize(
        &mut finalized_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(finalized_state.completed, true);
    assert_eq!(finalized_state.recipient, recipient.pubkey());
    
    // 7. Verify Empfänger hat Lamports erhalten
    let recipient_balance = framework.context.banks_client
        .get_balance(recipient.pubkey())
        .await?;
    
    // Reserve ist (amount * reserve_percent / 100)
    let expected_reserve = amount * (reserve_percent as u64) / 100;
    let expected_transfer = amount - expected_reserve;
    
    assert!(recipient_balance >= expected_transfer, 
           "Empfänger sollte mindestens {} Lamports erhalten haben", expected_transfer);
    
    // 8. Verify dass Reserve im Transfer-State bleibt
    let transfer_balance = framework.context.banks_client
        .get_balance(transfer_pda)
        .await?;
    
    assert!(transfer_balance >= expected_reserve, 
           "Transfer-State sollte Reserve von {} Lamports behalten", expected_reserve);
    
    // 9. Versuchen, bereits finalisierten Transfer erneut zu finalisieren
    let new_recipient = Keypair::new();
    let result = framework.finalize_transfer(&transfer_pda, &new_recipient.pubkey()).await;
    assert!(result.is_err(), "Erneute Finalisierung sollte fehlschlagen");
    
    Ok(())
}

#[tokio::test]
async fn test_finalize_with_zero_reserve() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer mit 0% Reserve initialisieren
    let amount = 25_000_000; // 0.025 SOL
    let reserve_percent = 0;
    let (transfer_pda, seed) = framework.initialize_transfer(amount, reserve_percent).await?;
    
    // 2. Alle Hops ausführen
    for hop_index in 0..4 {
        framework.execute_hop(&transfer_pda, &seed, hop_index, 0, false).await?;
    }
    
    // 3. Finalisieren
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 4. Verify Empfänger hat vollen Betrag erhalten
    let recipient_balance = framework.context.banks_client
        .get_balance(recipient.pubkey())
        .await?;
    
    // Bei 0% Reserve sollte der volle Betrag übertragen werden
    assert!(recipient_balance >= amount, 
           "Empfänger sollte vollen Betrag erhalten haben");
    
    Ok(())
}

#[tokio::test]
async fn test_update_config() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 30_000_000; // 0.03 SOL
    let initial_reserve = 5;
    let (transfer_pda, seed) = framework.initialize_transfer(amount, initial_reserve).await?;
    
    // 2. Config-Update ausführen
    let new_reserve = 10;
    let new_fee_multiplier = 200;
    
    let update_ix = config_update_instruction(
        config_update::ConfigUpdateParams {
            reserve_percent: Some(new_reserve),
            fee_multiplier: Some(new_fee_multiplier),
        },
        config_update::ConfigUpdateAccounts {
            authority: framework.user.pubkey(),
            transfer_state: transfer_pda,
        },
    );
    
    framework.execute_transaction(&[update_ix], &[&framework.user]).await?;
    
    // 3. Verify Config wurde aktualisiert
    let updated_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let updated_state = TransferState::try_deserialize(
        &mut updated_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(updated_state.config.reserve_percent, new_reserve);
    assert_eq!(updated_state.config.fee_multiplier, new_fee_multiplier);
    
    // 4. Verify andere Config-Parameter unverändert blieben
    assert_eq!(updated_state.config.num_hops, 4);
    assert_eq!(updated_state.config.real_splits_per_hop, 4);
    assert_eq!(updated_state.config.fake_splits_per_hop, 44);
    
    // 5. Alle Hops ausführen
    for hop_index in 0..4 {
        framework.execute_hop(&transfer_pda, &seed, hop_index, 0, false).await?;
    }
    
    // 6. Finalisieren und neue Reserve prüfen
    let recipient = Keypair::new();
    framework.finalize_transfer(&transfer_pda, &recipient.pubkey()).await?;
    
    // 7. Verify Empfänger hat Betrag - neue Reserve erhalten
    let recipient_balance = framework.context.banks_client
        .get_balance(recipient.pubkey())
        .await?;
    
    let expected_reserve = amount * (new_reserve as u64) / 100;
    let expected_transfer = amount - expected_reserve;
    
    assert!(recipient_balance >= expected_transfer, 
           "Empfänger sollte Betrag abzüglich neuer Reserve erhalten haben");
    
    Ok(())
}
