//! # BlackoutSOL - Privacy-Preserving Payment System for Solana
//! 
//! BlackoutSOL is a privacy-focused payment system built on Solana that enables
//! confidential transactions while maintaining the security and performance of
//! the Solana blockchain.
//!
//! ## Architecture Overview
//!
//! The system is composed of several key components:
//! - **Core Program**: Handles the main logic for private transactions
//! - **Poseidon Hashing**: Provides ZK-friendly cryptographic primitives
//! - **State Management**: Manages the on-chain state of transactions and accounts
//! - **Instruction Processing**: Handles the various transaction types
//!
//! ## Security Model
//!
//! BlackoutSOL uses zero-knowledge proofs to ensure transaction privacy while
//! maintaining the security guarantees of the Solana blockchain.
//!
//! ## Modules
//! - `instructions`: Implementation of the program's instructions
//! - `state`: Data structures for on-chain state
//! - `utils`: Utility functions and helpers
//! - `poseidon_validator`: Poseidon hashing implementation
//! - `errors`: Error types and handling
//!
//! ## Getting Started
//!
//! ### Prerequisites
//! - Rust 1.65.0 or later
//! - Solana CLI tools
//! - Anchor framework
//!
//! ### Building
//! ```bash
//! cargo build-bpf
//! ```
//!
//! ### Testing
//! ```bash
//! cargo test-bpf
//! ```
//!
//! For more details, see the [documentation](https://blackoutsol.dev/docs).

// Standard imports
use anchor_lang::prelude::*;
use anchor_lang::solana_program::program_error::ProgramError;

// Re-export commonly used types for easier access
pub use anchor_lang::{
    solana_program,
    system_program,
    prelude::*,
};

// Re-export program_error for consistency
pub use ProgramError;

// Core modules
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;

// Re-export important types
pub use state::{
    TransferConfig,
    TransferState,
    TransferStateStatus,
    BlackoutError,
};

// Re-export instructions for easier access
pub use instructions::{
    initialize,
    execute_hop,
    batch_hop,
    finalize,
    config_update,
    refund,
    reveal_fake,
};

// Re-export utils for external use
pub use utils::{
    TransferContext,
    Loggable,
};

// Program errors
use anchor_lang::solana_program::program_error::ProgramError;
use thiserror::Error;

/// Custom error type for the Blackout program
#[derive(Error, Debug, Copy, Clone)]
pub enum BlackoutError {
    /// Invalid configuration
    #[error("Invalid configuration")]
    InvalidConfig,
    
    /// Arithmetic overflow
    #[error("Arithmetic overflow")]
    ArithmeticOverflow,
    
    /// Invalid proof
    #[error("Invalid proof")]
    InvalidProof,
    
    /// Invalid range proof
    #[error("Invalid range proof")]
    InvalidRangeProof,
    
    /// Transfer not found
    #[error("Transfer not found")]
    TransferNotFound,
    
    /// Unauthorized
    #[error("Unauthorized")]
    Unauthorized,
    
    /// Invalid state transition
    #[error("Invalid state transition")]
    InvalidStateTransition,
    
    /// Invalid recipient
    #[error("Invalid recipient")]
    InvalidRecipient,
    
    /// Insufficient funds
    #[error("Insufficient funds")]
    InsufficientFunds,
    
    /// Transfer already completed
    #[error("Transfer already completed")]
    TransferCompleted,
    
    /// Transfer expired
    #[error("Transfer expired")]
    TransferExpired,
}

impl From<BlackoutError> for ProgramError {
    fn from(e: BlackoutError) -> Self {
        ProgramError::Custom(e as u32)
    }
}

impl std::fmt::Display for BlackoutError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

// Program ID
declare_id!("B1ack111111111111111111111111111111111111111");

// Core Solana imports for compatibility and easier maintenance
pub mod solana_imports;

// Standard modules
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;

// Re-export instruction modules for external use
pub use instructions::initialize;
pub use instructions::execute_hop;
pub use instructions::batch_hop;
pub use instructions::finalize;
pub use instructions::config_update;
pub use instructions::refund;
pub use instructions::reveal_fake;

// Formal verification modules, only available with the "verification" feature
#[cfg(feature = "verification")]
pub mod verification;
