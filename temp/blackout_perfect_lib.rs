// BlackoutSOL Core Library
//
// Diese zentrale Bibliothek wurde umstrukturiert, um eine saubere Trennung zwischen
// der Kernfunktionalität und der Anchor-Integration zu gewährleisten.
// Die wichtigste Änderung ist die Einführung eines Brückenmoduls (anchor_types),
// das die Kommunikation zwischen den beiden Ebenen ermöglicht.

// Standard-Solana-Importe
use solana_program::pubkey::Pubkey;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;

// Programm-ID definieren für direkten Zugriff
#[cfg(not(feature = "no-entrypoint"))]
solana_program::declare_id!("B1ack111111111111111111111111111111111111111");

// Modul für Anchor-spezifische Typen und Brückenfunktionen
// Dies hält den Rest der Codebasis frei von direkten Anchor-Abhängigkeiten
pub mod anchor_types;

// Standardmodule der Anwendung
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;

// Optional: CMD-Modul für alternative Kontrolle
#[cfg(feature = "include-cmd")]
pub mod cmd;

// Entrypoint für direkten Solana-Programmaufruf (nicht über Anchor)
// Deaktiviert, wenn als Bibliothek verwendet
#[cfg(not(feature = "no-entrypoint"))]
solana_program::entrypoint!(process_instruction);

// Hauptfunktion für die Programmausführung
#[cfg(not(feature = "no-entrypoint"))]
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Delegation an den Instruction-Prozessor
    instructions::processor::process_instruction(program_id, accounts, instruction_data)
}
