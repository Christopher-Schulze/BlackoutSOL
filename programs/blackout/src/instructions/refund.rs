use anchor_lang::prelude::*;
use anchor_lang::solana_program::clock::Clock;
use anchor_lang::solana_program::sysvar::Sysvar;

// Using our central Solana imports for compatibility with Solana 1.18.26
use crate::solana_imports::*;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::calculate_optimized_priority_fees;

/// Context for a refund in case of errors
/// 
/// This instruction allows cancelling a transfer and returning funds
/// to the sender if something went wrong.
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        mut,
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
        constraint = !transfer_state.refund_triggered @ BlackoutError::RefundAlreadyTriggered,
        close = owner
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: This is the original owner of the transfer
    #[account(
        mut,
        constraint = owner.key() == transfer_state.owner @ BlackoutError::UnauthorizedAccess
    )]
    pub owner: UncheckedAccount<'info>,
    
    /// CHECK: This is the recipient of the DEV share
    #[account(mut)]
    pub dev_account: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn refund(ctx: Context<Refund>) -> Result<()> {
    // 1. Optimized compute budget for the refund process
    let required_cu = 250_000; // Slightly increased for additional security checks
    let priority_fee = calculate_optimized_priority_fees(0, required_cu)?; // Priority for fast refunds, 0 hops
    
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
    
    // 2. Validation of transfer state
    if ctx.accounts.transfer_state.completed {
        msg!("Refund failed: Transfer already completed");
        return Err(BlackoutError::TransferAlreadyCompleted.into());
    }
    
    // Redundant check for refund_triggered removed, as it's covered by account constraint.
    // if ctx.accounts.transfer_state.refund_triggered { ... }
    
    // 3. Calculation of refund amounts
    // The percentage for refunds (95% goes back to the sender)
    let refund_percentage = 95;
    let dev_percentage = 100 - refund_percentage; // 5% for developers
    
    // Total amount from the transfer state
    let total_amount = ctx.accounts.transfer_state.amount;
    
    // Exact calculation of amounts with overflow protection
    let refund_amount = (total_amount as u128 * refund_percentage as u128 / 100) as u64;
    let dev_amount = total_amount.saturating_sub(refund_amount); // Remaining amount
    
    // 4. Check available lamports
    let available_lamports = ctx.accounts.transfer_state.to_account_info().lamports();
    
    if available_lamports < refund_amount.saturating_add(dev_amount) {
        msg!("Insufficient lamports for refund: {} < {}", 
             available_lamports, refund_amount.saturating_add(dev_amount));
        return Err(BlackoutError::InsufficientLamports.into());
    }
    
    // 5. Comprehensive preparation of all required values and AccountInfo objects
    // before any mutable access (according to RuleX)
    let ts_progress = ctx.accounts.transfer_state.progress_percent();
    let ts_current_hop = ctx.accounts.transfer_state.current_hop;
    let ts_owner = ctx.accounts.transfer_state.owner;
    let ts_bump = ctx.accounts.transfer_state.bump;
    
    // Extract AccountInfo objects before any mutable transfer state access
    let ts_account_info = ctx.accounts.transfer_state.to_account_info();
    let ts_key = ts_account_info.key;
    let dev_account_info = ctx.accounts.dev_account.to_account_info();
    let system_program_info = ctx.accounts.system_program.to_account_info();
    
    // Now we can create the mutable reference
    let transfer_state = &mut ctx.accounts.transfer_state;
    
    msg!("Refunding {} lamports to the owner ({}% of total amount)", 
         refund_amount, refund_percentage);
    msg!("Transfer status: Hop {} of 4, {}% completed", ts_current_hop, ts_progress);
    
    // 6. Collect timestamp for tracking
    let clock = Clock::get()?;
    let timestamp = clock.unix_timestamp;
    
    // 7. Transfer DEV share (if any)
    if dev_amount > 0 {
        // Minimum amount check
        let min_dev_amount = 1000; // 0.000001 SOL minimum amount
        let actual_dev_amount = if dev_amount < min_dev_amount { 0 } else { dev_amount };
        
        if actual_dev_amount > 0 {
            msg!("Sending {} lamports DEV share ({} % fee)", 
                 actual_dev_amount, dev_percentage);
            
            let seeds = &[
                b"transfer".as_ref(), 
                ts_owner.as_ref(), 
                &[ts_bump]
            ];
            let signer_seeds = &[&seeds[..]];

            // Transfer mit vorbereiteten AccountInfo-Objekten
            let dev_transfer_result = invoke_signed(
                &system_instruction::transfer(
                    ts_key,
                    dev_account_info.key,
                    actual_dev_amount,
                ),
                &[
                    ts_account_info,
                    dev_account_info,
                    system_program_info,
                ],
                signer_seeds,
            );
            
            // Continue on error, but log it. The owner will receive the untransferred DEV share 
            // when the transfer_state account is closed.
            if dev_transfer_result.is_err() {
                msg!("Warning: DEV share transfer failed. Untransferred DEV share will be returned to the owner via account closure.");
            }
        }
    }
    
    // 8. Mark transfer as refunded
    transfer_state.refund_triggered = true;
    transfer_state.timestamp = timestamp;
    
    // 9. Emit detailed event
    emit!(RefundExecuted {
        owner: ts_owner,
        refund_amount,
        dev_amount,
        total_amount,
        transfer_state: *ts_key,
        current_hop: ts_current_hop,
        progress_percent: ts_progress,
        timestamp,
    });
    
    // 10. Final log for audit
    msg!("Refund successfully completed: {} lamports returned to {} ({} hops executed)", 
         refund_amount, ts_owner, ts_current_hop);
    
    Ok(())
}

#[event]
pub struct RefundExecuted {
    pub owner: Pubkey,
    pub refund_amount: u64,
    pub dev_amount: u64,
    pub total_amount: u64,
    pub transfer_state: Pubkey,
    pub current_hop: u8,
    pub progress_percent: u8,
    pub timestamp: i64,
}
