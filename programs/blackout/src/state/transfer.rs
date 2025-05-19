use anchor_lang::prelude::*;
use super::config::BlackoutConfig;

// Import of the Solana compatibility layer for compute units
use crate::solana_imports::sol_remaining_compute_units;

/// Stores the state of an anonymous transfer with extended functionality
#[account]
pub struct TransferState {
    /// Owner of the transfer
    pub owner: Pubkey,
    
    /// Total amount of the transfer
    pub amount: u64,
    
    /// Current hop index
    pub current_hop: u8,
    
    /// Seed for generating stealth PDAs
    pub seed: [u8; 32],
    
    /// Flag indicating if the transfer is completed
    pub completed: bool,
    
    /// Bump for the PDA address
    pub bump: u8,
    
    /// Blackout configuration (fixed: 4 hops, 4 real splits, 44 fake splits)
    pub config: BlackoutConfig,
    
    /// Aggregated ZK proof for batch verification
    pub batch_proof: [u8; 128],
    
    /// Range proof for the split amounts
    pub range_proof: [u8; 128],
    
    /// Commitments for the split amounts (max 8)
    pub commitments: [[u8; 32]; 8],
    
    /// Number of batch transactions already processed
    pub batch_count: u8,
    
    /// Total fees for the transfer
    pub total_fees: u64,
    
    /// Reserve for the transfer
    pub reserve: u64,
    
    /// Recipients of the final payment (up to 6 wallets)
    pub recipients: [Pubkey; 6],
    
    /// Number of recipient wallets to use (3-6)
    pub recipient_count: u8,
    
    /// Bloom filter for fake splits (16 bytes for 128 bits)
    pub fake_bloom: [u8; 16],
    
    /// Challenge for ZK proofs
    pub challenge: [u8; 32],
    
    /// Timestamp for timing protection
    pub timestamp: i64,
    
    /// Merkle root for wallet set
    pub merkle_root: [u8; 32],
    
    /// Refund flag in case the transfer fails
    pub refund_triggered: bool,
}

impl TransferState {
    /// Calculates the memory requirement for the account
    pub const SIZE: usize = 8 +   // Discriminator
                            32 +  // owner
                            8 +   // amount
                            1 +   // current_hop
                            32 +  // seed
                            1 +   // completed
                            1 +   // bump
                            9 +   // config (BlackoutConfig size: 1+1+1+1+2+4 = 9 bytes)
                            128 + // batch_proof
                            128 + // range_proof
                            256 + // commitments (8 x 32)
                            1 +   // batch_count
                            8 +   // total_fees
                            8 +   // reserve
                            192 + // recipients (6 * 32)
                            1 +   // recipient_count
                            16 +  // fake_bloom
                            32 +  // challenge
                            8 +   // timestamp
                            32 +  // merkle_root
                            1;    // refund_triggered
                            // Total = 702 bytes
                            
    /// Initializes a new TransferState
    pub fn new(
        owner: Pubkey,
        amount: u64,
        seed: [u8; 32],
        bump: u8,
        recipients: [Pubkey; 6],
        recipient_count: u8,
        config: BlackoutConfig,
        batch_proof: [u8; 128],
        range_proof: [u8; 128],
        challenge: [u8; 32],
        merkle_root: [u8; 32],
        fake_bloom: [u8; 16],
        timestamp: i64,
    ) -> Self {
        Self {
            owner,
            amount,
            current_hop: 0,
            seed,
            completed: false,
            bump,
            config,
            batch_proof,
            range_proof,
            commitments: [[0; 32]; 8], // Will be filled later
            batch_count: 0,
            total_fees: 0,
            reserve: 0,
            recipients,
            recipient_count,
            fake_bloom,
            challenge,
            timestamp,
            merkle_root,
            refund_triggered: false,
        }
    }
    
    /// Checks if the transfer is in batch hop mode
    pub fn is_batch_mode(&self) -> bool {
        self.batch_count > 0
    }
    
    /// Calculates the number of remaining hops
    pub fn remaining_hops(&self) -> u8 {
        if self.current_hop >= self.config.num_hops {
            0
        } else {
            self.config.num_hops - self.current_hop
        }
    }
    
    /// Calculates the current batch progress in percent
    pub fn progress_percent(&self) -> u8 {
        if self.config.num_hops == 0 {
            return 100;
        }
        ((self.current_hop as u16 * 100) / (self.config.num_hops as u16)) as u8
    }
    
    /// Checks if enough compute units are available for the next critical step within the current transaction.
    /// `cu_needed_for_next_step` is an estimate of CUs required for the upcoming operations.
    pub fn has_enough_cu_for_next_hop(&self, cu_needed_for_next_step: u32) -> bool {
        let remaining_units = sol_remaining_compute_units();

        if remaining_units < cu_needed_for_next_step as u64 {
            msg!(
                "Not enough CUs for next step. Remaining: {}, Needed: {}",
                remaining_units,
                cu_needed_for_next_step
            );
            return false;
        }
        true
    }
}