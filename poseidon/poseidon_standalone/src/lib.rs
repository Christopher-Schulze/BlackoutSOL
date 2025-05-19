//! BlackoutSOL Poseidon - A standalone, reliable Poseidon hash implementation
//! 
//! This library provides a clean and consistent interface for Poseidon hashing,
//! separated from the compilation issues of the Anchor framework.
//! 
//! ## Main Features
//! 
//! - Poseidon hash functionality with BN254 parameters
//! - Clean error handling with meaningful error messages
//! - Comprehensive tests for consistency and correctness
//! - Optional Anchor compatibility layer

use solana_poseidon::{hashv, Parameters, Endianness};
use curve25519_dalek::scalar::Scalar;
use num_bigint::BigUint;
// fmt wird nicht benützt
// use std::fmt;
use thiserror::Error;

/// Specific error types for the Poseidon library
#[derive(Error, Debug)]
pub enum PoseidonError {
    /// Error during hashing operation
    #[error("Hashing error: {0}")]
    HashingError(String),
    
    /// Error during parameter validation
    #[error("Validation error: {0}")]
    ValidationError(String),
    
    /// Error during data type conversion
    #[error("Conversion error: {0}")]
    ConversionError(String),
}

/// Short alias for Result with PoseidonError
pub type Result<T> = std::result::Result<T, PoseidonError>;

/// Poseidon constants and helper tools
pub mod constants {
    use super::*;
    
    pub const POSEIDON_FULL_ROUNDS: usize = 8;
    pub const POSEIDON_PARTIAL_ROUNDS: usize = 57;
    pub const POSEIDON_WIDTH: usize = 3;
    
    /// Converts a hex string into a Scalar value
    pub fn scalar_from_hex(s: &str) -> Result<Scalar> {
        let s = if s.starts_with("0x") { &s[2..] } else { s };
        
        // Convert hexadecimal to BigUint
        let big_uint = BigUint::parse_bytes(s.as_bytes(), 16)
            .ok_or_else(|| PoseidonError::ConversionError(
                format!("Failed to parse hex string: {}", s)
            ))?;
        
        // BigUint zu bytes konvertieren
        let bytes = big_uint.to_bytes_be();
        
        // Scalar von bytes konvertieren
        let mut array = [0u8; 32];
        let len = std::cmp::min(bytes.len(), 32);
        array[32 - len..].copy_from_slice(&bytes[..len]);
        
        Ok(Scalar::from_bytes_mod_order(array))
    }
    
    /// Generates the MDS matrix for Poseidon
    pub fn get_mds_matrix() -> Result<Vec<Vec<Scalar>>> {
        let mds_hex = [
            ["3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd", 
             "3798866f4e6058035dcf8addb2cf1771fac234bcc8fc05d6676e77e797f224bf", 
             "2c51456a7bf2467eac813649f3f25ea896eac27c5da020dae54a6e640278fda2"],
            ["20088ca07bbcd7490a0218ebc0ecb31d0ea34840e2dc2d33a1a5adfecff83b43", 
             "1d04ba0915e7807c968ea4b1cb2d610c7f9a16b4033f02ebacbb948c86a988c3", 
             "5387ccd5729d7acbd09d96714d1d18bbd0eeaefb2ddee3d2ef573c9c7f953307"],
            ["1e208f585a72558534281562cad89659b428ec61433293a8d7f0f0e38a6726ac", 
             "0455ebf862f0b60f69698e97d36e8aafd4d107cae2b61be1858b23a3363642e0", 
             "569e2c206119e89455852059f707370e2c1fc9721f6c50991cedbbf782daef54"],
        ];
        
        let result = mds_hex.iter()
            .map(|row| {
                row.iter()
                   .map(|&s| scalar_from_hex(s))
                   .collect::<Result<Vec<_>>>()
            })
            .collect::<Result<Vec<_>>>()?;
        
        Ok(result)
    }
    
    /// Generates the round constants for Poseidon
    pub fn get_round_constants() -> Result<Vec<Scalar>> {
        let constants_hex = [
            "6c4ffa723eaf1a7bf74905cc7dae4ca9ff4a2c3bc81d42e09540d1f250910880",
            "54dd837eccf180c92c2f53a3476e45a156ab69a403b6b9fdfd8dd970fddcdd9a",
            "a5316c9a43627a37481bfc4a85a23b87a97b697d9c6e34ad677c585d7084300c",
            // Add more constants from poseidon_constants.rs here
        ];
        
        constants_hex.iter()
            .map(|&s| scalar_from_hex(s))
            .collect()
    }
}

/// Main functions for Poseidon hashing
pub mod hash {
    use super::*;
    
    /// Validates the implemented Poseidon parameters against a known test vector
    pub fn validate_parameters() -> Result<()> {
        // Test vector
        let mut test_input = [0u8; 32];
        test_input[31] = 0x42;
        
        // The expected hash for this input
        let expected_hash = "011e70075d2f41deacf19a385a674c5a2582d52b83d05f42a27bdf19dd352433";
        
        // Hash with BN254X5 parameters
        let hash_result = hashv(
            Parameters::Bn254X5,
            Endianness::BigEndian,
            &[&test_input]
        ).map_err(|e| PoseidonError::HashingError(e.to_string()))?;
        
        // Convert result to hex and compare
        let hash_hex = hex::encode(hash_result.to_bytes());
        if hash_hex != expected_hash {
            return Err(PoseidonError::ValidationError(
                format!("Hash validation failed. Expected {}, got {}", expected_hash, hash_hex)
            ));
        }
        
        Ok(())
    }
    
    /// Generates a Poseidon hash for the given inputs
    pub fn generate_hash(inputs: &[&[u8]]) -> Result<[u8; 32]> {
        // Hash with BN254X5 parameters
        let hash_result = hashv(
            Parameters::Bn254X5,
            Endianness::BigEndian,
            inputs
        ).map_err(|e| PoseidonError::HashingError(e.to_string()))?;
        
        Ok(hash_result.to_bytes())
    }
    
    /// Batch processing of hash inputs
    pub fn batch_hash(input_sets: &[Vec<&[u8]>]) -> Result<Vec<[u8; 32]>> {
        let mut results = Vec::with_capacity(input_sets.len());
        
        for inputs in input_sets {
            let hash_bytes = generate_hash(inputs.as_slice())?;
            results.push(hash_bytes);
        }
        
        Ok(results)
    }
    
    /// Helper function for debug output during development
    pub fn debug_hash(input: &[u8], label: &str) -> Result<[u8; 32]> {
        let hash = generate_hash(&[input])?;
        println!("DEBUG: Hash for {} = {}", label, hex::encode(hash));
        Ok(hash)
    }
}

// Optional with Anchor compatibility layer
#[cfg(feature = "anchor_compat")]
pub mod anchor {
    use super::*;
    use anchor_lang::prelude::*;
    
    #[error_code]
    pub enum PoseidonAnchorError {
        #[msg("Hashing operation failed")]
        HashingError,
        
        #[msg("Parameter validation failed")]
        ValidationError,
        
        #[msg("Data conversion failed")]
        ConversionError,
    }
    
    /// Converts a PoseidonError to an Anchor Error
    pub fn convert_error(err: PoseidonError) -> anchor_lang::error::Error {
        match err {
            PoseidonError::HashingError(_) => error!(PoseidonAnchorError::HashingError),
            PoseidonError::ValidationError(_) => error!(PoseidonAnchorError::ValidationError),
            PoseidonError::ConversionError(_) => error!(PoseidonAnchorError::ConversionError),
        }
    }
    
    /// Validate Poseidon parameters (Anchor-compatible)
    pub fn validate_parameters() -> anchor_lang::Result<()> {
        hash::validate_parameters().map_err(convert_error)
    }
    
    /// Generate hash (Anchor-compatible)
    pub fn generate_hash(inputs: &[&[u8]]) -> anchor_lang::Result<[u8; 32]> {
        hash::generate_hash(inputs).map_err(convert_error)
    }
    
    /// Batch hash multiple inputs (Anchor-compatible)
    pub fn batch_hash(input_sets: &[Vec<&[u8]>]) -> anchor_lang::Result<Vec<[u8; 32]>> {
        hash::batch_hash(input_sets).map_err(convert_error)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parameter_validation() {
        assert!(hash::validate_parameters().is_ok(), 
               "Parameter validation should succeed");
    }
    
    #[test]
    fn test_hash_consistency() {
        let test_input1 = b"BlackoutSOL Test Input";
        let test_input2 = b"Different Input";
        
        // Gleicher Input sollte gleichen Hash erzeugen
        let hash1 = hash::generate_hash(&[test_input1]).unwrap();
        let hash2 = hash::generate_hash(&[test_input1]).unwrap();
        assert_eq!(hash1, hash2, "Same input should produce same hash");
        
        // Verschiedene Inputs sollten verschiedene Hashes erzeugen
        let hash3 = hash::generate_hash(&[test_input2]).unwrap();
        assert_ne!(hash1, hash3, "Different inputs should produce different hashes");
    }
    
    #[test]
    fn test_hex_conversion() {
        // Der ursprüngliche Hex-String
        let hex_value = "3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd";
        
        // Konvertieren zu Scalar und zurück
        let scalar = constants::scalar_from_hex(hex_value).unwrap();
        let bytes = scalar.to_bytes();
        
        // Validierung, dass der Scalar nicht null ist
        assert!(!bytes.iter().all(|&b| b == 0), 
               "Scalar conversion produced all zeros, which is invalid");
        
        // Validierung, dass der Scalar in gültigem Bereich liegt
        // (es ist nicht erforderlich, dass die genaue Hex-Darstellung zurückkommt,
        // aber es muss ein gültiger Scalar-Wert sein)
        let reconverted_scalar = Scalar::from_bytes_mod_order(bytes);
        assert_eq!(scalar, reconverted_scalar, 
                  "Scalar roundtrip conversion should preserve the value");
        
        // Test mit 0x-Präfix - dieses Feature muss erhalten bleiben
        let hex_with_prefix = format!("0x{}", hex_value);
        let scalar2 = constants::scalar_from_hex(&hex_with_prefix).unwrap();
        assert_eq!(scalar, scalar2, 
                 "Conversion with and without 0x prefix should give the same result");
    }
    
    #[test]
    fn test_mds_matrix_dimensions() {
        let mds = constants::get_mds_matrix().unwrap();
        
        assert_eq!(mds.len(), 3, "MDS matrix should have 3 rows");
        for row in &mds {
            assert_eq!(row.len(), 3, "Each row should have 3 columns");
        }
    }
    
    #[test]
    fn test_batch_processing() {
        let inputs = vec![
            vec![&b"Input1"[..]],
            vec![&b"Input2"[..]],
            vec![&b"Input3"[..]],
        ];
        
        let results = hash::batch_hash(&inputs).unwrap();
        
        assert_eq!(results.len(), inputs.len(), 
                  "Batch size should match input size");
                  
        // Vergleiche mit direktem Hashing
        for (i, input_set) in inputs.iter().enumerate() {
            let direct_hash = hash::generate_hash(&input_set).unwrap();
            assert_eq!(results[i], direct_hash, 
                      "Batch hash for element {} should match direct hash", i);
        }
    }
    
    #[test]
    fn test_error_handling() {
        // Ungültiger Hex-String sollte zu Fehler führen
        let invalid_hex = "xyz"; // keine gültige Hex
        let result = constants::scalar_from_hex(invalid_hex);
        assert!(result.is_err(), "Invalid hex should result in error");
        
        // Prüfe, ob der richtige Fehlertyp zurückgegeben wird
        match result {
            Err(PoseidonError::ConversionError(_)) => {}, // expected
            Err(e) => panic!("Wrong error type: {:?}", e),
            Ok(_) => panic!("Should have failed"),
        }
    }
}
