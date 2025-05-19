//! BlackoutSOL - Entrypoint for Anchor integration

use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Import of Anchor structures for program functions
use crate::instructions::initialize::Initialize;
use crate::instructions::execute_hop::ExecuteHop;
use crate::instructions::batch_hop::BatchHop;
use crate::instructions::finalize::Finalize;
use crate::instructions::config_update::ConfigUpdate;
use crate::instructions::config_update::ConfigUpdateParams;
use crate::instructions::refund::Refund;
use crate::instructions::reveal_fake::RevealFake;

// Import of the actual module for implementation
use crate::instructions;

/// Main program module processed by the Anchor framework
#[program]
pub mod blackout {
    use super::*;

    /// Initializes a new anonymous transfer
    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    ) -> Result<()> {
        instructions::initialize::initialize(
            ctx,
            amount,
            hyperplonk_proof,
            range_proof,
            challenge,
            merkle_proof,
        )
    }

    /// Executes a single hop in the anonymous transfer
    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    /// Executes multiple hops in a single transaction
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        instructions::batch_hop::process_batch_hop(ctx, batch_index)
    }

    /// Finalizes the anonymous transfer
    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::finalize::finalize(ctx, proof_data)
    }
    
    /// Updates specific configuration parameters
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        instructions::config_update::update_config(ctx, update_params)
    }
    
    /// Triggers a refund if a transfer has failed
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        instructions::refund::refund(ctx)
    }
    
    /// Proves that a split address is a fake split
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        instructions::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
