/// Poseidon Validator Integration for BlackoutSOL
/// 
/// This file integrates the validated Poseidon parameters into the BlackoutSOL application.
/// Starting with version 3.0, the standalone `blackout_poseidon` package is used, which provides
/// a robust and independent implementation for Poseidon hashing.
/// 
/// MODULAR ARCHITECTURE:
/// - core: Contains core functionality without Anchor dependencies
/// - anchor: Adapter for using Poseidon with Anchor

// External crates and global imports
extern crate hex;
use blackout_poseidon; // Independent Poseidon package

// Selective imports for different modules
#[cfg(not(feature = "no-entrypoint"))]
use anchor_lang::prelude::*;


/// Error types and core functionality, independent of Anchor
pub mod core {
    use blackout_poseidon;
    
    /// Custom error type for Poseidon-specific errors
    /// This structure is independent of Anchor dependencies
    #[derive(Debug)]
    pub enum PoseidonError {
        InvalidParameters,
        HashingError,
        BatchError,
    }
    
    /// Validation of Poseidon parameters - Core functionality without Anchor dependencies
    pub fn validate_parameters() -> std::result::Result<(), PoseidonError> {
        // Delegate to the independent package
        match blackout_poseidon::hash::validate_parameters() {
            Ok(_) => Ok(()),
            Err(_) => Err(PoseidonError::InvalidParameters),
        }
    }
    
    /// Generates hashes for ZK proofs - Core implementation
    pub fn generate_hash(inputs: &[&[u8]]) -> std::result::Result<[u8; 32], PoseidonError> {
        blackout_poseidon::hash::generate_hash(inputs)
            .map_err(|_| PoseidonError::HashingError)
    }
    
    /// Batch processing - Core implementation
    pub fn batch_process(input_sets: &[Vec<&[u8]>]) -> std::result::Result<Vec<[u8; 32]>, PoseidonError> {
        blackout_poseidon::hash::batch_hash(input_sets)
            .map_err(|_| PoseidonError::BatchError)
    }
}

/// Re-export of the independent error type for backward compatibility
#[derive(Debug)]
pub enum PoseidonError {
    InvalidParameters,
    HashingError,
    BatchError,
    SerializationError,
}

/// Anchor-specific adapter implementations for the Poseidon validator
/// These functions have Anchor dependencies and are only compiled if
/// the no-entrypoint feature is not enabled
#[cfg(not(feature = "no-entrypoint"))]
pub mod anchor {
    use super::*;
    use super::core;
    
    use crate::errors::BlackoutError;
     // Import of the msg! macro from our central import file

    /// Converts core Poseidon errors to Anchor errors
    pub fn core_error_to_anchor_error(err: &core::PoseidonError) -> anchor_lang::error::Error {
        match err {
            core::PoseidonError::InvalidParameters => BlackoutError::InvalidParameters.into(),
            core::PoseidonError::HashingError => BlackoutError::HashingError.into(),
            core::PoseidonError::BatchError => BlackoutError::HashingError.into(), // Use HashingError for batch errors as well
        }
    }
    
    /// Converts local Poseidon errors to Anchor errors for backward compatibility
    pub fn poseidon_error_to_anchor_error(err: &super::PoseidonError) -> anchor_lang::error::Error {
        match err {
            super::PoseidonError::InvalidParameters => BlackoutError::InvalidParameters.into(),
            super::PoseidonError::HashingError => BlackoutError::HashingError.into(),
            super::PoseidonError::BatchError => BlackoutError::HashingError.into(),
            super::PoseidonError::SerializationError => BlackoutError::DeserializationError.into(),
        }
    }
    
    /// Converts Blackout-Poseidon errors to Anchor errors
    pub fn blackout_poseidon_error_to_anchor_error(err: &blackout_poseidon::PoseidonError) -> anchor_lang::error::Error {
        match err {
            blackout_poseidon::PoseidonError::ValidationError(_) => BlackoutError::InvalidParameters.into(),
            blackout_poseidon::PoseidonError::HashingError(_) => BlackoutError::HashingError.into(),
            blackout_poseidon::PoseidonError::ConversionError(_) => BlackoutError::DeserializationError.into(),
            // Note: All cases are already covered, no fallback needed
        }
    }
}

/// Performs a quick validation of the Poseidon parameters
/// 
/// This function delegates the core functionality to the independent Poseidon module,
/// but maintains the existing API and logging.
#[cfg(not(feature = "no-entrypoint"))]
pub fn validate_poseidon_parameters() -> Result<()> {
    msg!("Validating Poseidon parameters...");
    
    // Delegiere an das Core-Modul
    match core::validate_parameters() {
        Ok(_) => {
            msg!("Poseidon-Parameter erfolgreich validiert.");
            Ok(())
        },
        Err(e) => {
            msg!("Poseidon-Parameter-Validierung fehlgeschlagen: {:?}", e);
            Err(anchor::core_error_to_anchor_error(&e))
        }
    }
}

/// Generates consistent hashes for use in Zero-Knowledge proofs
/// This function is optimized for integration with the BlackoutSOL project
/// 
/// Uses the core module and converts errors to Anchor errors
// This function is now always available, regardless of entrypoint features
pub fn generate_zk_hash(inputs: &[&[u8]]) -> anchor_lang::Result<[u8; 32]> {
    core::generate_hash(inputs)
        .map_err(|e| {
            anchor_lang::solana_program::msg!("Poseidon-Hashing fehlgeschlagen: {:?}", e);
            crate::errors::BlackoutError::HashingError.into()
        })
}

/// Performs efficient batch processing of multiple inputs
/// Optimized for BlackoutSOL with integrated error handling
/// 
/// Uses the core module with Anchor error handling
#[cfg(not(feature = "no-entrypoint"))]
pub fn batch_hash_inputs(input_sets: &[Vec<&[u8]>]) -> Result<Vec<[u8; 32]>> {
    core::batch_process(input_sets)
        .map_err(|e| {
            msg!("Poseidon batch processing failed: {:?}", e);
            anchor::core_error_to_anchor_error(&e)
        })
}
/// This function forwards to the Anchor adapter
#[cfg(not(feature = "no-entrypoint"))]
pub fn poseidon_error_to_blackout_error(err: PoseidonError) -> anchor_lang::error::Error {
    anchor::poseidon_error_to_anchor_error(&err)
}
    ///
    /// This function forwards to the Anchor adapter
#[cfg(not(feature = "no-entrypoint"))]
pub fn blackout_poseidon_error_to_error(err: blackout_poseidon::PoseidonError) -> anchor_lang::error::Error {
    anchor::blackout_poseidon_error_to_anchor_error(&err)
}

// These functions are independent of Anchor and use the core functionality
// for tests or other applications without Anchor dependencies

/// Test API: Validates Poseidon parameters without Anchor dependencies
pub fn validate_for_test() -> std::result::Result<(), String> {
    // Direct use of the core module
    core::validate_parameters()
        .map_err(|e| format!("Validation failed: {:?}", e))
}

/// Test API: Generates hashes without Anchor dependencies
pub fn generate_hash_for_test(inputs: &[&[u8]]) -> std::result::Result<[u8; 32], String> {
    // Direct use of the core module
    core::generate_hash(inputs)
        .map_err(|e| format!("Hashing failed: {:?}", e))
}

/// Test API: Performs batch processing without Anchor dependencies
pub fn batch_hash_for_test(input_sets: &[Vec<&[u8]>]) -> std::result::Result<Vec<[u8; 32]>, String> {
    // Direct use of the core module
    core::batch_process(input_sets)
        .map_err(|e| format!("Batch processing failed: {:?}", e))
}

/// Test API: Debug helper function for individual hashes without Anchor dependencies
pub fn debug_hash_for_test(input: &[u8]) -> std::result::Result<[u8; 32], String> {
    blackout_poseidon::hash::debug_hash(input, "test")
        .map_err(|e| format!("Debug hash error: {:?}", e))
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_parameter_validation() {
        // Checks if the core functionality works correctly
        assert!(core::validate_parameters().is_ok());
        // Checks if the test API works correctly
        assert!(validate_for_test().is_ok());
    }
    
    #[test]
    fn test_zk_hash_consistency() {
        let inputs = [
            &[1, 2, 3][..],
            &[4, 5, 6][..],
        ];
        
        // Direct test of core functionality
        let core_hash1 = core::generate_hash(&inputs).unwrap();
        let core_hash2 = core::generate_hash(&inputs).unwrap();
        assert_eq!(core_hash1, core_hash2, "Core-Hash-Funktionalität ist inkonsistent");
        
        // Test der Test-API
        let test_hash = generate_hash_for_test(&inputs).unwrap();
        
        // Beide Methoden sollten konsistente Ergebnisse liefern
        assert_eq!(core_hash1, test_hash, "Core und Test-API produzieren unterschiedliche Hashes");
    }
    
    #[test]
    fn test_batch_processing() {
        let input_sets = [
            vec![&[1, 2, 3][..], &[4, 5, 6][..]],
            vec![&[7, 8, 9][..], &[10, 11, 12][..]],
        ];
        
        // Test core functionality
        let core_batch = core::batch_process(&input_sets).unwrap();
        
        // Test der Test-API
        let test_batch = batch_hash_for_test(&input_sets).unwrap();
        
        // Individual hashes for comparison
        let single1 = core::generate_hash(&input_sets[0]).unwrap();
        let single2 = core::generate_hash(&input_sets[1]).unwrap();
        
        // Consistency check
        assert_eq!(core_batch.len(), 2, "Incorrect batch result size");
        assert_eq!(core_batch[0], single1, "First batch element inconsistent");
        assert_eq!(core_batch[1], single2, "Second batch element inconsistent");
        assert_eq!(core_batch, test_batch, "Core and test API produce different batch results");
    }
    
    #[test]
    fn test_error_types() {
        // Test der Fehlertypen und Konvertierungen
        let core_err = core::PoseidonError::HashingError;
        let compat_err = PoseidonError::HashingError;
        
        // Ensure both error types exist and can be used
        assert!(matches!(core_err, core::PoseidonError::HashingError));
        assert!(matches!(compat_err, PoseidonError::HashingError));
    }
    
    #[test]
    fn test_error_conversion() {
        // Test of error conversion - should compile but doesn't necessarily need to be executed
        let err = blackout_poseidon::PoseidonError::HashingError("Testfehler".to_string());
        let _anchor_err = blackout_poseidon_error_to_error(err);
        // No further assertions needed - compilability is the main goal
    }
}
