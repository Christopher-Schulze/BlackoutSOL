use anchor_lang::prelude::*;
use anchor_lang::solana_program::sysvar::clock::Clock;
use anchor_lang::solana_program::program_error::ProgramError;
use anchor_lang::solana_program::bs58;
use anchor_spl::token::{self, Token, TokenAccount};

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{
    verify_hyperplonk_proof,
    verify_range_proof,
    generate_bloom_filter,
    calculate_fees,
    TransferContext,
    info,
    error,
};

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{
    verify_hyperplonk_proof,
    verify_range_proof,
    generate_bloom_filter,
    calculate_fees,
    TransferContext,
    info,
    error,
};

/// Context for initializing an anonymous transfer
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(
        init,
        payer = payer,
        space = TransferState::SIZE,
        seeds = [b"transfer", payer.key().as_ref()],
        bump
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: Primary recipient for the final payment
    pub primary_recipient: UncheckedAccount<'info>,
    
    /// CHECK: Optional second recipient wallet
    pub recipient_2: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional third recipient wallet
    pub recipient_3: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional fourth recipient wallet
    pub recipient_4: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional fifth recipient wallet
    pub recipient_5: Option<UncheckedAccount<'info>>,
    
    /// CHECK: Optional sixth recipient wallet
    pub recipient_6: Option<UncheckedAccount<'info>>,
    
    /// CHECK: This is the Merkle root for wallet set verification
    #[account()]
    pub merkle_root_account: UncheckedAccount<'info>,
    
    /// System program
    pub system_program: Program<'info, System>,
    
    /// Clock sysvar
    pub clock: Sysvar<'info, Clock>,
    
    /// Transfer configuration
    /// Transfer configuration
    pub transfer_config: Account<'info, TransferConfig>,
    
    /// Token program for SPL token transfers
    pub token_program: Program<'info, Token>,
    
    /// Token account for the payer
    #[account(mut)]
    pub payer_token_account: Account<'info, TokenAccount>,
    
    /// Token account for the recipient
    #[account(mut)]
    pub recipient_token_account: Account<'info, TokenAccount>,
}

pub fn initialize(
    ctx: Context<Initialize>,
    amount: u64,
    hyperplonk_proof: [u8; 128],
    range_proof: [u8; 128],
    challenge: [u8; 32],
    _merkle_proof: Vec<u8>,
) -> Result<()> {
    // Set compute unit limit
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(400_000);
    invoke(
        &cu_limit_ix,
        &[ctx.accounts.payer.to_account_info()],
    )?;
    
    // Old compute unit check removed as it's superseded by a later check and used an outdated API.
    
    // Basic validation of the amount
    if amount == 0 {
        msg!("Amount must be greater than 0");
        return Err(BlackoutError::InvalidAmount.into());
    }
    
    // Check if sender and recipient are different
    if ctx.accounts.payer.key() == ctx.accounts.primary_recipient.key() {
        msg!("Sender and recipient must be different");
        return Err(BlackoutError::InvalidRecipient.into());
    }
    
    // Use fixed Blackout configuration
    let config = BlackoutConfig::new();
    
    // Check challenge data
    if challenge == [0; 32] {
        msg!("Invalid challenge data");
        return Err(BlackoutError::InvalidChallenge.into());
    }
    
    // Extract Merkle root for wallet set
    let mut merkle_root = [0u8; 32];
    merkle_root.copy_from_slice(&ctx.accounts.merkle_root_account.key().to_bytes()[0..32]);
    
    // Store bump for the PDA - adjusted for Anchor 0.29.0
    // In newer Anchor versions, `bumps` is a HashMap-like object
    let bump = ctx.bumps.transfer_state;
    
    // Generate seed for stealth PDAs (deterministic from challenge + payer)
    let seed = Pubkey::find_program_address(
        &[
            b"blackout",
            &challenge,
            ctx.accounts.payer.key().as_ref(),
        ],
        ctx.program_id,
    ).0.to_bytes();
    
    // Generate bloom filter for fake splits
    let fake_bloom = generate_bloom_filter(&config, &challenge);
    
    // Dummy commitments (will be set later)
    let commitments = [[0; 32]; 8];
    
    // Validate HyperPlonk proof with extended log data
    msg!("Validating HyperPlonk proof for anonymous transfers...");
    // Create transfer context for logging
    let transfer_ctx = TransferContext::new(
        &bs58::encode(&challenge).into_string(),
        amount,
    );
    
    // Verify the hyperplonk proof
    info!("Verifying HyperPlonk proof...");
    if !verify_hyperplonk_proof(&hyperplonk_proof, &challenge).map_err(|e| {
        error!("Hyperplonk proof verification failed: {}", e);
        BlackoutError::InvalidProof
    })? {
        transfer_ctx.error(&"Hyperplonk proof verification failed", "Initialize");
        return Err(BlackoutError::InvalidProof.into());
    }
    info!("HyperPlonk proof verified successfully");
    
    // Verify the range proof
    info!("Verifying range proof...");
    if !verify_range_proof(&range_proof, amount).map_err(|e| {
        error!("Range proof verification failed: {}", e);
        BlackoutError::InvalidRangeProof
    })? {
        transfer_ctx.error(&"Range proof verification failed", "Initialize");
        return Err(BlackoutError::InvalidRangeProof.into());
    }
    info!("Range proof verified successfully");
    
    // Calculate fees
    let fee_rate = ctx.accounts.transfer_config.fee_rate as u64;
    let fee_amount = calculate_fees(amount, fee_rate);
    let net_amount = amount.checked_sub(fee_amount).ok_or_else(|| {
        let msg = format!(
            "Arithmetic overflow calculating net amount: {} - {}",
            amount, fee_amount
        );
        transfer_ctx.error(&msg, "Initialize");
        BlackoutError::ArithmeticOverflow
    })?;
    
    // Verify the fee amount is within acceptable range
    if fee_amount < ctx.accounts.transfer_config.min_transfer_amount {
        let msg = format!(
            "Fee amount {} is below minimum required {}",
            fee_amount, ctx.accounts.transfer_config.min_transfer_amount
        );
        transfer_ctx.error(&msg, "Initialize");
        return Err(BlackoutError::InsufficientFunds.into());
    }
    
    info!("Calculated fees: {} ({}% of {})", fee_amount, fee_rate as f64 / 100.0, amount);
    
    // Total amount to transfer (real transaction + reserve + fees)
    let total_amount = amount + fee_amount;
    
    // Check if the payer has enough lamports
    if ctx.accounts.payer.lamports() < total_amount {
        msg!("Insufficient lamports: {} < {}", ctx.accounts.payer.lamports(), total_amount);
        return Err(BlackoutError::InsufficientLamports.into());
    }
    
    // Initialize transfer state with the extended data
    let transfer_state = &mut ctx.accounts.transfer_state;
    
    // Initialize TransferState with double dereferencing to get from Account<'_, TransferState> type to TransferState type
    **transfer_state = TransferState::new(
        ctx.accounts.payer.key(),
        amount,
        seed,
        bump,
        vec![],
        0,
        config,
        hyperplonk_proof,
        range_proof,
        challenge,
        merkle_root,
        fake_bloom,
        ctx.accounts.clock.unix_timestamp, // Pass current timestamp
    );
    
    // Store fees and reserve
    transfer_state.total_fees = total_fee;
    transfer_state.reserve = reserve;
    
    // Deposit lamports into the transfer state
    let transfer_ix = system_instruction::transfer(
        &ctx.accounts.payer.key(),
        &ctx.accounts.transfer_state.key(),
        total_amount,
    );
    invoke(
        &transfer_ix,
        &[
            ctx.accounts.payer.to_account_info(),
            ctx.accounts.transfer_state.to_account_info(),
            ctx.accounts.system_program.to_account_info(),
        ],
    )?;
    
    // 4. Check remaining compute units (optional, for debugging)
    // This check can be removed in production to save compute units.
    // It helps ensure that the transaction has enough CUs left after the main logic.
    // CU-Monitor und Logging für Optimierung
    let remaining_cus = sol_remaining_compute_units();
    const MIN_REMAINING_CUS_THRESHOLD: u64 = 20_000; // Example threshold, adjust as needed
    if remaining_cus < MIN_REMAINING_CUS_THRESHOLD { 
        msg!("Warning: Low CUs after initialization. Remaining: {}. Threshold: {}", 
             remaining_cus, MIN_REMAINING_CUS_THRESHOLD);
    }
    
    // 5. Emit event for successful initialization
    msg!("Transfer initialized: {} lamports, 4 hops, 4 real splits, 44 fake splits", 
         amount);
    msg!("Fees: {} lamports, Reserve: {} lamports", total_fee, reserve);
    msg!("Total possible: {} paths", transfer_config.total_paths());
    
    // Collect recipient addresses
    let mut recipients = vec![ctx.accounts.primary_recipient.key()];
    
    // Add optional recipients if they exist
    if let Some(recipient) = &ctx.accounts.recipient_2 {
        recipients.push(recipient.key());
    }
    if let Some(recipient) = &ctx.accounts.recipient_3 {
        recipients.push(recipient.key());
    }
    if let Some(recipient) = &ctx.accounts.recipient_4 {
        recipients.push(recipient.key());
    }
    if let Some(recipient) = &ctx.accounts.recipient_5 {
        recipients.push(recipient.key());
    }
    if let Some(recipient) = &ctx.accounts.recipient_6 {
        recipients.push(recipient.key());
    }
    
    let recipient_count = recipients.len() as u8;
    
    // Transfer tokens
    let transfer_ix = anchor_spl::token::Transfer {
        from: ctx.accounts.payer_token_account.to_account_info(),
        to: ctx.accounts.recipient_token_account.to_account_info(),
        authority: ctx.accounts.payer.to_account_info(),
    };
    
    let cpi_ctx = CpiContext::new(
        ctx.accounts.token_program.to_account_info(),
        transfer_ix,
    );
    
    // Execute the token transfer
    anchor_spl::token::transfer(cpi_ctx, net_amount)?;
    
    // Emit event
    let clock = Clock::get()?;
    emit!(TransferInitialized {
        transfer_id: transfer_state.transfer_id,
        primary_recipient: ctx.accounts.primary_recipient.key(),
        recipient_count,
        amount: net_amount,
        total_amount: amount,
        fee_amount,
        timestamp: clock.unix_timestamp,
    });
    
    // Update transfer state
    transfer_state.initialize(
        transfer_state.transfer_id,
        ctx.accounts.payer.key(),
        ctx.accounts.primary_recipient.key(),
        net_amount,
        fee_amount,
        *ctx.bumps.get("transfer_state").ok_or(BlackoutError::InvalidStateTransition)?,
        &clock,
    );
    
    info!(
        "Transfer initialized: id={}, amount={}, recipients={}",
        transfer_state.transfer_id, net_amount, recipient_count
    );
    
    info!(
        "Transfer initialized: id={}, amount={}, recipients={}",
        transfer_state.transfer_id, amount, recipient_count
    );
    
    Ok(())
}

#[event]
pub struct TransferInitialized {
    /// Unique identifier for this transfer
    pub transfer_id: u64,
    
    /// Primary recipient of the transfer
    pub primary_recipient: Pubkey,
    
    /// Total number of recipients
    pub recipient_count: u8,
    
    /// Amount being transferred (in lamports)
    pub amount: u64,
    
    /// Total amount including fees
    pub total_amount: u64,
    
    /// Fee amount (in lamports)
    pub fee_amount: u64,
    
    /// Timestamp when the transfer was initialized
    pub timestamp: i64,
}