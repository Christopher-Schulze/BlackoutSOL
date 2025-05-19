// This file (instructions/mod.rs) defines the structure of the instruction module

// IMPORTANT: To avoid naming conflicts with solana_program::sysvar::instructions::Instructions,
// we never use direct references to this type in this module without fully
// qualifying it as solana_program::sysvar::instructions::Instructions

// All submodules of the instructions module:
pub mod initialize;    // Initialization of an anonymous transfer
pub mod execute_hop;   // Execution of a single hop
pub mod finalize;      // Completion of an anonymous transfer
pub mod batch_hop;     // Efficient batch processing of multiple hops
pub mod refund;        // Refund mechanism
pub mod config_update; // Configuration changes
pub mod reveal_fake;   // Revealing fake splits for audit purposes
pub mod processor;     // Central command processor for manual integration

// Key strategy for avoiding naming conflicts:
// 1. Never use 'use solana_program::sysvar::instructions::Instructions;' in this module
// 2. Instead, always use the fully qualified path: solana_program::sysvar::instructions::Instructions
// 3. Or import and use the SolanaInstructions alias defined in lib.rs