// anchor_types.rs
//
// This file provides bridge types and functions that establish a consistent
// interface between the core library and the Anchor integration.
//
// This allows us to keep the actual code independent of Anchor,
// while still providing the necessary types for Anchor integration.

pub use anchor_lang;
pub use anchor_lang::prelude::*;
pub use anchor_lang::solana_program;

// Re-exports of the ID for internal use
#[cfg(not(feature = "no-entrypoint"))]
pub use crate::ID;

// A concrete Anchor error type for the core library
#[error_code]
pub enum BlackoutAnchorError {
    #[msg("Invalid parameter")]
    InvalidParameter,
    #[msg("Hashing error")]
    HashingError,
    #[msg("Validation error")]
    ValidationError,
    #[msg("Deserialization error")]
    DeserializationError,
    #[msg("Transfer already completed")]
    TransferAlreadyCompleted,
    #[msg("Invalid hop configuration")]
    InvalidHopConfiguration,
    #[msg("Too many splits")]
    TooManySplits,
}

// Helper functions for error conversion
pub fn map_err_to_anchor<T, E: std::fmt::Debug>(result: std::result::Result<T, E>, error_code: BlackoutAnchorError) -> Result<T> {
    result.map_err(|e| {
        msg!("Error: {:?}", e);
        error_code.into()
    })
}

// Ensures that the ID is available in no-entrypoint mode
#[cfg(feature = "no-entrypoint")]
pub static ID: solana_program::pubkey::Pubkey = solana_program::pubkey::Pubkey::new_from_array([
    0xB1, 0xac, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 
    0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11, 0x11
]);
