// Processor module for handling all BlackoutSOL instructions
// This file serves as a central dispatcher for all program instructions
// and enables clean integration with the Solana program interface
// without direct dependency on Anchor

use solana_program::account_info::AccountInfo;
use solana_program::entrypoint::ProgramResult;
use solana_program::pubkey::Pubkey;
use solana_program::program_error::ProgramError;
use solana_program::msg;

// We use Anchor types directly for integration
// This simplifies integration with the Anchor framework

// Instruction types as an enum for the dispatcher
#[derive(Debug)]
pub enum BlackoutInstruction {
    // Initialize a new anonymous transfer
    Initialize {
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    },
    
    // Execute a single hop
    ExecuteHop {
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    },
    
    // Batch processing of multiple hops
    BatchHop {
        batch_index: u8,
    },
    
    // Finalize the transfer
    Finalize {
        proof_data: [u8; 128],
    },
    
    // Configuration update
    UpdateConfig {
        params: crate::instructions::config_update::ConfigUpdateParams,
    },
    
    // Refund request
    Refund {},
    
    // Reveal fake splits
    RevealFakeSplit {
        hop_index: u8,
        split_index: u8,
    },
}

// Processor function - called by the Solana program
pub fn process_instruction(
    _program_id: &Pubkey,
    _accounts: &[AccountInfo],
    _instruction_data: &[u8],
) -> ProgramResult {
    // This function is a placeholder for direct Solana integration
    // In reality, all functionality is provided by the blackout-anchor crate
    // We could implement manual deserialization here, but it's
    // not necessary since we use the Anchor integration through the separate crate
    
    msg!("BlackoutSOL: Direct call not supported. Please use the Anchor integration.");
    
    // In a complete implementation, we would:
    // 1. Extract the instruction tag from instruction_data
    // 2. Deserialize the corresponding data
    // 3. Route to the appropriate handler function
    
    // For now, we return a descriptive error
    Err(ProgramError::InvalidInstructionData)
}
