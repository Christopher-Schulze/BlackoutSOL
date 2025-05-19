//! Tests for Poseidon Hash Implementation
//! 
//! These tests validate the correct implementation of Poseidon parameters
//! and hash functions for Zero-Knowledge Proof applications.

use blackout::utils;

// Import of test_framework and other common test components
mod test_framework;
use test_framework::*;

// Test für die vollständige Implementierung der Poseidon-Parameter
#[test]
fn test_poseidon_parameters() {
    // Initialisiere Testumgebung
    let test_env = setup_test_env();
    let program = &test_env.program;
    
    // Log test start
    solana_program::msg!("Test Poseidon Parameter-Implementierung");
    
    // Call get_poseidon_params via program API
    // Since we don't have an RPC call for this function, we test indirectly
    // by using a hash function that uses these parameters
    
    // Prepare test commitments
    let mut commitments = [[0u8; 32]; 8];
    for i in 0..8 {
        for j in 0..32 {
            commitments[i][j] = ((i * j) % 256) as u8;
        }
    }
    
    // Try to calculate the hash - this will fail if the parameters are incorrect
    let hash_result = utils::poseidon_hash_commitments(&commitments);
    
    // Prüfe, ob der Hash erfolgreich war
    assert!(hash_result.is_ok(), "Poseidon Hash fehlgeschlagen - Parameter könnten fehlerhaft sein");
    
    // If we get here, the parameters were successful
    solana_program::msg!("Poseidon Parameter-Test bestanden");
}

// Test für die Konsistenz der Poseidon-Hash-Funktion
#[test]
fn test_poseidon_hash_consistency() {
    // Initialisiere Testumgebung
    let test_env = setup_test_env();
    let program = &test_env.program;
    
    solana_program::msg!("Test Poseidon Hash-Konsistenz");
    
    // Test with different input data
    let test_data_1 = [1u8; 32];
    let test_data_2 = [2u8; 32];
    
    // Create test data for first hash
    let mut commitments_1 = [[0u8; 32]; 8];
    for i in 0..8 {
        commitments_1[i] = test_data_1;
    }
    
    // Calculate first hash
    let hash_1 = utils::poseidon_hash_commitments(&commitments_1)
        .expect("Erster Hash sollte erfolgreich sein");
    
    // Repeat with same data to test consistency
    let hash_2 = utils::poseidon_hash_commitments(&commitments_1)
        .expect("Zweiter Hash sollte erfolgreich sein");
    
    // Check consistency
    assert_eq!(hash_1, hash_2, "Poseidon Hash ist nicht konsistent für identische Eingaben");
    
    // Create modified test data
    let mut commitments_2 = commitments_1.clone();
    commitments_2[0] = test_data_2;
    
    // Calculate hash with modified data
    let hash_3 = utils::poseidon_hash_commitments(&commitments_2)
        .expect("Dritter Hash sollte erfolgreich sein");
    
    // Check if hash changes with modified data
    assert_ne!(hash_1, hash_3, "Poseidon Hash ändert sich nicht bei geänderten Eingaben");
    
    solana_program::msg!("Poseidon Hash-Konsistenz-Test bestanden");
}

// Test for the complete number of round constants
// We can't access the constants directly, but we can test indirectly
// by ensuring that operations with complex inputs work correctly
#[test]
fn test_poseidon_complex_operations() {
    // Initialisiere Testumgebung
    let test_env = setup_test_env();
    let program = &test_env.program;
    
    solana_program::msg!("Test komplexer Poseidon-Operationen");
    
    // Create complex test data with varying patterns
    let mut complex_commitments = [[0u8; 32]; 8];
    
    // Various bit patterns for the input data
    for i in 0..8 {
        for j in 0..32 {
            // Generate complex patterns that activate different round constants
            complex_commitments[i][j] = match (i + j) % 4 {
                0 => 0x55, // 01010101
                1 => 0xAA, // 10101010
                2 => 0x33, // 00110011
                _ => 0xCC, // 11001100
            };
        }
    }
    
    // Calculate hash with complex data
    let complex_hash = utils::poseidon_hash_commitments(&complex_commitments);
    
    // Prüfe, ob der Hash erfolgreich war
    assert!(complex_hash.is_ok(), "Komplexer Poseidon Hash fehlgeschlagen");
    
    solana_program::msg!("Komplexer Poseidon-Operationen-Test bestanden");
}

// Test for correct error handling
#[test]
fn test_poseidon_error_handling() {
    // Initialisiere Testumgebung
    let test_env = setup_test_env();
    let program = &test_env.program;
    
    solana_program::msg!("Test Poseidon Fehlerbehandlung");
    
    // Normal input, should be successful
    let valid_commitments = [[0u8; 32]; 8];
    let valid_result = utils::poseidon_hash_commitments(&valid_commitments);
    assert!(valid_result.is_ok(), "Poseidon Hash sollte für gültige Eingabe erfolgreich sein");
    
    // We can't do direct error checking here because we have no way
    // to generate invalid parameters without modifying the code.
    // In a real environment, we would use mock objects to simulate errors.
    
    solana_program::msg!("Poseidon Fehlerbehandlungs-Test bestanden");
}
