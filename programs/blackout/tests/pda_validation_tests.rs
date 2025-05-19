/*
 * Test Suite für die PDA-Validierungslogik
 * 
 * Diese Tests validieren die korrekte Funktionsweise der verbesserten
 * PDA-Validierungslogik im BlackoutSOL-Projekt.
 */

use anchor_lang::prelude::*;
use blackout::utils::{
    verify_pda_derivation,
    verify_bloom_filter,
    derive_stealth_pda,
    generate_bloom_filter,
};
use blackout::state::config::BlackoutConfig;
use solana_program::pubkey::Pubkey;
use solana_program_test::*;
use solana_sdk::account::AccountSharedData;
use solana_sdk::account_info::AccountInfo;
use solana_sdk::signer::keypair::Keypair;
use std::cell::RefCell;
use std::rc::Rc;

/// Test für die korrekte Validierung von realen PDA-Pfaden
#[tokio::test]
async fn test_real_pda_validation() {
    // 1. Setup Test-Umgebung
    let program_id = Pubkey::new_unique();
    let seed = [42u8; 32];
    let hop_index = 2;
    let split_index = 1;
    
    // 2. PDA ableiten
    let (expected_pda, bump) = derive_stealth_pda(
        &program_id,
        &seed,
        hop_index,
        split_index,
        false // Nicht ein Fake-Split
    );
    
    // 3. Mock AccountInfo erstellen
    let key = expected_pda;
    let mut lamports = 10000;
    let mut data = vec![0; 32];
    let owner = program_id;
    
    let account_info = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // 4. PDA-Validierung durchführen
    let validation_result = verify_pda_derivation(
        &program_id,
        &seed,
        hop_index,
        split_index,
        &account_info
    );
    
    // 5. Ergebnis überprüfen
    assert!(validation_result.is_ok(), "PDA-Validierung sollte erfolgreich sein");
    
    // 6. Validieren, dass die zurückgegebene PDA und der Bump korrekt sind
    let (returned_pda, returned_bump) = validation_result.unwrap();
    assert_eq!(expected_pda, returned_pda, "Zurückgegebene PDA sollte mit der erwarteten übereinstimmen");
    assert_eq!(bump, returned_bump, "Zurückgegebener Bump sollte mit dem erwarteten übereinstimmen");
}

/// Test für die Erkennung einer ungültigen PDA
#[tokio::test]
async fn test_invalid_pda_validation() {
    // 1. Setup Test-Umgebung
    let program_id = Pubkey::new_unique();
    let seed = [42u8; 32];
    let hop_index = 2;
    let split_index = 1;
    
    // 2. Falsche PDA erstellen (andere PDA als erwartet)
    let incorrect_key = Pubkey::new_unique(); // Komplett andere PDA
    
    // 3. Mock AccountInfo erstellen
    let mut lamports = 10000;
    let mut data = vec![0; 32];
    let owner = program_id;
    
    let account_info = AccountInfo::new(
        &incorrect_key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // 4. PDA-Validierung durchführen
    let validation_result = verify_pda_derivation(
        &program_id,
        &seed,
        hop_index,
        split_index,
        &account_info
    );
    
    // 5. Ergebnis überprüfen
    assert!(validation_result.is_err(), "PDA-Validierung sollte fehlschlagen bei falscher PDA");
}

/// Test für die Validierung von Fake-Splits über den Bloom-Filter
#[tokio::test]
async fn test_fake_split_validation() {
    // 1. Setup Test-Umgebung
    let program_id = Pubkey::new_unique();
    let seed = [42u8; 32];
    let hop_index = 2;
    let split_index = 10; // Höherer Index für Fake-Split
    let challenge = [0u8; 32];
    
    // 2. Konfiguration erstellen
    let config = BlackoutConfig {
        real_splits: 4,
        fake_splits: 44,
        num_hops: 4,
        reserve_percent: 40,
        fee_multiplier: 200, // 2%
        cu_budget_per_hop: 220_000,
    };
    
    // 3. Bloom-Filter für die Fake-Splits generieren
    let bloom_filter = generate_bloom_filter(&config, &challenge);
    
    // 4. PDA ableiten (als Fake-Split)
    let (fake_pda, bump) = derive_stealth_pda(
        &program_id,
        &seed,
        hop_index,
        split_index,
        true // Ist ein Fake-Split
    );
    
    // 5. Mock AccountInfo erstellen
    let key = fake_pda;
    let mut lamports = 10000;
    let mut data = vec![0; 32];
    let owner = program_id;
    
    let account_info = AccountInfo::new(
        &key,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // 6. PDA-Validierung durchführen (sollte fehlschlagen, da wir eine Fake-PDA validieren)
    let validation_result = verify_pda_derivation(
        &program_id,
        &seed,
        hop_index,
        split_index,
        &account_info
    );
    
    // 7. Bloom-Filter-Validierung durchführen
    let bloom_validation = verify_bloom_filter(
        &bloom_filter,
        hop_index,
        split_index
    );
    
    // 8. Ergebnisse überprüfen
    assert!(validation_result.is_err(), "Direkte PDA-Validierung sollte für Fake-Splits fehlschlagen");
    assert!(bloom_validation.is_ok(), "Bloom-Filter-Validierung sollte für Fake-Splits erfolgreich sein");
    assert!(bloom_validation.unwrap(), "Der Split sollte im Bloom-Filter als Fake-Split markiert sein");
}

/// Integrationstest für die Dual-Path-Validierungsstrategie
#[tokio::test]
async fn test_dual_path_validation_strategy() {
    // 1. Setup Test-Umgebung
    let program_id = Pubkey::new_unique();
    let seed = [42u8; 32];
    let hop_index = 2;
    let challenge = [0u8; 32];
    
    // 2. Konfiguration erstellen
    let config = BlackoutConfig {
        real_splits: 4,
        fake_splits: 44,
        num_hops: 4,
        reserve_percent: 40,
        fee_multiplier: 200, // 2%
        cu_budget_per_hop: 220_000,
    };
    
    // 3. Bloom-Filter für die Fake-Splits generieren
    let bloom_filter = generate_bloom_filter(&config, &challenge);
    
    // Test für einen realen Split
    {
        let split_index = 1; // Realer Split
        
        // PDA ableiten
        let (real_pda, _) = derive_stealth_pda(
            &program_id,
            &seed,
            hop_index,
            split_index,
            false // Nicht ein Fake-Split
        );
        
        // Mock AccountInfo erstellen
        let key = real_pda;
        let mut lamports = 10000;
        let mut data = vec![0; 32];
        let owner = program_id;
        
        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        
        // Dual-Path-Validierung implementieren
        let validation_result = verify_pda_derivation(
            &program_id,
            &seed,
            hop_index,
            split_index,
            &account_info
        );
        
        let is_valid = if validation_result.is_ok() {
            true
        } else {
            // Fallback zur Bloom-Filter-Validierung
            verify_bloom_filter(
                &bloom_filter,
                hop_index,
                split_index
            ).unwrap_or(false)
        };
        
        assert!(is_valid, "Realer Split sollte durch direkte PDA-Validierung validiert werden");
    }
    
    // Test für einen Fake-Split
    {
        let split_index = 10; // Fake-Split (außerhalb des realen Bereichs)
        
        // PDA ableiten
        let (fake_pda, _) = derive_stealth_pda(
            &program_id,
            &seed,
            hop_index,
            split_index,
            true // Ist ein Fake-Split
        );
        
        // Mock AccountInfo erstellen
        let key = fake_pda;
        let mut lamports = 10000;
        let mut data = vec![0; 32];
        let owner = program_id;
        
        let account_info = AccountInfo::new(
            &key,
            false,
            false,
            &mut lamports,
            &mut data,
            &owner,
            false,
            0,
        );
        
        // Dual-Path-Validierung implementieren
        let validation_result = verify_pda_derivation(
            &program_id,
            &seed,
            hop_index,
            split_index,
            &account_info
        );
        
        let is_valid = if validation_result.is_ok() {
            true
        } else {
            // Fallback zur Bloom-Filter-Validierung
            verify_bloom_filter(
                &bloom_filter,
                hop_index,
                split_index
            ).unwrap_or(false)
        };
        
        assert!(is_valid, "Fake-Split sollte durch Bloom-Filter-Validierung validiert werden");
    }
}
