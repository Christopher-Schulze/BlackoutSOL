// Bridge between pure Poseidon and Anchor framework
// This file serves as an adapter between our Anchor-free Poseidon module
// and the BlackoutSOL project that uses Anchor

use anchor_lang::prelude::*;
use crate::errors::BlackoutError;
use super::{hash, PurePoseidonError};

/// Converts an error from the Pure-Poseidon module to an Anchor Error
/// 
/// This function implements an exact type conversion using the error! macro,
/// which directly generates Anchor-compatible errors.
pub fn convert_error(err: PurePoseidonError) -> anchor_lang::error::Error {
    // Log the error for debugging
    msg!("Poseidon-Fehler aufgetreten: {:?}", err);
    
    // Categorize the error based on the specific error type
    match err {
        PurePoseidonError::HashingError(_) => error!(BlackoutError::HashingError),
        PurePoseidonError::ValidationError(_) => error!(BlackoutError::InvalidParameters),
        PurePoseidonError::ConversionError(_) => error!(BlackoutError::HashingError),
    }
}

/// Performs Poseidon parameter validation and returns an Anchor Result
pub fn validate_poseidon_parameters() -> Result<()> {
    hash::validate_parameters()
        .map_err(convert_error)
}

/// Generates a Poseidon hash for the specified inputs
pub fn generate_poseidon_hash(inputs: &[&[u8]]) -> Result<[u8; 32]> {
    hash::generate_hash(inputs)
        .map_err(convert_error)
}

/// Batch-Verarbeitung von Hash-Eingaben
pub fn batch_hash_inputs(input_sets: &[Vec<&[u8]>]) -> Result<Vec<[u8; 32]>> {
    hash::batch_hash(input_sets)
        .map_err(convert_error)
}

/// Simplified debugging function for displaying hash results
pub fn debug_hash(input: &[u8]) -> Result<()> {
    let hash_result = generate_poseidon_hash(&[input])?;
    msg!("Hash result for input: {:?}", hash_result);
    Ok(())
}
