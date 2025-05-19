use anchor_lang::prelude::*;
use anchor_lang::solana_program::program::invoke;
use anchor_lang::solana_program;

use crate::state::*;
use crate::errors::BlackoutError;
use crate::utils::{check_bloom_filter, derive_stealth_pda};

/// Context for revealing a fake split address
/// 
/// This instruction allows proving that a specific
/// stealth PDA is marked as a fake split. This serves
/// to verify the system and secure it against attacks.
#[derive(Accounts)]
pub struct RevealFake<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    /// CHECK: This is the PDA to be revealed as a fake split
    pub fake_pda: UncheckedAccount<'info>,
    
    pub system_program: Program<'info, System>,
}

pub fn reveal_fake(
    ctx: Context<RevealFake>,
    hop_index: u8,
    split_index: u8,
) -> Result<()> {
    // Set compute limit
    // Note: Revealing fakes is a diagnostic tool, use a fixed CU limit
    let cu_limit_ix = solana_program::instruction::ComputeBudgetInstruction::set_compute_unit_limit(200_000);
    invoke(
        &cu_limit_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    // Check if the hop index is valid (0-3 for 4 hops)
    if hop_index >= 4 {
        msg!("Invalid hop index: {} (must be between 0 and 3)", hop_index);
        return Err(BlackoutError::InvalidHopIndex.into());
    }
    
    // Check if the split index is valid (0-47 for 4 real + 44 fake splits)
    if split_index >= 48 { // 4 + 44 = 48 splits total
        msg!("Invalid split index: {} (must be between 0 and 47)", split_index);
        return Err(BlackoutError::InvalidParameters.into());
    }
    
    // Check if the index is in the fake split range (4-47)
    if split_index < 4 {
        msg!("Split index {} is a real split, must be between 4 and 47 for fake splits", split_index);
        return Err(BlackoutError::InvalidParameters.into());
    }
    
    // Check if the split is marked as fake in the bloom filter
    let is_fake = check_bloom_filter(
        &ctx.accounts.transfer_state.fake_bloom,
        hop_index,
        split_index,
    );
    
    if !is_fake {
        msg!("Split is not marked as fake in the bloom filter");
        return Err(BlackoutError::BloomFilterError.into());
    }
    
    // Calculate the expected PDA for the fake split
    let (expected_pda, _) = derive_stealth_pda(
        ctx.program_id,
        &ctx.accounts.transfer_state.seed,
        hop_index,
        split_index,
        true, // Fake split
    );
    
    // Check if the provided PDA matches the expected one
    if ctx.accounts.fake_pda.key() != expected_pda {
        msg!("PDA does not match the expected fake split PDA");
        return Err(BlackoutError::InvalidPda.into());
    }
    
    // Proof successfully provided - the PDA is a fake split
    msg!("Successful verification: PDA for hop {} and split {} is a fake split",
         hop_index, split_index);
    
    // Emit event
    emit!(FakeRevealed {
        owner: ctx.accounts.transfer_state.owner,
        hop_index,
        split_index,
        fake_pda: ctx.accounts.fake_pda.key(),
        transfer_state: ctx.accounts.transfer_state.key(),
    });
    
    Ok(())
}

#[event]
pub struct FakeRevealed {
    pub owner: Pubkey,
    pub hop_index: u8,
    pub split_index: u8,
    pub fake_pda: Pubkey,
    pub transfer_state: Pubkey,
}
