use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program;
use anchor_lang::solana_program::system_instruction;
use anchor_lang::solana_program::sysvar::Sysvar;
use anchor_lang::solana_program::clock::Clock;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{verify_hyperplonk_proof, calculate_optimized_priority_fees};

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
        close = recipient
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: This is the recipient of the payment
    #[account(mut)]
    pub recipient: UncheckedAccount<'info>,
    
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
    
    let cu_limit_ix = solana_program::instruction::ComputeBudgetInstruction::set_compute_unit_limit(required_cu);
    let priority_fee_ix = solana_program::instruction::ComputeBudgetInstruction::set_compute_unit_price(priority_fee);
    
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
    challenge[16..24].copy_from_slice(&ctx.accounts.recipient.key().to_bytes()[0..8]);
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
    
    // 6. Transfer the main amount to the recipient
    if recipient_amount > 0 {
        msg!("Transferring {} lamports to recipient (Calculated Reserve: {} lamports)", 
             recipient_amount, reserve_amount);
        
        let transfer_ix = system_instruction::transfer(
            &ctx.accounts.transfer_state.key(),
            &ctx.accounts.recipient.key(),
            recipient_amount
        );
        
        let owner_key = ctx.accounts.transfer_state.owner;
        let bump_seed = ctx.accounts.transfer_state.bump;
        let seeds = &[
            b"transfer".as_ref(), 
            owner_key.as_ref(), 
            &[bump_seed]
        ];
        let signer_seeds = &[&seeds[..]];

        invoke_signed(
            &transfer_ix,
            &[
                ctx.accounts.transfer_state.to_account_info(),
                ctx.accounts.recipient.to_account_info(),
                ctx.accounts.system_program.to_account_info(),
            ],
            signer_seeds,
        )?;
    }
    
    // 7. Reserve amount handling clarification:
    // The `transfer_state` account is marked with `close = recipient`.
    // This means any lamports remaining in `transfer_state` after the explicit
    // transfer of `recipient_amount` (i.e., the calculated `reserve_amount`)
    // will also be transferred to `recipient` when the account is closed.
    // Effectively, the recipient receives the `total_amount`.
    
    // 8. Mark transfer as completed
    ctx.accounts.transfer_state.completed = true;
    ctx.accounts.transfer_state.recipient = ctx.accounts.recipient.key();
    ctx.accounts.transfer_state.timestamp = timestamp;
    
    // 9. Emit event with detailed information
    emit!(TransferFinalized {
        owner: ctx.accounts.transfer_state.owner,
        recipient: ctx.accounts.recipient.key(),
        amount: recipient_amount,
        reserve: reserve_amount,
        total_amount: total_amount,
        transfer_state: ctx.accounts.transfer_state.key(),
        timestamp,
    });
    
    msg!("Transfer successfully finalized: {} lamports transferred ({}% reserve)", 
         recipient_amount, reserve_percent);
    
    Ok(())
}

#[event]
pub struct TransferFinalized {
    pub owner: Pubkey,
    pub recipient: Pubkey,
    pub amount: u64,
    pub reserve: u64,
    pub total_amount: u64,
    pub transfer_state: Pubkey,
    pub timestamp: i64,
}