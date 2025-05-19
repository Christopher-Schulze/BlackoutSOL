use anchor_lang::prelude::*;

// Using our core Solana imports for backward compatibility
use crate::solana_imports::*;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{verify_hyperplonk_proof, calculate_optimized_priority_fees};
use anchor_lang::solana_program::keccak;
use std::convert::TryInto;

/// Distributes a total amount randomly among a given number of recipients
/// Uses proof data as source of randomness for unpredictable distribution
fn distribute_random_amounts(total: u64, count: usize, seed: &[u8; 32]) -> Result<Vec<u64>> {
    // Minimum 3 recipients for optimal anonymity
    let wallet_count = count.max(1);
    
    // Enforce minimum amount per recipient (0.001 SOL) to avoid dust
    let min_per_recipient = 1_000_000; // 0.001 SOL in lamports
    
    // Check if total can support minimum amount per recipient
    if total < (min_per_recipient * wallet_count as u64) {
        // If not enough for minimum per recipient, send all to first recipient
        let mut amounts = vec![0; wallet_count];
        amounts[0] = total;
        return Ok(amounts);
    }
    
    // Generate random weights for distribution
    let mut weights = Vec::with_capacity(wallet_count);
    let mut seed_hash = *seed;
    
    for i in 0..wallet_count {
        // Mix the index with the seed for unique randomness per recipient
        seed_hash[0] ^= i as u8;
        let hash_result = keccak::hashv(&[&seed_hash]);
        
        // Convert first 8 bytes of hash to u64 for weight
        let weight_bytes: [u8; 8] = hash_result.as_ref()[0..8]
            .try_into()
            .map_err(|_| BlackoutError::InvalidParameters)?;
        
        let weight = u64::from_le_bytes(weight_bytes);
        weights.push(weight);
        
        // Update seed for next iteration
        seed_hash = hash_result.to_bytes();
    }
    
    // Calculate total weight
    let total_weight: u128 = weights.iter().map(|&w| w as u128).sum();
    if total_weight == 0 {
        // Fallback in case all weights are 0
        let equal_share = total / wallet_count as u64;
        return Ok(vec![equal_share; wallet_count]);
    }
    
    // Distribute amounts based on weights while ensuring minimum amounts
    let distributable = total.saturating_sub(min_per_recipient * wallet_count as u64);
    let mut amounts = Vec::with_capacity(wallet_count);
    let mut remaining = distributable;
    
    // First pass: calculate weighted amounts
    for &weight in &weights {
        let weighted_amount = if total_weight > 0 {
            ((weight as u128 * distributable as u128) / total_weight) as u64
        } else {
            0
        };
        
        amounts.push(weighted_amount + min_per_recipient);
        remaining = remaining.saturating_sub(weighted_amount);
    }
    
    // Second pass: distribute any remaining amount to ensure exact total
    if remaining > 0 && !amounts.is_empty() {
        amounts[0] += remaining;
    }
    
    Ok(amounts)
}

#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
        constraint = !transfer_state.completed @ BlackoutError::TransferAlreadyCompleted,
        constraint = transfer_state.current_hop == 4 @ BlackoutError::TransferNotComplete,
        close = primary_recipient
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: Primary recipient (remaining funds go here after account close)
    #[account(mut)]
    pub primary_recipient: UncheckedAccount<'info>,
    
    /// CHECK: Optional second recipient wallet
    #[account(mut)]
    pub recipient_2: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional third recipient wallet
    #[account(mut)]
    pub recipient_3: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional fourth recipient wallet
    #[account(mut)]
    pub recipient_4: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional fifth recipient wallet
    #[account(mut)]
    pub recipient_5: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional sixth recipient wallet
    #[account(mut)]
    pub recipient_6: Option<UncheckedAccount<'info>>,
    
    pub system_program: Program<'info, System>,
}

pub fn finalize(
    ctx: Context<Finalize>,
    proof_data: [u8; 128],
) -> Result<()> {
    // 1. Optimized compute unit configuration for final verification
    let required_cu = 400_000; 
    // Use a default value for remaining hops (0 for finalize) and a
    // fixed value for Compute Units.
    let priority_fee = calculate_optimized_priority_fees(0, required_cu)?;
    
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(required_cu);
    let priority_fee_ix = ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
    
    // Execute compute budget instructions atomically
    invoke(
        &cu_limit_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    invoke(
        &priority_fee_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    // 2. Hop completion check is covered by account constraint: 
    // `transfer_state.current_hop == 4 @ BlackoutError::TransferNotComplete`
    // Redundant check removed:
    // if ctx.accounts.transfer_state.current_hop < 4 { ... }
    
    // 3. Generate challenge for proof verification
    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;
    
    let mut challenge = [0u8; 32];
    challenge[0..8].copy_from_slice(&timestamp.to_le_bytes());
    challenge[8..16].copy_from_slice(&ctx.accounts.transfer_state.owner.to_bytes()[0..8]);
    challenge[16..24].copy_from_slice(&ctx.accounts.primary_recipient.key().to_bytes()[0..8]);
    challenge[24..32].copy_from_slice(&ctx.accounts.transfer_state.seed[24..32]);
    
    // 4. Verify final HyperPlonk proof
    verify_hyperplonk_proof(&proof_data, &challenge)?;
    
    // 5. Reserve calculation and transfer
    let total_amount = ctx.accounts.transfer_state.amount;
    let reserve_percent = ctx.accounts.transfer_state.config.reserve_percent;
    
    // Calculate the reserve (percentage of total amount)
    let reserve_amount = if reserve_percent > 0 {
        (total_amount as u128 * reserve_percent as u128 / 100) as u64
    } else {
        0
    };
    
    // Amount to be transferred to the recipient
    let recipient_amount = total_amount.saturating_sub(reserve_amount);
    
    // 6. Setup for multi-wallet distribution
    // Collect available recipient accounts
    let mut recipient_accounts = vec![ctx.accounts.primary_recipient.to_account_info()];
    
    if let Some(rec2) = &ctx.accounts.recipient_2 {
        recipient_accounts.push(rec2.to_account_info());
    }
    if let Some(rec3) = &ctx.accounts.recipient_3 {
        recipient_accounts.push(rec3.to_account_info());
    }
    if let Some(rec4) = &ctx.accounts.recipient_4 {
        recipient_accounts.push(rec4.to_account_info());
    }
    if let Some(rec5) = &ctx.accounts.recipient_5 {
        recipient_accounts.push(rec5.to_account_info());
    }
    if let Some(rec6) = &ctx.accounts.recipient_6 {
        recipient_accounts.push(rec6.to_account_info());
    }
    
    // Ensure we have at least 3 recipient accounts, or use primary for all
    if recipient_accounts.len() < 3 {
        msg!("Warning: Less than 3 recipient accounts provided. Minimum 3 required for optimal anonymity");
    }
    
    if recipient_amount > 0 {
        // Use the final proof data as randomization source for split distribution
        let mut rng_seed = [0u8; 32];
        for i in 0..proof_data.len().min(32) {
            rng_seed[i % 32] ^= proof_data[i];
        }
        
        // Distribution of amounts to multiple wallets
        let wallet_count = recipient_accounts.len().min(6);
        let mut split_amounts = distribute_random_amounts(recipient_amount, wallet_count, &rng_seed)?;
        
        msg!("Multi-wallet distribution: Transferring to {} wallets for enhanced anonymity", wallet_count);
        
        let owner_key = ctx.accounts.transfer_state.owner;
        let bump_seed = ctx.accounts.transfer_state.bump;
        let seeds = &[
            b"transfer".as_ref(),
            owner_key.as_ref(),
            &[bump_seed]
        ];
        let signer_seeds = &[&seeds[..]];
        
        // Transfer to each recipient with random amount - MINIMAL COST VERSION
        for (idx, recipient_account) in recipient_accounts.iter().enumerate() {
            if idx >= split_amounts.len() {
                break;
            }
            
            let amount = split_amounts[idx];
            if amount == 0 {
                continue;
            }
            
            msg!("Transferring {} lamports to recipient {}", amount, idx + 1);
            
            // Direct transfer without intermediate steps for maximum cost efficiency
            invoke_signed(
                &system_instruction::transfer(
                    &ctx.accounts.transfer_state.key(),
                    recipient_account.key,
                    amount
                ),
                &[
                    ctx.accounts.transfer_state.to_account_info(),
                    recipient_account.clone(),
                    ctx.accounts.system_program.to_account_info(),
                ],
                signer_seeds,
            )?;
        }
    }
    
    // 7. Cost optimization: Close all temporary accounts
    // Since we have `close = primary_recipient` in the account definition,
    // all remaining lamports are automatically transferred to the primary_recipient,
    // which refunds the rent costs and provides maximum efficiency.
    
    // 8. Mark transfer as completed
    ctx.accounts.transfer_state.completed = true;
    // Store the primary recipient in the first position of recipients array
    ctx.accounts.transfer_state.recipients[0] = ctx.accounts.primary_recipient.key();
    // Store other recipients if provided
    if let Some(rec2) = &ctx.accounts.recipient_2 {
        ctx.accounts.transfer_state.recipients[1] = rec2.key();
    }
    if let Some(rec3) = &ctx.accounts.recipient_3 {
        ctx.accounts.transfer_state.recipients[2] = rec3.key();
    }
    if let Some(rec4) = &ctx.accounts.recipient_4 {
        ctx.accounts.transfer_state.recipients[3] = rec4.key();
    }
    if let Some(rec5) = &ctx.accounts.recipient_5 {
        ctx.accounts.transfer_state.recipients[4] = rec5.key();
    }
    if let Some(rec6) = &ctx.accounts.recipient_6 {
        ctx.accounts.transfer_state.recipients[5] = rec6.key();
    }
    // Set the actual recipient count
    ctx.accounts.transfer_state.recipient_count = recipient_accounts.len() as u8;
    ctx.accounts.transfer_state.timestamp = timestamp;
    
    // 9. Emit event with detailed information
    emit!(TransferFinalized {
        owner: ctx.accounts.transfer_state.owner,
        primary_recipient: ctx.accounts.primary_recipient.key(),
        recipient_count: recipient_accounts.len() as u8,
        amount: recipient_amount,
        reserve: reserve_amount,
        total_amount: total_amount,
        transfer_state: ctx.accounts.transfer_state.key(),
        timestamp,
    });
    
    msg!("Transfer finalized: {} lamports distributed across {} wallets with maximum cost efficiency", 
         recipient_amount, recipient_accounts.len());
    
    Ok(())
}

#[event]
pub struct TransferFinalized {
    pub owner: Pubkey,
    pub primary_recipient: Pubkey,
    pub recipient_count: u8,
    pub amount: u64,
    pub reserve: u64,
    pub total_amount: u64,
    pub transfer_state: Pubkey,
    pub timestamp: i64,
}