//! Defines the Anchor-compatible account structures for BlackoutSOL
//! 
//! These types are specifically designed for use with the Anchor Framework
//! and serve as wrappers for the corresponding structures in the main library.

use anchor_lang::prelude::*;
use blackout::state::transfer::TransferState;
use blackout::state::config::ConfigAccount;

/// Context for initializing an anonymous transfer
#[derive(Accounts)]
pub struct Initialize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    pub system_program: Program<'info, System>,
}

/// Context for executing a single hop
#[derive(Accounts)]
pub struct ExecuteHop<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for executing a batch hop
#[derive(Accounts)]
pub struct BatchHop<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    #[account(mut)]
    pub pda_1: AccountInfo<'info>,
    
    #[account(mut)]
    pub pda_2: Option<AccountInfo<'info>>,
    
    #[account(mut)]
    pub pda_3: Option<AccountInfo<'info>>,
    
    #[account(mut)]
    pub pda_4: Option<AccountInfo<'info>>,
    
    #[account(mut)]
    pub pda_5: Option<AccountInfo<'info>>,
    
    #[account(mut)]
    pub pda_6: Option<AccountInfo<'info>>,
    
    #[account(mut)]
    pub pda_7: Option<AccountInfo<'info>>,
    
    pub system_program: Program<'info, System>,
}

/// Context for finalizing a transfer
#[derive(Accounts)]
pub struct Finalize<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    #[account(mut)]
    pub recipient: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for configuration updates
#[derive(Accounts)]
pub struct ConfigUpdate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    #[account(mut)]
    pub config: Account<'info, ConfigAccount>,
    
    pub system_program: Program<'info, System>,
}

/// Context for refunds
#[derive(Accounts)]
pub struct Refund<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    #[account(mut)]
    pub owner: AccountInfo<'info>,
    
    #[account(mut)]
    pub dev_account: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}

/// Context for revealing a fake split
#[derive(Accounts)]
pub struct RevealFake<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,
    
    #[account(mut)]
    pub transfer_state: Account<'info, TransferState>,
    
    #[account(mut)]
    pub split_pda: AccountInfo<'info>,
    
    pub system_program: Program<'info, System>,
}
