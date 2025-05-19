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

// WICHTIG: Wir verwenden aus Kompatibilitätsgründen mit dem Anchor-Framework
// den Modulnamen 'instructions', müssen aber Namenskonflikte mit
// solana_program::sysvar::instructions::Instructions vermeiden
pub mod instructions;

// Um Namenskonflikte zu vermeiden, geben wir der Solana-Struktur einen Alias
use solana_program::sysvar::instructions::Instructions as SolanaInstructions;

// Importe der Kontextstrukturen für die Programmfunktionen
use instructions::initialize::Initialize;
use instructions::execute_hop::ExecuteHop;
use instructions::batch_hop::BatchHop;
use instructions::finalize::Finalize;
use instructions::config_update::ConfigUpdate;
use instructions::config_update::ConfigUpdateParams;
use instructions::refund::Refund;
use instructions::reveal_fake::RevealFake;

/// Die zentrale Programmdefinition für BlackoutSOL
#[program]
pub mod blackout {
    use super::*;

    /// Initialisiert einen neuen anonymen Transfer mit 4 Hops, 4 echten Splits und 44 Fake-Splits
    /// 
    /// Dies ist der Einstiegspunkt für neue Transfers im Blackout-System.
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

    /// Führt einen einzelnen Hop im anonymen Transfer aus
    /// 
    /// Diese Funktion wird verwendet, wenn Hops einzeln ausgeführt werden müssen.
    /// Für effizientere Ausführung sollte batch_hop bevorzugt werden.
    pub fn execute_hop(
        ctx: Context<ExecuteHop>,
        hop_index: u8,
        proof_data: [u8; 128],
        range_proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::execute_hop::execute_hop(ctx, hop_index, proof_data, range_proof_data)
    }
    
    /// Führt mehrere Hops in einer einzigen Transaktion aus
    /// 
    /// Optimiert Compute Units und Netzwerkgebühren für den Transfer.
    /// batch_index gibt an, welcher Batch von Hops ausgeführt werden soll.
    pub fn execute_batch_hop(
        ctx: Context<BatchHop>,
        batch_index: u8,
    ) -> Result<()> {
        instructions::batch_hop::process_batch_hop(ctx, batch_index)
    }

    /// Finalisiert den anonymen Transfer
    /// 
    /// Nachdem alle 4 Hops ausgeführt wurden, kann der Transfer finalisiert werden.
    /// Der Empfänger erhält dann den endgültigen Betrag.
    pub fn finalize_transfer(
        ctx: Context<Finalize>,
        proof_data: [u8; 128],
    ) -> Result<()> {
        instructions::finalize::finalize(ctx, proof_data)
    }
    
    /// Aktualisiert spezifische Konfigurationsparameter
    /// 
    /// Die Grundstruktur (4 Hops, 4 echte Splits, 44 Fake-Splits) bleibt unverändert,
    /// aber Parameter wie Reserve-Prozentsatz, Gebühren und CU-Budget können angepasst werden.
    pub fn update_config(
        ctx: Context<ConfigUpdate>,
        update_params: ConfigUpdateParams,
    ) -> Result<()> {
        instructions::config_update::update_config(ctx, update_params)
    }
    
    /// Löst eine Rückerstattung aus, wenn ein Transfer fehlgeschlagen ist
    /// 
    /// Der Eigentümer kann dies verwenden, um seine Gelder zurückzufordern.
    pub fn trigger_refund(
        ctx: Context<Refund>,
    ) -> Result<()> {
        instructions::refund::refund(ctx)
    }
    
    /// Beweist, dass eine Split-Adresse ein Fake-Split ist
    /// 
    /// Dies kann für Audit-Zwecke und zur Validierung des Systems verwendet werden.
    pub fn reveal_fake_split(
        ctx: Context<RevealFake>,
        hop_index: u8,
        split_index: u8,
    ) -> Result<()> {
        instructions::reveal_fake::reveal_fake(ctx, hop_index, split_index)
    }
}
