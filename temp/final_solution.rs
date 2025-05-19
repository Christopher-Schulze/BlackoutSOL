//! BlackoutSOL - Privacy-Payment System for Solana

use anchor_lang::prelude::*;

// Programm-ID für die Identifikation im Solana-Netzwerk
declare_id!("B1ack111111111111111111111111111111111111111");

// Basismodule des Programms
pub mod errors;
pub mod state;
pub mod utils;
pub mod poseidon_validator;
pub mod poseidon_constants;

// Wir behalten das ursprüngliche Modul bei, da es vom Anchor-Framework erwartet wird
pub mod instructions;

// Importe der Kontextstrukturen für die Programmfunktionen
// Wir vermeiden direkte Importe von Instructions-Typen
use instructions::initialize::Initialize;
use instructions::execute_hop::ExecuteHop;
use instructions::batch_hop::BatchHop;
use instructions::finalize::Finalize;
use instructions::config_update::ConfigUpdate;
use instructions::config_update::ConfigUpdateParams;
use instructions::refund::Refund;
use instructions::reveal_fake::RevealFake;

// Import des Solana instructions-Typs mit einem expliziten Alias
// um Namenskonflikte zu vermeiden
use solana_program::sysvar::instructions::Instructions as SolanaInstructions;

/// Zentrale Programmdefinition für BlackoutSOL
#[program]
pub mod blackout {
    use super::*;
    // Explizite Verwendung des Alias, um Namenskonflikte zu vermeiden
    use solana_program::sysvar::instructions::Instructions as SolanaInstructions;

    /// Initialisiert einen neuen anonymen Transfer mit 4 Hops, 4 echten Splits und 44 Fake-Splits
    pub fn initialize(
        ctx: Context<Initialize>,
        amount: u64,
        hyperplonk_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_proof: Vec<u8>,
    ) -> Result<()> {
        // Verwendung des vollqualifizierten Pfads zum Modul
        crate::instructions::initialize::initialize(
            ctx,
            amount,
            hyperplonk_proof,
            range_proof,
            challenge,
            merkle_proof,
        )
    }

    /// Führt einen einzelnen Hop im anonymen Transfer aus
    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        crate::instructions::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    /// Führt mehrere Hops in einer einzigen Transaktion aus
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        crate::instructions::batch_hop::process_batch_hop(ctx, batch_index)
    }

    /// Finalisiert den anonymen Transfer
    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        crate::instructions::finalize::finalize(ctx, proof_data)
    }
    
    /// Aktualisiert spezifische Konfigurationsparameter
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        crate::instructions::config_update::update_config(ctx, update_params)
    }
    
    /// Löst eine Rückerstattung aus, wenn ein Transfer fehlgeschlagen ist
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        crate::instructions::refund::refund(ctx)
    }
    
    /// Beweist, dass eine Split-Adresse ein Fake-Split ist
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        crate::instructions::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
