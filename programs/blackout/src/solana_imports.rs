//! Central import file for Solana API compatibility
//! 
//! This file ensures that the correct import paths for the used
//! Solana version (1.18.26) are consistently used throughout the codebase and
//! only need to be updated in one place when version updates occur.

// Direct import of the Solana SDK ComputeBudget modules
pub use solana_sdk::compute_budget::ComputeBudgetInstruction;

// In Solana 1.18.26, sol_remaining_compute_units is in the compute_units module
pub use solana_program::compute_units::sol_remaining_compute_units;

// Re-export of the compute_budget module from Solana SDK for possible direct use
pub use solana_sdk::compute_budget;
// Standard program imports
pub use anchor_lang::solana_program::program::invoke;
pub use anchor_lang::solana_program::program::invoke_signed;

// Standard functions that are frequently used
pub use anchor_lang::solana_program::system_instruction;
pub use anchor_lang::solana_program::pubkey::Pubkey;
pub use anchor_lang::solana_program::account_info::AccountInfo;
pub use anchor_lang::solana_program::msg;
