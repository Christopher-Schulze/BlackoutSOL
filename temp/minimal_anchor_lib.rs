// BlackoutSOL - Privacy-Payment System for Solana
// Minimales Beispiel für Anchor-Integration
use anchor_lang::prelude::*;

// Programm-ID für die Identifikation im Solana-Netzwerk
declare_id!("B1ack111111111111111111111111111111111111111");

// Hier definieren wir direkte Importe, ohne Module zu verwenden
// Dies vermeidet potenzielle Konflikte mit solana_program::sysvar::instructions::Instructions
use crate::instructions::initialize::Initialize;
use crate::instructions::execute_hop::ExecuteHop;
use crate::instructions::batch_hop::BatchHop;
use crate::instructions::finalize::Finalize;
use crate::instructions::config_update::ConfigUpdate;
use crate::instructions::config_update::ConfigUpdateParams;
use crate::instructions::refund::Refund;
use crate::instructions::reveal_fake::RevealFake;

// Minimale Programmdefinition für Anchor-Framework
#[program]
pub mod blackout {
    use super::*;
    use crate::instructions;
    
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

    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        instructions::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        instructions::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        instructions::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        instructions::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}

// Module nach #[program] deklarieren, um potenzielle Konflikte zu vermeiden
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;
pub mod instructions;
