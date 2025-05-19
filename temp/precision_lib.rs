//! BlackoutSOL - Privacy-Payment System for Solana

// Grundlegende Importe 
use anchor_lang::prelude::*;

// Programm-ID für die Identifikation im Solana-Netzwerk
declare_id!("B1ack111111111111111111111111111111111111111");

// Basismodule des Programms
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Um den Namenskonflikt zu lösen, behalten wir das ursprüngliche Modul bei,
// aber importieren instructions::Instructions direkt in die Module, wo es benötigt wird,
// mit einem expliziten Alias
pub mod instructions;

// Wir direkt nur genau die Typen, die wir brauchen, ohne das gesamte Solana-Modul zu importieren
use instructions::initialize::Initialize;
use instructions::execute_hop::ExecuteHop;
use instructions::batch_hop::BatchHop;
use instructions::finalize::Finalize;
use instructions::config_update::ConfigUpdate;
use instructions::config_update::ConfigUpdateParams;
use instructions::refund::Refund;
use instructions::reveal_fake::RevealFake;

// Anchor-Programmdefinition
#[program]
pub mod blackout {
    use super::*;
    // Kritisch: Wir nennen solana_program::sysvar::instructions::Instructions explizit,
    // wenn wir es brauchen, aber mit einem klaren Alias
    use solana_program::sysvar::instructions::Instructions as SolanaInstructions;

    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    ) -> Result<()> {
        // Explizite Modul-Qualifizierung zur Vermeidung von Konflikten
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
