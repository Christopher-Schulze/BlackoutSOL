//! BlackoutSOL - Privacy-Payment System for Solana
//! Dies ist die Kernbibliothek, die nun von der Anchor-Integration getrennt ist.

// Standard-Importe
use solana_program::entrypoint::ProgramResult;

// Programm-ID definieren - wird von tests und CPI Clients verwendet
#[cfg(not(feature = "no-entrypoint"))]
solana_program::declare_id!("B1ack111111111111111111111111111111111111111");

// Standardmodule
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;

// Optional CMD-Modul für alternative Kontrolle
#[cfg(feature = "include-cmd")]
pub mod cmd;

// Diese Funktion ist ein Platzhalter für direkten, manuellen Zugriff auf das Programm
// Ohne das Anchor-Framework
#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

/// Hauptprozessorfunktion für direkte Solana-Programmaufrufe
/// Diese wird nur verwendet, wenn das Feature "no-entrypoint" nicht aktiviert ist
#[cfg(not(feature = "no-entrypoint"))]
pub fn process_instruction(
    program_id: &solana_program::pubkey::Pubkey,
    accounts: &[solana_program::account_info::AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Hier würde eine manuelle Deserialisierung und Delegation erfolgen
    // Dies ist ein Platzhalter, da die eigentliche Implementierung über das
    // Anchor-Framework im separaten blackout-anchor-Crate erfolgt.
    Err(solana_program::program_error::ProgramError::Custom(0))
}
