//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Standardmodule des Programms
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Der originale Modulname wird beibehalten, aber wir verwenden ihn mit expliziten Pfaden
pub mod instructions;

// Wir importieren die Kontextstrukturen für die Anchor-Funktionen
use instructions::initialize::Initialize;
use instructions::execute_hop::ExecuteHop;
use instructions::batch_hop::BatchHop;
use instructions::finalize::Finalize;
use instructions::config_update::ConfigUpdate;
use instructions::config_update::ConfigUpdateParams;
use instructions::refund::Refund;
use instructions::reveal_fake::RevealFake;

// Die Hauptprogrammdefinition, folgt genau der Anchor-Dokumentation
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
        let ix_mod = crate::instructions::initialize;
        ix_mod::initialize(
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
        let ix_mod = crate::instructions::execute_hop;
        ix_mod::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        let ix_mod = crate::instructions::batch_hop;
        ix_mod::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        let ix_mod = crate::instructions::finalize;
        ix_mod::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        let ix_mod = crate::instructions::config_update;
        ix_mod::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        let ix_mod = crate::instructions::refund;
        ix_mod::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        let ix_mod = crate::instructions::reveal_fake;
        ix_mod::reveal_fake(ctx, hop_index, split_index)
    }
}
