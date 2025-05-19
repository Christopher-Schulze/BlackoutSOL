//! BlackoutSOL - Anchor Framework Integration with Clean Isolation
//! 
//! This crate serves as a dedicated bridge between the Anchor Framework and the
//! main BlackoutSOL program. It resolves naming conflicts between the `instructions` module
//! and `solana_program::sysvar::instructions::Instructions` by cleanly isolating
//! the Anchor integration in a separate crate.

// Basic Anchor integrations
use anchor_lang::prelude::*;

// Program ID (identical to the original BlackoutSOL program ID)
declare_id!("B1ack111111111111111111111111111111111111111");

// Account structures for Anchor
pub mod anchor_accounts;
use anchor_accounts::*;

// Import parameters and types from blackout
use blackout::instructions::config_update::ConfigUpdateParams;

/// The Anchor program module for BlackoutSOL
#[program]
pub mod blackout_anchor {
    use super::*;

    /// Initializes a new anonymous transfer
    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        _hyperplonk_proof: [u8; 128],
        _range_proof: [u8; 128],
        _challenge: [u8; 32],
        _merkle_proof: Vec<u8>,
    ) -> Result<()> {
        msg!("Initializing Blackout Transfer with amount: {}", amount);
        Ok(())
    }

    /// Executes a single hop in the anonymous transfer
    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        _proof_data: [u8; 128],
        _range_proof_data: [u8; 128],
    ) -> Result<()> {
        msg!("Executing hop {}", hop_index);
        Ok(())
    }

    /// Executes multiple hops in a batch operation
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        msg!("Batch execution of hop {}", batch_index);
        Ok(())
    }

    /// Finalizes the anonymous transfer
    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        _proof_data: [u8; 128],
    ) -> Result<()> {
        msg!("Finalizing the transfer");
        Ok(())
    }

    /// Updates configuration parameters
    pub fn config_update(
        ctx: Context<ConfigUpdate>,
        _update_params: ConfigUpdateParams,
    ) -> Result<()> {
        msg!("Updating configuration");
        Ok(())
    }

    /// Triggers a refund if a transfer has failed
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        msg!("Triggering refund");
        Ok(())
    }

    /// Proves that a split address is a fake split
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        msg!("Revealing fake split at hop {} split {}", hop_index, split_index);
        Ok(())
    }
}
