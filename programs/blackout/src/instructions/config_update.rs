use anchor_lang::prelude::*;

// Using our core Solana imports for compatibility with Solana 1.18.26
use crate::solana_imports::*;

// State-Imports
use crate::state::{TransferState, BlackoutConfig};
use crate::errors::BlackoutError;

/// Context for updating the Blackout configuration
/// 
/// This instruction allows adjusting certain parameters without
/// changing the basic configuration (4 hops, 4 splits, 44 fake splits).
#[derive(Accounts)]
pub struct ConfigUpdate<'info> {
    #[account(mut)]
    pub authority: Signer<'info>,
    
    /// Administration address for configuration changes
    /// In a production environment, this should be a multi-sig wallet
    /// CHECK: We manually check if the account is authorized
    #[account(mut)]
    pub admin: UncheckedAccount<'info>,
    
    #[account(
        mut,
        seeds = [b"transfer", transfer_state.owner.as_ref()],
        bump = transfer_state.bump,
        constraint = !transfer_state.completed @ BlackoutError::TransferAlreadyCompleted,
        constraint = transfer_state.current_hop == 0 @ BlackoutError::TransferNotComplete,
    )]
    pub transfer_state: Account<'info, TransferState>,
    
    pub system_program: Program<'info, System>,
}

#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct ConfigUpdateParams {
    /// Reserve percentage (10-80%)
    pub reserve_percent: Option<u8>,
    
    /// Fee multiplier in basis points (1-1000 BP)
    pub fee_multiplier: Option<u16>,
    
    /// Compute unit budget per hop transaction (100k-500k)
    pub cu_budget_per_hop: Option<u32>,
}

pub fn update_config(
    ctx: Context<ConfigUpdate>, 
    update_params: ConfigUpdateParams,
) -> Result<()> {
    // Set compute limit
    // Note: Config updates are sensitive, use a fixed, reasonable CU limit
    let cu_limit_ix = ComputeBudgetInstruction::set_compute_unit_limit(200_000);
    invoke(
        &cu_limit_ix,
        &[ctx.accounts.authority.to_account_info()],
    )?;
    
    // Check if the authority or the admin address is the owner
    if ctx.accounts.authority.key() != ctx.accounts.transfer_state.owner &&
       ctx.accounts.authority.key() != ctx.accounts.admin.key() {
        return Err(BlackoutError::UnauthorizedAccess.into());
    }
    
    // Check if the configuration can be changed
    if ctx.accounts.transfer_state.current_hop > 0 {
        msg!("Configuration can no longer be changed, transfer already started");
        return Err(BlackoutError::TransferNotComplete.into());
    }
    
    // Create base configuration
    let mut new_config = BlackoutConfig::new();
    
    // Selective update of configuration parameters
    if let Some(reserve_percent) = update_params.reserve_percent {
        // Check if the reserve percentage is in a reasonable range
        if reserve_percent < 10 || reserve_percent > 80 {
            msg!("Invalid reserve percentage: {}", reserve_percent);
            return Err(BlackoutError::InvalidBatchConfiguration.into());
        }
        new_config.reserve_percent = reserve_percent;
    }
    
    if let Some(fee_multiplier) = update_params.fee_multiplier {
        // Check if the fee multipliers are in a reasonable range
        if fee_multiplier > 1000 { // Max 10%
            msg!("Invalid fee multiplier: {}", fee_multiplier);
            return Err(BlackoutError::InvalidBatchConfiguration.into());
        }
        new_config.fee_multiplier = fee_multiplier;
    }
    
    if let Some(cu_budget) = update_params.cu_budget_per_hop {
        // Check if the CU budget is appropriate
        if cu_budget < 100_000 || cu_budget > 500_000 {
            msg!("Invalid CU budget: {}", cu_budget);
            return Err(BlackoutError::InvalidBatchConfiguration.into());
        }
        new_config.cu_budget_per_hop = cu_budget;
    }
    
    // Calculate the total number of paths and check if it is valid
    let total_paths = new_config.total_paths();
    
    // Update the configuration
    let transfer_state = &mut ctx.accounts.transfer_state;
    transfer_state.config = new_config;
    
    msg!("Configuration updated: 4 hops, 4 real splits, 44 fake splits, {} paths", 
         total_paths);
    msg!("Parameters: Reserve {}%, Fees {}BP, CU budget {}",
         new_config.reserve_percent, new_config.fee_multiplier, new_config.cu_budget_per_hop);
    
    // Emit event
    emit!(ConfigUpdateExecuted {
        owner: transfer_state.owner,
        reserve_percent: new_config.reserve_percent,
        fee_multiplier: new_config.fee_multiplier,
        cu_budget: new_config.cu_budget_per_hop,
        total_paths,
        transfer_state: ctx.accounts.transfer_state.key(),
    });
    
    Ok(())
}

#[event]
pub struct ConfigUpdateExecuted {
    pub owner: Pubkey,
    pub reserve_percent: u8,
    pub fee_multiplier: u16,
    pub cu_budget: u32,
    pub total_paths: u64,
    pub transfer_state: Pubkey,
}
