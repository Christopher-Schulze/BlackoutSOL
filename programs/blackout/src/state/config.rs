use anchor_lang::prelude::*;

/// Configuration account for global parameters of the Blackout system
#[account]
pub struct ConfigAccount {
    /// The current configuration
    pub config: BlackoutConfig,
    
    /// Administrator key with permission to configure
    pub authority: Pubkey,
    
    /// Bump seed for PDA validation
    pub bump: u8,
}

/// Configuration parameters for the Blackout system
/// Fixed configuration with 4 hops, 4 splits, and 44 fake splits
#[derive(AnchorSerialize, AnchorDeserialize, Clone, Copy, Debug, PartialEq)]
pub struct BlackoutConfig {
    /// Number of hops in the anonymity path (fixed at 4)
    pub num_hops: u8,
    
    /// Number of real splits per hop (fixed at 4)
    pub real_splits: u8,
    
    /// Number of fake splits per hop (fixed at 44)
    pub fake_splits: u8,
    
    /// Reserve percentage (40%)
    pub reserve_percent: u8,

    /// Fee multiplier in basis points (1 BP = 0.01%)
    /// Set to 200 BP = 2%
    pub fee_multiplier: u16,
    
    /// Compute unit budget per hop transaction
    pub cu_budget_per_hop: u32,
}

impl BlackoutConfig {
    /// Creates the default Blackout configuration
    /// 4 hops x 4 real splits x 44 fake splits
    pub fn new() -> Self {
        Self {
            num_hops: 4,
            real_splits: 4,
            fake_splits: 44,
            reserve_percent: 40,
            fee_multiplier: 200, // 2%
            cu_budget_per_hop: 220_000,
        }
    }
    
    /// Calculates the total number of anonymity paths
    pub fn total_paths(&self) -> u64 {
        let total_outputs = self.real_splits as u64 + self.fake_splits as u64;
        total_outputs.pow(self.num_hops as u32)
    }
    
    /// Calculates the maximum number of batch hops
    pub fn max_batch_size(&self) -> u8 {
        // Ensure that the batch size stays within the CU limit.
        // Use a conservative estimate for CU per hop within a batch transaction,
        // which includes PDA initialization and ZK verification costs for each batched hop.
        // This internal estimate might differ from `self.cu_budget_per_hop` which is for single hop execution.
        let cu_per_hop_in_batch = 108_000; // ~18k for PDA + ~90k for ZK verification
        let max_cu_for_batch = 1_300_000;    // Slightly below the Solana transaction limit of 1.4M
        let batch_size = max_cu_for_batch / cu_per_hop_in_batch;
        std::cmp::min(self.num_hops, batch_size as u8)
    }
}
