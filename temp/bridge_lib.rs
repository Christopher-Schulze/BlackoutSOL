//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Kernmodule des Programms
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Die ursprüngliche Modulstruktur bleibt erhalten
pub mod instructions;

// Brückenmodul zur Vermeidung von Namenskonflikten
pub mod anchor_bridge;

// Wir verwenden Importe aus dem Brückenmodul, nicht direkt aus instructions
use anchor_bridge::Initialize;
use anchor_bridge::ExecuteHop;
use anchor_bridge::BatchHop;
use anchor_bridge::Finalize;
use anchor_bridge::ConfigUpdate;
use anchor_bridge::ConfigUpdateParams;
use anchor_bridge::Refund;
use anchor_bridge::RevealFake;

// Hauptprogrammdefinition für das Anchor-Framework
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
        // Verwendung des Brückenmoduls
        anchor_bridge::initialize::initialize(
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
        anchor_bridge::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        anchor_bridge::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        anchor_bridge::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        anchor_bridge::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        anchor_bridge::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        anchor_bridge::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
