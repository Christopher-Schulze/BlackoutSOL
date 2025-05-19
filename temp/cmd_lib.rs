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

// Beide Module behalten, aber das cmd-Modul für die Programmlogik verwenden
pub mod instructions;
pub mod cmd;

// Importe der Kontextstrukturen für die Programmfunktionen
// Verwende das cmd-Modul, um Namenskonflikte zu vermeiden
use cmd::initialize::Initialize;
use cmd::execute_hop::ExecuteHop;
use cmd::batch_hop::BatchHop;
use cmd::finalize::Finalize;
use cmd::config_update::ConfigUpdate;
use cmd::config_update::ConfigUpdateParams;
use cmd::refund::Refund;
use cmd::reveal_fake::RevealFake;

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
        // Verwende cmd statt instructions
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
        // Verwende cmd statt instructions
        cmd::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        // Verwende cmd statt instructions
        cmd::batch_hop::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        // Verwende cmd statt instructions
        cmd::finalize::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        // Verwende cmd statt instructions
        cmd::config_update::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        // Verwende cmd statt instructions
        cmd::refund::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        // Verwende cmd statt instructions
        cmd::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
