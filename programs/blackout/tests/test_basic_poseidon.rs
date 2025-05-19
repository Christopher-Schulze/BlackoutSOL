//! Basic tests for the Poseidon hash implementation
//! 
//! These tests ensure that the fundamental Poseidon functionality
//! works correctly without relying on the more complex Solana program structures.

use solana_program::program_error::ProgramError;
use solana_program::pubkey::Pubkey;
use solana_poseidon::poseidon::{Poseidon, Parameters, Scalar};

// Simple constant values for testing
const TEST_ROUND_CONSTANTS: [[u8; 32]; 3] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [2, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

const TEST_MDS_MATRIX: [[u8; 32]; 9] = [
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
    [0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
];

#[test]
fn test_poseidon_initialization() {
    // Test 1: Basic initialization
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8, // Full rounds
        56, // Partial rounds
        3, // Width (t)
    )
    .unwrap();
    
    let poseidon = Poseidon::new_with_params(params.clone());
    assert!(poseidon.is_ok(), "Poseidon should be initialized successfully");
    
    // Test 2: Check parameter boundaries
    let negative_params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        0, // Invalid full rounds (too small)
        56,
        3,
    );
    assert!(negative_params.is_err(), "Invalid parameters should be rejected");
}

#[test]
fn test_poseidon_hash_single_input() {
    // Test with a single input
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        3,
    )
    .unwrap();
    
    let mut poseidon = Poseidon::new_with_params(params).unwrap();
    
    // Simple input
    let input = [1u8; 32];
    
    poseidon.hash_many_bytes(&[&input]).unwrap();
    let result1 = poseidon.get_hash().unwrap();
    
    // After reset with same input, same result should be produced
    poseidon.reset();
    poseidon.hash_many_bytes(&[&input]).unwrap();
    let result2 = poseidon.get_hash().unwrap();
    
    assert_eq!(result1, result2, "Hash-Ergebnisse sollten deterministisch sein");
}

#[test]
fn test_poseidon_hash_multiple_inputs() {
    // Test with multiple inputs
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        3,
    )
    .unwrap();
    
    let mut poseidon = Poseidon::new_with_params(params).unwrap();
    
    // Different inputs
    let input1 = [1u8; 32];
    let input2 = [2u8; 32];
    
    // First hash with both inputs
    poseidon.hash_many_bytes(&[&input1, &input2]).unwrap();
    let result1 = poseidon.get_hash().unwrap();
    
    // Zweiter Hash mit umgekehrter Reihenfolge - sollte anderes Ergebnis liefern
    poseidon.reset();
    poseidon.hash_many_bytes(&[&input2, &input1]).unwrap();
    let result2 = poseidon.get_hash().unwrap();
    
    assert_ne!(result1, result2, "Verschiedene Eingabereihenfolgen sollten verschiedene Hashes ergeben");
}

#[test]
fn test_poseidon_hash_consistency() {
    // Test für konsistente Hashes über mehrere Instanzen hinweg
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        3,
    )
    .unwrap();
    
    // Erste Poseidon-Instanz
    let mut poseidon1 = Poseidon::new_with_params(params.clone()).unwrap();
    
    // Zweite Poseidon-Instanz mit gleichen Parametern
    let mut poseidon2 = Poseidon::new_with_params(params).unwrap();
    
    // Gleiche Eingabe für beide
    let input = [5u8; 32];
    
    // Hash mit beiden Instanzen berechnen
    poseidon1.hash_many_bytes(&[&input]).unwrap();
    let result1 = poseidon1.get_hash().unwrap();
    
    poseidon2.hash_many_bytes(&[&input]).unwrap();
    let result2 = poseidon2.get_hash().unwrap();
    
    assert_eq!(result1, result2, "Verschiedene Instanzen mit gleichen Parametern und Inputs sollten gleiche Hashes erzeugen");
}

// Helper function to compare two byte arrays
fn array_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    for (x, y) in a.iter().zip(b.iter()) {
        if x != y {
            return false;
        }
    }
    true
}

#[test]
fn test_poseidon_byte_representation() {
    // Test für die korrekte Byte-Repräsentation der Hashes
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        3,
    )
    .unwrap();
    
    let mut poseidon = Poseidon::new_with_params(params).unwrap();
    
    // Test-Input
    let input = [42u8; 32];
    
    // Hash berechnen
    poseidon.hash_many_bytes(&[&input]).unwrap();
    let result = poseidon.get_hash().unwrap();
    
    // Konvertiere zu Bytes und zurück zu Scalar
    let bytes = result.to_bytes();
    let recovered = Scalar::from_bytes(&bytes).unwrap();
    
    assert_eq!(result, recovered, "Byte-Konvertierung sollte korrekt funktionieren");
}

#[test]
fn test_poseidon_error_handling() {
    // Test für korrektes Fehlerverhalten bei ungültigen Parametern
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        0, // Ungültige Breite
    );
    
    assert!(params.is_err(), "Ungültige Parameter sollten einen Fehler erzeugen");
}

#[test]
fn test_poseidon_reset_behavior() {
    // Test für korrektes Reset-Verhalten
    let params = Parameters::new(
        TEST_MDS_MATRIX.to_vec(),
        TEST_ROUND_CONSTANTS.to_vec(),
        8,
        56,
        3,
    )
    .unwrap();
    
    let mut poseidon = Poseidon::new_with_params(params).unwrap();
    
    // Erster Input und Hash
    let input1 = [10u8; 32];
    poseidon.hash_many_bytes(&[&input1]).unwrap();
    let result1 = poseidon.get_hash().unwrap();
    
    // Zweiter Input ohne Reset
    let input2 = [20u8; 32];
    poseidon.hash_many_bytes(&[&input2]).unwrap();
    let result2 = poseidon.get_hash().unwrap();
    
    // Reset und erneuter Hash nur mit input2
    poseidon.reset();
    poseidon.hash_many_bytes(&[&input2]).unwrap();
    let result3 = poseidon.get_hash().unwrap();
    
    assert_ne!(result1, result2, "Verschiedene Inputs sollten verschiedene Hashes ergeben");
    assert_ne!(result2, result3, "Hash ohne Reset sollte anders sein als Hash nach Reset");
}
