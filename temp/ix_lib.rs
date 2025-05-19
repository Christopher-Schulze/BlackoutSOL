//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

// Programm-ID für die Identifikation im Solana-Netzwerk
declare_id!("B1ack111111111111111111111111111111111111111");

// Standardmodule
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Originales instructions-Modul, behalten wir für Anchor-Kompatibilität bei
pub mod instructions;

// Neues Modul mit anderem Namen zur Vermeidung von Konflikten
pub mod ix;

// Importe der Kontextstrukturen für die Programmfunktionen
use ix::Initialize;
use ix::ExecuteHop;
use ix::BatchHop;
use ix::Finalize;
use ix::ConfigUpdate;
use ix::ConfigUpdateParams;
use ix::Refund;
use ix::RevealFake;

// Anchor-Programmdefinition
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
        ix::initialize::initialize(
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
        ix::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        ix::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        ix::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        ix::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        ix::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        ix::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
