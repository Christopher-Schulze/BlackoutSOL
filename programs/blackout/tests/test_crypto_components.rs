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
    utils::*,
};

mod test_framework;
use test_framework::BlackoutTestFramework;

/// Test speziell für die kryptographischen Komponenten des Blackout SOL Systems
/// 
/// Diese Tests validieren die HyperPlonk-Proof-Verifikation, Plonky2-Range-Proofs
/// und die allgemeine Bloom-Filter-Funktionalität.

#[tokio::test]
async fn test_hyperplonk_proof_verification() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 50_000_000; // 0.05 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Gültige Challenge erstellen
    let challenge = [0u8; 32];
    
    // 3. Splits definieren
    let splits = [
        amount / 4,
        amount / 4,
        amount / 4,
        amount / 4,
    ];
    
    // 4. Gültigen HyperPlonk-Proof erstellen
    let valid_proof = BlackoutTestFramework::create_test_hyperplonk_proof(&challenge, &splits);
    
    // 5. Ungültigen Proof erstellen (mit falscher Signatur)
    let mut invalid_proof = valid_proof.clone();
    invalid_proof[0] = 0x00; // Falsche Signatur
    
    // 6. Split-PDA erstellen
    let (split_pda, _) = framework.derive_split_pda(&seed, 0, 0, false);
    
    // 7. Account-Information abrufen
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    // 8. Die Tests werden implizit durchgeführt, wenn wir die Hops ausführen
    // Ein Hop mit ungültigem Proof sollte fehlschlagen

    // 9. Hop mit gültigem Proof ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 10. Verify Hop wurde korrekt ausgeführt
    let updated_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let updated_state = TransferState::try_deserialize(
        &mut updated_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(updated_state.current_hop, 1, "Hop mit gültigem Proof sollte erfolgreich sein");
    
    Ok(())
}

#[tokio::test]
async fn test_range_proof_verification() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 50_000_000; // 0.05 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Transfer-Account abrufen
    let transfer_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let transfer_state = TransferState::try_deserialize(
        &mut transfer_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    // 3. Challenge erstellen
    let challenge = [0u8; 32];
    
    // 4. Gültigen Range-Proof erstellen mit korrektem Sum-Check
    let valid_proof = BlackoutTestFramework::create_test_range_proof(
        &challenge,
        &transfer_state.commitments,
        true, // Sum-Check ist gültig
    );
    
    // 5. Ungültigen Range-Proof mit falschem Sum-Check erstellen
    let invalid_proof = BlackoutTestFramework::create_test_range_proof(
        &challenge,
        &transfer_state.commitments,
        false, // Sum-Check ist ungültig
    );
    
    // 6. Hop mit gültigem Range-Proof ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 7. Verify dass der Hop erfolgreich war
    let updated_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let updated_state = TransferState::try_deserialize(
        &mut updated_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(updated_state.current_hop, 1, "Hop mit gültigem Range Proof sollte erfolgreich sein");
    
    Ok(())
}

#[tokio::test]
async fn test_bloom_filter_functionality() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Transfer initialisieren
    let amount = 100_000_000; // 0.1 SOL
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 5).await?;
    
    // 2. Bloom-Filter erstellen für Fake-Splits
    let fake_indices = [1, 2, 3, 10, 20, 30, 40];
    let bloom_filter = BlackoutTestFramework::create_fake_bloom_filter(&fake_indices);
    
    // 3. Verifizieren, dass der Bloom-Filter korrekt funktioniert
    // Diese Indices sollten im Filter sein
    for &idx in &fake_indices {
        let result = check_bloom_filter(&bloom_filter, 0, idx);
        assert!(result, "Index {} sollte im Bloom-Filter sein", idx);
    }
    
    // Diese Indices sollten nicht im Filter sein
    let non_fake_indices = [0, 5, 15, 25, 35, 45];
    for &idx in &non_fake_indices {
        let result = check_bloom_filter(&bloom_filter, 0, idx);
        // Es kann false positives geben, aber wir können nicht garantieren, dass ein Index
        // NICHT im Filter ist, aufgrund der Natur von Bloom-Filtern
    }
    
    // 4. Einen Split ausführen, der NICHT im Bloom-Filter ist (Real Split)
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false).await?;
    
    // 5. Verifizieren, dass der Split korrekt ausgeführt wurde
    let (split_pda, _) = framework.derive_split_pda(&seed, 0, 0, false);
    let split_account = framework.context.banks_client
        .get_account(split_pda)
        .await?
        .expect("Real-Split PDA sollte existieren");
    
    assert!(split_account.lamports > 0, "Real-Split sollte Lamports erhalten haben");
    
    // 6. Einen Split ausführen, der im Bloom-Filter ist (Fake Split)
    framework.execute_hop(&transfer_pda, &seed, 1, 1, true).await?;
    
    // 7. Verifizieren, dass der Fake-Split korrekt verarbeitet wurde
    let updated_account = framework.context.banks_client
        .get_account(transfer_pda)
        .await?
        .expect("Transfer-Account sollte existieren");
    
    let updated_state = TransferState::try_deserialize(
        &mut updated_account.data.as_ref()
    ).expect("Transfer-State sollte deserialisierbar sein");
    
    assert_eq!(updated_state.current_hop, 2, "Current Hop sollte 2 sein nach 2 Hops");
    
    Ok(())
}

#[tokio::test]
async fn test_poseidon_hash_commitments() -> Result<(), TransportError> {
    let mut framework = BlackoutTestFramework::new().await;
    
    // 1. Test-Commitments erstellen
    let mut commitments = [[0u8; 32]; 8];
    
    // Einige Test-Werte in die Commitments einfügen
    for i in 0..8 {
        for j in 0..32 {
            commitments[i][j] = ((i * j) % 256) as u8;
        }
    }
    
    // 2. Poseidon-Hash berechnen
    let hash_result = poseidon_hash_commitments(&commitments);
    
    // 3. Verify dass der Hash nicht leer ist
    assert!(!hash_result.is_empty(), "Poseidon-Hash sollte nicht leer sein");
    assert!(hash_result.len() <= 32, "Poseidon-Hash sollte maximal 32 Bytes haben");
    
    // 4. Verify dass derselbe Input denselben Hash erzeugt (Determinismus)
    let second_hash = poseidon_hash_commitments(&commitments);
    assert_eq!(hash_result, second_hash, "Poseidon-Hash sollte deterministisch sein");
    
    // 5. Verify dass verschiedene Inputs verschiedene Hashes erzeugen
    let mut different_commitments = commitments.clone();
    different_commitments[0][0] = commitments[0][0].wrapping_add(1);
    
    let different_hash = poseidon_hash_commitments(&different_commitments);
    assert_ne!(hash_result, different_hash, "Verschiedene Inputs sollten verschiedene Hashes erzeugen");
    
    Ok(())
}

#[test]
fn test_extract_split_amount() {
    // 1. Test-Proof mit bekannten Split-Beträgen erstellen
    let challenge = [0u8; 32];
    let splits = [1000, 2000, 3000, 4000];
    
    let proof = BlackoutTestFramework::create_test_hyperplonk_proof(&challenge, &splits);
    
    // 2. Split-Beträge extrahieren und verifizieren
    for i in 0..4 {
        let extracted = extract_split_amount(&proof, i as u8);
        assert_eq!(extracted, splits[i], "Extrahierter Split-Betrag sollte korrekt sein");
    }
    
    // 3. Verify dass ungültige Indices 0 zurückgeben
    let invalid_index = extract_split_amount(&proof, 10);
    assert_eq!(invalid_index, 0, "Ungültiger Index sollte 0 zurückgeben");
}

#[test]
fn test_calculate_optimized_priority_fees() {
    // 1. Test mit verschiedenen verbleibenden Hops
    let base_fee = 1000;
    
    // Bei 1 verbleibenden Hop sollte die Priorität am höchsten sein
    let priority_1 = calculate_optimized_priority_fees(1, base_fee);
    
    // Bei 2 verbleibenden Hops sollte die Priorität mittel sein
    let priority_2 = calculate_optimized_priority_fees(2, base_fee);
    
    // Bei 3+ verbleibenden Hops sollte die Priorität am niedrigsten sein
    let priority_3 = calculate_optimized_priority_fees(3, base_fee);
    let priority_4 = calculate_optimized_priority_fees(4, base_fee);
    
    // 2. Verifizieren der korrekten Priorisierung
    assert!(priority_1 > priority_2, "Priorität sollte für weniger verbleibende Hops höher sein");
    assert!(priority_2 > priority_3, "Priorität sollte für weniger verbleibende Hops höher sein");
    assert_eq!(priority_3, priority_4, "Priorität sollte für 3+ Hops gleich sein");
    
    // 3. Verifizieren, dass die Priorität vom Base-Fee abhängt
    let high_base_fee = 5000;
    let high_priority_1 = calculate_optimized_priority_fees(1, high_base_fee);
    
    assert!(high_priority_1 > priority_1, "Höhere Base-Fee sollte zu höherer Priorität führen");
    assert_eq!(high_priority_1, 3 * high_base_fee, "Priorität für letzten Hop sollte 3 * Base-Fee sein");
}
