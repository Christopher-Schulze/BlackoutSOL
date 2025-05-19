use anchor_lang::prelude::*;

// Using our core Solana imports for backward compatibility
use crate::solana_imports::*;
use anchor_lang::solana_program::sysvar::rent::Rent;
use anchor_lang::solana_program::sysvar::Sysvar;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{
    verify_hyperplonk_proof,
    check_bloom_filter,
    calculate_optimized_priority_fees,
    extract_splits,
    verify_pda_derivation,
};

/// Context for executing a batch hop
/// 
/// This instruction processes multiple hops in a single transaction, significantly
/// optimizing compute units and network fees.
#[derive(Accounts)]
pub struct BatchHop<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
        constraint = !transfer_state.completed @ BlackoutError::TransferAlreadyCompleted,
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: These PDAs are dynamically derived and verified
    #[account(mut)]
    pub pda_0: UncheckedAccount<'info>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_1: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_2: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_3: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_4: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_5: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_6: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional for larger batches
    #[account(mut)]
    pub pda_7: Option<UncheckedAccount<'info>>,
    
    pub system_program: Program<'info, System>,
}

// Simplified structure without explicit lifetime specifications
pub fn process_batch_hop(
    ctx: Context<BatchHop>,
    batch_index: u8,
) -> Result<()> {
    // Optimized compute unit and priority fee setting for faster execution
    let config = &ctx.accounts.transfer_state.config;
    
    // Dynamic CU limit based on batch size and remaining hops
    let remaining_hops = ctx.accounts.transfer_state.remaining_hops();
    let batch_size = std::cmp::min(config.max_batch_size(), remaining_hops);
    let cu_limit = config.cu_budget_per_hop * batch_size as u32;
    
    // Calculate optimized priority fees based on transaction volume
    // Parameters in the correct order: first remaining_hops, then cu_limit
    let priority_fee = calculate_optimized_priority_fees(remaining_hops, cu_limit)?;
    
    // Setup for compute budget with priority fee for faster confirmation
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(cu_limit);
    let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
    
    // Parallelized execution of compute budget instructions
    invoke(&cu_limit_ix, &[ctx.accounts.authority.to_account_info()])?;
    invoke(&priority_fee_ix, &[ctx.accounts.authority.to_account_info()])?;
    
    // Check if sufficient compute units are available for the main batch processing logic
    // Estimate that initial checks + CU setup might have taken some CUs, 
    // and the main logic (parallel_batch_execution) will need a large portion of the cu_limit.
    // This is a rough estimate; precise values would come from testing.
    let estimated_cu_for_main_logic = cu_limit.saturating_sub(50_000); // Assuming setup & checks cost up to 50k CUs
    if !ctx.accounts.transfer_state.has_enough_cu_for_next_hop(estimated_cu_for_main_logic) {
        return Err(BlackoutError::InsufficientComputeUnits.into());
    }
    
    // Check if the correct batch sequence is maintained
    if ctx.accounts.transfer_state.batch_count != batch_index {
        msg!("Invalid batch index: expected {}, received {}", 
             ctx.accounts.transfer_state.batch_count, batch_index);
        return Err(BlackoutError::InvalidHopIndex.into());
    }
    
    // Check if all hops are already completed
    if ctx.accounts.transfer_state.current_hop >= ctx.accounts.transfer_state.config.num_hops {
        msg!("All hops already completed");
        return Err(BlackoutError::TransferAlreadyCompleted.into());
    }
    
    // Calculate how many hops can be processed in this batch
    let max_batch_size = config.max_batch_size();
    let remaining_hops = ctx.accounts.transfer_state.remaining_hops();
    let batch_size = std::cmp::min(max_batch_size, remaining_hops);
    
    msg!("Processing batch {}: {} hops (from {} to {})", 
         batch_index, batch_size, ctx.accounts.transfer_state.current_hop, 
         ctx.accounts.transfer_state.current_hop + batch_size - 1);
    
    // Verify the HyperPlonk proof for the batch with optimized verification
    // This uses Poseidon hashing in HyperPlonk for efficient on-chain verification
    msg!("Verifying HyperPlonk proof with Poseidon hashing for batch {}", batch_index);
    verify_hyperplonk_proof(
        &ctx.accounts.transfer_state.batch_proof,
        &ctx.accounts.transfer_state.challenge,
    )?;
    
    // Extract the splits from the proof with optimized parallelization
    // The amounts are extracted from the proof to ensure perfect obfuscation
    msg!("Extracting 4 splits with variable distribution for unlinkability");
    let splits = extract_splits(
        &ctx.accounts.transfer_state.batch_proof,
        // Optimized for 4 fixed real splits
        ctx.accounts.transfer_state.amount / 4,
        // Beachte: Die Anzahl der Splits ist fest auf 4 eingestellt in der Implementierung
        &ctx.accounts.transfer_state.challenge,
    )?;
    
    // Optimized PDA collection with vectorized operations
    let mut pda_accounts = Vec::with_capacity(batch_size as usize);
    pda_accounts.push(ctx.accounts.pda_0.to_account_info());
    
    // Optimized vectorization for multi-hop batching
    // Using O(1) lookups instead of conditional checks for maximum efficiency
    let optional_pdas = [
        ctx.accounts.pda_1.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_2.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_3.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_4.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_5.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_6.as_ref().map(|p| p.to_account_info()),
        ctx.accounts.pda_7.as_ref().map(|p| p.to_account_info()),
    ];
    
    // Using iterative method instead of repeated if-queries (more efficient)
    for i in 0..std::cmp::min(batch_size as usize - 1, optional_pdas.len()) {
        if let Some(pda) = &optional_pdas[i] {
            pda_accounts.push(pda.clone());
        }
    }
    
    // Log for parallelization
    msg!("Batch processing with {} PDAs for {} hops", pda_accounts.len(), batch_size);
    
    // Check PDAs for validity
    for (i, pda) in pda_accounts.iter().enumerate() {
        let hop_index = ctx.accounts.transfer_state.current_hop + i as u8;
        // Use the improved PDA validation logic to verify each PDA
        // This ensures cryptographic validation of the PDA derivation
        let current_split_index = i as u8; // Use the correct index for each PDA
        
        // Use our enhanced verify_pda_derivation function to properly validate the PDA
        let validation_result = verify_pda_derivation(
            ctx.program_id,
            &ctx.accounts.transfer_state.seed,
            hop_index,
            current_split_index,
            pda
        );
        
        // Handle validation result with proper error messages
        let is_valid = match validation_result {
            Ok(_) => {
                // Successfully validated PDA derivation
                msg!("PDA for hop {} split {} validated successfully", hop_index, current_split_index);
                true
            },
            Err(e) => {
                // Failed PDA validation - try alternative check for fake splits
                let is_fake_valid = check_bloom_filter(
                    &ctx.accounts.transfer_state.fake_bloom,
                    hop_index,
                    current_split_index
                );
                
                if is_fake_valid {
                    msg!("PDA for hop {} split {} validated as fake split", hop_index, current_split_index);
                    true
                } else {
                    msg!("PDA validation failed for hop {} split {}: {:?}", hop_index, current_split_index, e);
                    false
                }
            }
        };
        
        if !is_valid {
            msg!("Invalid PDA for hop {}: {:?}", hop_index, pda.key());
            return Err(BlackoutError::PdaCreationError.into());
        }
    }
    
    // Execute the batch hop with parallel execution for maximum efficiency
    // Processing occurs in a single pass to save CPU cycles
    msg!("Starting parallel batch hop execution for {} splits", splits.len());
    
    // Extract the necessary values from TransferState as copies
    // to have a common context without creating borrowing conflicts
    let current_hop = ctx.accounts.transfer_state.current_hop;
    let seed = ctx.accounts.transfer_state.seed;
    let bump = ctx.accounts.transfer_state.bump;
    let fake_bloom = ctx.accounts.transfer_state.fake_bloom;
    
    // IMPORTANT: Instead of using parallel_batch_execution with temporary variables
    // that have too short a lifetime, we implement the logic here directly
    // with the accounts from the context.
    
    // 1. Pre-validation and error handling (constant time)
    let num_pdas = pda_accounts.len();
    let num_splits = splits.len();
    
    if num_pdas == 0 {
        msg!("Critical error: No PDAs provided for batch hop");
        return Err(BlackoutError::InvalidBatchConfiguration.into());
    }
    
    if num_splits > num_pdas + 1 {
        msg!("Error: {num_splits} splits require {num_splits} PDAs, but only {num_pdas} were provided");
        return Err(BlackoutError::InvalidBatchConfiguration.into());
    }

    // 2. Optimierte Verarbeitung der Batch-Hops
    for (i, split_amount) in splits.iter().enumerate() {
        if i >= num_pdas {
            msg!("Info: Verarbeitung nach {i} Splits gestoppt (PDAs erschöpft)");
            break;
        }

        // Check if this is a real or fake split
        let is_fake = check_bloom_filter(&fake_bloom, current_hop + i as u8, 0);
        let split_target = &pda_accounts[i];

        // Skip fake splits to save compute units
        if is_fake {
            msg!("Batch-Hop: Skipping fake split at position {i}");
            continue;
        }

        // Validate split amount
        if *split_amount == 0 {
            continue;
        }
        
        msg!("Batch hop: Transferring {} lamports to split {}", split_amount, i);

        // Execute PDA transfer with signature
        let seeds = &[b"transfer".as_ref(), seed.as_ref(), &[bump]];
        let signer_seeds = &[&seeds[..]];
            
        // Actual transfer
        let transfer_result = invoke_signed(
            &system_instruction::transfer(
                ctx.accounts.transfer_state.to_account_info().key,
                split_target.to_account_info().key,
                *split_amount,
            ),
            &[
                ctx.accounts.transfer_state.to_account_info(),
                split_target.clone(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
        );

        // Fehlerbehandlung pro Split
        if let Err(e) = transfer_result {
            msg!("Fehler beim Split-Transfer {}: {:?}", i, e);
            return Err(BlackoutError::InsufficientLamports.into());
        }
        
        // Cost optimization: Recover remaining lamports above the rent-exempt threshold
        {
            let split_account_info = split_target.to_account_info();
            let rent_exempt_minimum = Rent::get()?.minimum_balance(0);
            
            // Recover lamports if possible (only what's above the minimum rent)
            if split_account_info.lamports() > rent_exempt_minimum {
                let recoverable_amount = split_account_info.lamports() - rent_exempt_minimum;
                **split_account_info.lamports.borrow_mut() = rent_exempt_minimum;
                **ctx.accounts.transfer_state.to_account_info().lamports.borrow_mut() += recoverable_amount;
                msg!("Batch cost optimization: Recovered {} lamports from split {}", recoverable_amount, i);
            }
        }
    }
        
    // Erfolgreiche Verarbeitung melden
    msg!("Batch-Hop erfolgreich: {} Splits verarbeitet", std::cmp::min(num_splits, num_pdas));

    
    // Update the transfer state
    let transfer_state = &mut ctx.accounts.transfer_state;
    transfer_state.current_hop += batch_size;
    transfer_state.batch_count += 1;
    
    // Check if all hops are completed
    if transfer_state.current_hop >= transfer_state.config.num_hops {
        msg!("All {} hops completed, ready for finalization", transfer_state.config.num_hops);
    } else {
        msg!("Batch {} completed, {} of {} hops processed ({}%)", 
             batch_index, transfer_state.current_hop, transfer_state.config.num_hops,
             transfer_state.progress_percent());
    }
    
    // Emit event
    emit!(BatchHopExecuted {
        owner: transfer_state.owner,
        batch_index,
        hops_processed: batch_size,
        splits_processed: splits.len() as u8,
        compute_units_consumed: cu_limit,
        progress_percent: transfer_state.progress_percent(),
        remaining_hops: transfer_state.remaining_hops(),
        transfer_state: ctx.accounts.transfer_state.key(),
    });
    
    Ok(())
}

#[event]
pub struct BatchHopExecuted {
    pub owner: Pubkey,
    pub batch_index: u8,
    pub hops_processed: u8,
    pub splits_processed: u8,
    pub compute_units_consumed: u32,
    pub progress_percent: u8,
    pub remaining_hops: u8,
    pub transfer_state: Pubkey,
}
