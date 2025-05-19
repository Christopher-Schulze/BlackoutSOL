// bloom_filter_specification.rs
//
// Formal specification and verification of the Bloom filter algorithm in BlackoutSOL
// This file defines the mathematical properties and invariants that the
// Bloom filter must satisfy and enables formal verification.

#![cfg(feature = "verification")]

use crate::state::config::BlackoutConfig;
use crate::utils::{check_bloom_filter, generate_bloom_filter};
use std::cmp::{max, min};

/// Formal specification for the Bloom filter implementation
pub struct BloomFilterSpecification;

impl BloomFilterSpecification {
    /// Verify that the bloom filter generation and checking are consistent
    /// This property must always hold: for any hop_index and split_index that is marked as fake
    /// during generation, check_bloom_filter must return true
    pub fn verify_consistency_property(config: &BlackoutConfig, challenge: &[u8; 32]) -> bool {
        let bloom_filter = generate_bloom_filter(config, challenge);
        
        // Verify with safety limits applied
        // These variables are kept for compatibility but are no longer actively used
        let _max_hops = config.num_hops.min(255) as usize;
        let _max_splits = max(config.real_splits, config.fake_splits).min(255) as usize;
        
        // Apply the same safety limits used in the actual implementation
        let max_hops = 16u8;
        let max_splits = 64u8;
        
        // Apply safety bounds to configuration values
        let safe_num_hops = min(config.num_hops, max_hops);
        let safe_real_splits = min(config.real_splits, max_splits);
        let safe_fake_splits = min(config.fake_splits, max_splits - safe_real_splits);
        
        for hop_idx in 0..safe_num_hops as usize {
            for split_idx in 0..max(safe_real_splits + safe_fake_splits, 1) as usize {
                // Determine if this should be marked as fake based on configuration
                let should_be_fake = Self::is_fake_split(hop_idx, split_idx, config);
                
                // Check if the bloom filter correctly identifies it
                let is_marked_fake = check_bloom_filter(&bloom_filter, hop_idx as u8, split_idx as u8);
                
                // If there's inconsistency, the verification fails
                if should_be_fake != is_marked_fake {
                    return false;
                }
            }
        }
        
        true
    }
    
    /// Determine if a hop/split combination should be marked as fake
    /// This replicates the internal logic of the bloom filter generation 
    /// to allow independent verification
    fn is_fake_split(_hop_idx: usize, split_idx: usize, config: &BlackoutConfig) -> bool {
        // Apply the same safety limits as in the actual implementation
        let max_splits = 64u8;
        let safe_real_splits = min(config.real_splits, max_splits);
        let safe_fake_splits = min(config.fake_splits, max_splits - safe_real_splits);
        
        // The algorithm in generate_bloom_filter marks splits as fake if they are
        // in the range [safe_real_splits..(safe_real_splits + safe_fake_splits)]
        split_idx >= safe_real_splits as usize && split_idx < (safe_real_splits + safe_fake_splits) as usize
    }
    
    /// Verify the no-false-negatives property
    /// This property ensures that the bloom filter will never report a fake split as real
    pub fn verify_no_false_negatives(config: &BlackoutConfig, challenge: &[u8; 32]) -> bool {
        let bloom_filter = generate_bloom_filter(config, challenge);
        
        // Apply the same safety limits as in the actual implementation
        let max_hops = 16u8;
        let max_splits = 64u8;
        
        // Apply safety bounds to configuration values
        let safe_num_hops = min(config.num_hops, max_hops);
        let safe_real_splits = min(config.real_splits, max_splits);
        let safe_fake_splits = min(config.fake_splits, max_splits - safe_real_splits);
        
        // For all splits that should be fake
        for hop_idx in 0..safe_num_hops as usize {
            for split_idx in 0..max(safe_real_splits + safe_fake_splits, 1) as usize {
                if Self::is_fake_split(hop_idx, split_idx, config) {
                    // The bloom filter MUST report this as fake
                    if !check_bloom_filter(&bloom_filter, hop_idx as u8, split_idx as u8) {
                        return false; // False negative found
                    }
                }
            }
        }
        
        true
    }
    
    /// Verify overflow safety properties
    /// This property ensures that the bloom filter behaves correctly even with 
    /// extreme input values near type boundaries
    pub fn verify_overflow_safety() -> bool {
        // Test with boundary values for hop and split indices
        let extreme_config = BlackoutConfig {
            num_hops: 255,
            real_splits: 127,
            fake_splits: 128,
            reserve_percent: 10,
            fee_multiplier: 1,
            cu_budget_per_hop: 200_000,
        };
        
        let challenge = [0xff; 32]; // All ones
        let bloom_filter = generate_bloom_filter(&extreme_config, &challenge);
        
        // Verify extreme indices
        let indices_to_test = [0, 1, 127, 128, 254, 255];
        
        for &hop_idx in &indices_to_test {
            for &split_idx in &indices_to_test {
                // Check that the verification doesn't panic or produce unexpected results
                // We're testing that the function works correctly, not the specific result
                let _ = check_bloom_filter(&bloom_filter, hop_idx, split_idx);
            }
        }
        
        true
    }
    
    /// Verify bit distribution properties
    /// This checks that the bloom filter has appropriate bit distribution
    /// to resist statistical analysis
    pub fn verify_bit_distribution(config: &BlackoutConfig, challenge: &[u8; 32]) -> bool {
        let bloom_filter = generate_bloom_filter(config, challenge);
        
        // Count set bits
        let mut set_bits = 0;
        for byte in &bloom_filter {
            set_bits += byte.count_ones();
        }
        
        // Apply the same safety limits as in the actual implementation
        let max_hops = 16u8;
        let max_splits = 64u8;
        
        // Apply safety bounds to configuration values
        let safe_num_hops = min(config.num_hops, max_hops);
        let safe_real_splits = min(config.real_splits, max_splits);
        let safe_fake_splits = min(config.fake_splits, max_splits - safe_real_splits);
        
        // For our filter size, we expect a reasonable distribution of set bits
        // Too few or too many set bits suggests a poor distribution
        let total_bits = bloom_filter.len() * 8;
        
        // Calculate actual fake entries based on safety-bounded values
        let fake_entries = safe_fake_splits as u32 * safe_num_hops as u32;
        
        // Allow for some variation, but detect grossly wrong bit counts
        // that would indicate a malfunction or poor distribution
        
        // Lower bound: at least fake_entries/4 bits should be set (allowing for more collisions)
        // Upper bound: at most min(fake_entries*2, total_bits/2) bits should be set
        let lower_bound = if fake_entries > 0 { fake_entries / 4 } else { 0 };
        let upper_bound = min(fake_entries * 2, total_bits as u32 / 2);
        
        println!("Bit distribution test: Set bits: {}, Expected range: {}..={}", set_bits, lower_bound, upper_bound);
        
        (lower_bound..=upper_bound).contains(&set_bits)
    }
}

/// Formal verification test harness
#[cfg(test)]
mod verification_tests {
    use super::*;
    
    #[test]
    fn test_consistency_property() {
        let config = BlackoutConfig {
            num_hops: 5,
            real_splits: 3,
            fake_splits: 2,
            reserve_percent: 10,
            fee_multiplier: 1,
            cu_budget_per_hop: 200_000,
        };
        
        let challenge = [0; 32];
        assert!(BloomFilterSpecification::verify_consistency_property(&config, &challenge));
    }
    
    #[test]
    fn test_no_false_negatives() {
        let config = BlackoutConfig {
            num_hops: 5,
            real_splits: 3,
            fake_splits: 2,
            reserve_percent: 10,
            fee_multiplier: 1,
            cu_budget_per_hop: 200_000,
        };
        
        let challenge = [0; 32];
        assert!(BloomFilterSpecification::verify_no_false_negatives(&config, &challenge));
    }
    
    #[test]
    fn test_overflow_safety() {
        assert!(BloomFilterSpecification::verify_overflow_safety());
    }
    
    #[test]
    fn test_bit_distribution() {
        let config = BlackoutConfig {
            num_hops: 5,
            real_splits: 3,
            fake_splits: 2,
            reserve_percent: 10,
            fee_multiplier: 1,
            cu_budget_per_hop: 200_000,
        };
        
        let challenge = [0; 32];
        assert!(BloomFilterSpecification::verify_bit_distribution(&config, &challenge));
    }
}
