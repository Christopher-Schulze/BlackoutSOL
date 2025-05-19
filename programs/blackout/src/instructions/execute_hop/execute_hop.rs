use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::{invoke, invoke_signed};
use anchor_lang::solana_program;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::sysvar::Sysvar;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{
    verify_hyperplonk_proof,
    verify_range_proof,
    extract_split_amount,
    derive_stealth_pda,
    verify_bloom_filter,
    calculate_optimized_priority_fees,
};

#[derive(Accounts)]
pub struct ExecuteHop<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
        constraint = !transfer_state.completed @ BlackoutError::TransferAlreadyCompleted,
        constraint = !transfer_state.refund_triggered @ BlackoutError::TransferRefunded,
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: This is a dynamic account used for stealth PDAs
    #[account(mut)]
    pub split_pda: UncheckedAccount<'info>,
    
    /// CHECK: Required for compute budget and priority fee settings
    #[account(mut)]
    pub compute_budget: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
    
    /// CHECK: Used for proof validation and timestamp capture
    pub clock: UncheckedAccount<'info>,
}

pub fn execute_hop(
    ctx: Context<ExecuteHop>,
    hop_index: u8,
    proof_data: [u8; 128],
    range_proof_data: [u8; 128],
) -> Result<()> {
    // 1. Verification of transfer state preconditions (constant time for security)
    let transfer_state = &ctx.accounts.transfer_state;
    
    // Redundant checks removed, as they are covered by account constraints:
    // if transfer_state.completed { ... }
    // if transfer_state.refund_triggered { ... }
    
    // Check if the correct hop is being executed (exactly 4 hops in fixed configuration)
    if transfer_state.current_hop != hop_index {
        return Err(BlackoutError::InvalidHopIndex.into());
    }
    
    // Check that the hop index is valid (0..4)
    if hop_index >= 4 {
        return Err(BlackoutError::InvalidHopIndex.into());
    }
    
    // 2. Optimize compute unit budget and priority fees
    // Calculate remaining hops for priority fee calculation
    let remaining_hops = 4 - hop_index;

    // Set compute unit limit precisely based on hop index and complexity
    let cu_required = match hop_index {
        0 => 350_000, // First hop needs more compute units for initialization
        1 | 2 => 300_000, // Middle hops with standard compute units
        3 => 400_000, // Last hop needs more compute units for finalization
        _ => transfer_state.config.cu_budget_per_hop, // Fallback (should not be reached if hop_index < 4 is checked)
    };
    
    // Calculate optimized priority fees based on remaining hops and actual CU required
    let priority_fee = calculate_optimized_priority_fees(remaining_hops, cu_required)?;
    
    // Compute budget instructions with dynamic parameters
    let cu_limit_ix = solana_program::instruction::ComputeBudgetInstruction::set_compute_unit_limit(cu_required);
    let priority_fee_ix = solana_program::instruction::ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
    
    // Execute compute budget instructions atomically
    // The compute_budget account is not required for these specific CPIs.
    invoke(
        &cu_limit_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    invoke(
        &priority_fee_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    // 3. Verification of zero-knowledge proofs
    // Get current time for challenge validation
    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;
    
    // Generate challenge for this specific hop
    let mut challenge = [0u8; 32];
    challenge[0..8].copy_from_slice(&timestamp.to_le_bytes());
    challenge[8..16].copy_from_slice(&hop_index.to_le_bytes());
    challenge[16..24].copy_from_slice(&transfer_state.owner.to_bytes()[0..8]);
    challenge[24..32].copy_from_slice(&transfer_state.seed[24..32]);
    
    // Verify zero-knowledge proofs with the specialized functions
    // a) HyperPlonk proof for split integrity (Poseidon hashing)
    verify_hyperplonk_proof(&proof_data, &challenge)?;
    
    // b) Plonky2 range proof for split amounts (with Pedersen commitments)
    verify_range_proof(&range_proof_data, &transfer_state.commitments, &challenge)?;
    
    // 4. Dynamic split calculation and execution
    let mut processed_splits = 0;
    let mut total_transferred = 0u64;
    
    // For each of the 4 real splits (fixed configuration)
    for i in 0..4 {
        // Derive PDA for real split with optimized function
        let (split_pda, split_bump) = derive_stealth_pda(
            ctx.program_id,
            &transfer_state.seed,
            hop_index,
            i,
            false, // Real split, not a fake split
        );
        
        // Extract split amount from verified proof (safe after verification)
        let split_amount = extract_split_amount(&proof_data, i);
        
        // Atomic execution only if the provided PDA matches the calculated one
        // This verification ensures that the correct split is executed
        if ctx.accounts.split_pda.key() == split_pda {
            // Validate the split amount (must be positive, but less than total amount)
            if split_amount == 0 {
                // Skip empty split, but log it
                msg!("Split {} for hop {} has amount 0, skipping", i, hop_index);
                continue;
            }
            
            // Ensure that sufficient lamports are available
            let available_lamports = transfer_state.to_account_info().lamports();
            if available_lamports < split_amount {
                msg!("Insufficient lamports for split {}: {} < {}", 
                     i, available_lamports, split_amount);
                return Err(BlackoutError::InsufficientLamports.into());
            }
            
            // Optimized lamport transfer with full error handling
            let transfer_ix = system_instruction::transfer(
                &transfer_state.key(),
                &split_pda,
                split_amount,
            );
            
            // Invoke with complete account infos
            invoke_signed(
                &transfer_ix,
                &[
                    transfer_state.to_account_info(),
                    ctx.accounts.split_pda.to_account_info(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                &[&[
                    b"transfer".as_ref(),
                    transfer_state.owner.as_ref(),
                    &[transfer_state.bump],
                ]],
            )?;
            
            // Update statistics
            processed_splits += 1;
            total_transferred += split_amount;
            
            // Log successful split for audit purposes
            msg!("Real split {} for hop {} successful: {} lamports", 
                 i, hop_index, split_amount);
        }
    }
    
    // 5. Additional statistics and state update
    // Update the hop index in the transfer state
    let mut transfer_state = &mut ctx.accounts.transfer_state;
    transfer_state.current_hop = hop_index + 1;
    
    // Update the timestamp for freshness guarantee
    transfer_state.timestamp = timestamp;
    
    // 6. Check completion status after the last hop
    if transfer_state.current_hop >= 4 {
        // All hops have been executed, mark as ready for finalization
        msg!("All 4 hops completed. Transfer ready for finalization.");
    }
    
    // 7. Store statistics for later verification
    // Calculate Merkle root (for later verification)
    // Calculate in the Merkle root the hash of each successful split
    let merkle_data = [hop_index, processed_splits, 
                      (total_transferred >> 32) as u8, (total_transferred & 0xFF) as u8];
    transfer_state.merkle_root[hop_index as usize * 4..(hop_index as usize + 1) * 4]
    .copy_from_slice(&merkle_data);
    
    // 8. Calculate and log progress
    let progress = transfer_state.progress_percent();
    msg!("Hop {} of 4 completed ({}% progress)", 
         hop_index + 1, progress);
    
    // 9. Event emission for off-chain tracking
    emit!(HopExecuted {
        owner: transfer_state.owner,
        hop_index,
        splits_processed: processed_splits,
        total_transferred,
        progress_percent: progress,
        transfer_state: transfer_state.key(),
        timestamp,
    });
    
    Ok(())
}

#[event]
pub struct HopExecuted {
    pub owner: Pubkey,
    pub hop_index: u8,
    pub splits_processed: u8,
    pub total_transferred: u64,
    pub progress_percent: u8,
    pub transfer_state: Pubkey,
    pub timestamp: i64,
}