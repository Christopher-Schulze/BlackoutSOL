//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Importieren aller Programmmodule
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Original-Modul wird beibehalten
pub mod instructions;

// Importieren des Entrypoint-Moduls speziell für die Anker-Integration
// Dieses Modul vermeidet direkte Importe von solana_program::sysvar::instructions
pub mod entrypoint;

// Re-Export des Anchor-Entrypoints
pub use entrypoint::*;
