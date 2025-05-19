// BlackoutSOL - Privacy-Payment System for Solana
use anchor_lang::prelude::*;

declare_id!("B1ack111111111111111111111111111111111111111");

// Standardmodule - behalten wir bei
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Da der Fehler mit #[program] zusammenhängt, muss die Module-Struktur
// exakt den Erwartungen von Anchor entsprechen
#[program]
pub mod blackout {
    use super::*;
    
    // Importieren wir die Instructions-Module innerhalb des blackout-Moduls
    // um Namenskonflikte mit solana_program::sysvar::instructions::Instructions zu vermeiden
    use crate::instructions;

    pub fn initialize(
        ctx: Context<instructions::initialize::Initialize>,
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

    pub fn execute_hop(
        ctx: Context<instructions::execute_hop::ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<instructions::batch_hop::BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        instructions::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<instructions::finalize::Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<instructions::config_update::ConfigUpdate>,
        update_params: instructions::config_update::ConfigUpdateParams,
    ) -> Result<()> {
        instructions::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<instructions::refund::Refund>,
    ) -> Result<()> {
        instructions::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<instructions::reveal_fake::RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        instructions::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}

// Instructions-Modul erst nach dem #[program]-Makro definieren
// um potenzielle Konflikte mit solana_program::sysvar::instructions::Instructions zu vermeiden
pub mod instructions;
