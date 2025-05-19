// BlackoutSOL - Privacy-Payment System for Solana

// Verwenden der notwendigen Anchor-Importe
use anchor_lang::prelude::*;

// Programm-ID definieren
declare_id!("B1ack111111111111111111111111111111111111111");

// Modulstruktur
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Wichtig: Wir importieren zuerst alle anderen Module,
// definieren das blackout-Programm OHNE andere Importe zu verwenden,
// und importieren erst DANACH das instructions-Modul
// Dies umgeht den Namenskonflikt mit solana_program::sysvar::instructions::Instructions

// ANCHOR-PROGRAMM-DEFINITION
// Das Programm definiert die RPC-Endpunkte
#[program]
pub mod blackout {
    use anchor_lang::{prelude::*, solana_program};
    
    // Kontext-Typen direkt aus instructions-Modul importieren
    use crate::instructions::initialize::Initialize;
    use crate::instructions::execute_hop::ExecuteHop;
    use crate::instructions::batch_hop::BatchHop;
    use crate::instructions::finalize::Finalize;
    use crate::instructions::config_update::{ConfigUpdate, ConfigUpdateParams};
    use crate::instructions::refund::Refund;
    use crate::instructions::reveal_fake::RevealFake;

    // Funktionalität aus instructions-Modul aufrufen
    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    ) -> Result<()> {
        let instructions_mod = crate::instructions::initialize;
        instructions_mod::initialize(
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
        let instructions_mod = crate::instructions::execute_hop;
        instructions_mod::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        let instructions_mod = crate::instructions::batch_hop;
        instructions_mod::process_batch_hop(ctx, batch_index)
    }

    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        let instructions_mod = crate::instructions::finalize;
        instructions_mod::finalize(ctx, proof_data)
    }
    
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        let instructions_mod = crate::instructions::config_update;
        instructions_mod::update_config(ctx, update_params)
    }
    
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        let instructions_mod = crate::instructions::refund;
        instructions_mod::refund(ctx)
    }
    
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        let instructions_mod = crate::instructions::reveal_fake;
        instructions_mod::reveal_fake(ctx, hop_index, split_index)
    }
}

// Instruktionsmodul NACH dem Anchor-Programm definieren,
// um Namenskonflikte mit solana_program::sysvar::instructions zu vermeiden
pub mod instructions;
