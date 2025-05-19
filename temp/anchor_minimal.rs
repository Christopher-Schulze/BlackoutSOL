//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Modul mit anderem Namen als 'instructions', um Konflikte zu vermeiden
pub mod cmd {
    // Re-Export der ursprünglichen Module
    pub use crate::instructions::initialize;
    pub use crate::instructions::execute_hop;
    pub use crate::instructions::batch_hop;
    pub use crate::instructions::finalize;
    pub use crate::instructions::config_update;
    pub use crate::instructions::refund;
    pub use crate::instructions::reveal_fake;
}

// Wir behalten das ursprüngliche Modul für die Anchor-Kompatibilität
pub mod instructions;

// Weitere Module des Programms
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Importe für Kontextstrukturen
use instructions::initialize::Initialize;
use instructions::execute_hop::ExecuteHop;
use instructions::batch_hop::BatchHop;
use instructions::finalize::Finalize;
use instructions::config_update::ConfigUpdate;
use instructions::config_update::ConfigUpdateParams;
use instructions::refund::Refund;
use instructions::reveal_fake::RevealFake;

#[program]
pub mod blackout {
    use super::*;
    
    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    ) -> Result<()> {
        // Verwendung des alternativen Moduls
        cmd::initialize::initialize(
            ctx,
            amount,
            hyperplonk_proof,
            range_proof,
            challenge,
            merkle_proof,
        )
    }

    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        cmd::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        cmd::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        cmd::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        cmd::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        cmd::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        cmd::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
