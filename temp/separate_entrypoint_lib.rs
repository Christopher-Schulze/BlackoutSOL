//! BlackoutSOL - Privacy-Payment System for Solana

// Standard-Importe
use anchor_lang::prelude::*;
use solana_program::account_info::AccountInfo;
use solana_program::entrypoint;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;

// Programm-ID für die Identifikation im Solana-Netzwerk
declare_id!("B1ack111111111111111111111111111111111111111");

// Standardmodule
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;

// Brückenmodule für Namenskonflikte
pub mod cmd;
pub mod anchor_bridge;

// LÖSUNG: Wir deaktivieren das Anchor-Makro und implementieren einen
// eigenen Entrypoint, um das Problem mit dem #[program]-Makro zu umgehen

// Einfacher manueller Entrypoint für die Solana-Programm-Integration
#[cfg(not(feature = "no-entrypoint"))]
entrypoint!(process_instruction);

// Programm-Handler-Funktion
pub fn process_instruction(
    program_id: &Pubkey,
    accounts: &[AccountInfo],
    instruction_data: &[u8],
) -> ProgramResult {
    // Wir delegieren an den Anchor-Prozessor, aber umgehen das #[program]-Makro
    instructions::processor::process_instruction(program_id, accounts, instruction_data)
}

// Export der Anchor-Strukturen für Client-Anwendungen
pub use instructions::initialize::Initialize;
pub use instructions::execute_hop::ExecuteHop; 
pub use instructions::batch_hop::BatchHop;
pub use instructions::finalize::Finalize;
pub use instructions::config_update::ConfigUpdate;
pub use instructions::config_update::ConfigUpdateParams;
pub use instructions::refund::Refund;
pub use instructions::reveal_fake::RevealFake;
