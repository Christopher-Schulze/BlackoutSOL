//! Optimized implementations for cryptographic validations
//!
//! This module contains highly optimized versions of validation functions
//! specifically developed for minimal compute unit consumption on the Solana
//! blockchain.

use anchor_lang::prelude::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use anchor_lang::solana_program::program::invoke_signed;
use anchor_lang::solana_program::system_instruction;
use solana_poseidon::{Parameters, Endianness, hashv};
use crate::errors::BlackoutError;
use crate::state::config::BlackoutConfig;
use std::convert::TryInto;
use std::cell::RefCell;
use std::rc::Rc;
use arrayref::array_ref;

/// Constant for the maximum number of cache entries
const PDA_CACHE_SIZE: usize = 8;

/// Structure for the PDA cache
/// 
/// This structure stores recently validated PDAs to
/// avoid redundant calculations.
#[derive(Debug)]
pub struct PdaCache {
    /// The stored PDA entries
    pub entries: [(Pubkey, [u8; 32], u8, u8, Pubkey, u8); PDA_CACHE_SIZE],
    /// The current index in the cache
    pub current_index: usize,
}

impl Default for PdaCache {
    fn default() -> Self {
        // Initialize with empty entries
        PdaCache {
            entries: [
                (Pubkey::default(), [0; 32], 0, 0, Pubkey::default(), 0);
                PDA_CACHE_SIZE
            ],
            current_index: 0,
        }
    }
}

impl PdaCache {
    /// Checks if a PDA is in the cache
    pub fn check_cache(
        &self,
        program_id: &Pubkey,
        seed: &[u8; 32],
        hop_index: u8,
        split_index: u8,
        pda: &Pubkey,
    ) -> Option<u8> {
        for entry in &self.entries {
            if entry.0 == *program_id && 
               entry.1 == *seed && 
               entry.2 == hop_index && 
               entry.3 == split_index && 
               entry.4 == *pda {
                // Cache hit: Return bump
                return Some(entry.5);
            }
        }
        None
    }
    
    /// Adds a validated PDA to the cache
    pub fn add_to_cache(
        &mut self,
        program_id: Pubkey,
        seed: [u8; 32],
        hop_index: u8,
        split_index: u8,
        pda: Pubkey,
        bump: u8,
    ) {
        // FIFO strategy: Replace the oldest entry
        self.entries[self.current_index] = (program_id, seed, hop_index, split_index, pda, bump);
        self.current_index = (self.current_index + 1) % PDA_CACHE_SIZE;
    }
}

/// Thread-local instance of the PDA cache
thread_local! {
    static PDA_CACHE: RefCell<PdaCache> = RefCell::new(PdaCache::default());
}

/// Optimized version of PDA validation with caching
///
/// This function validates that a PDA was correctly derived by the program with the
/// expected seeds. It uses caching to avoid redundant calculations.
pub fn optimized_verify_pda_derivation<'a>(
    program_id: &Pubkey,
    seed: &[u8; 32],
    hop_index: u8,
    split_index: u8,
    pda_account: &AccountInfo<'a>
) -> Result<(Pubkey, u8)> {
    // 1. Check the cache for a previous hit
    let cached_bump = PDA_CACHE.with(|cache| {
        cache.borrow().check_cache(program_id, seed, hop_index, split_index, pda_account.key)
    });
    
    if let Some(bump) = cached_bump {
        // Cache hit: We can skip the full calculation
        return Ok((*pda_account.key, bump));
    }
    
    // 2. On cache miss: Perform full validation
    let is_fake = split_index >= 4; // The first 4 splits are real, the rest are fake
    
    // Prepare seeds for PDA derivation
    let fake_marker: &[u8] = if is_fake { b"fake" } else { b"real" };
    let seed_prefix = b"stealth";
    
    // 3. Optimierte PDA-Ableitung
    let mut bump_seed = 255;
    let mut seeds = vec![
        seed_prefix.as_ref(),
        seed.as_ref(),
        &[hop_index],
        &[split_index],
        fake_marker,
        &[bump_seed],
    ];
    
    // Binary search for the bump seed
    // Dies ist schneller als lineare Suche, aber noch schneller als der std::iter::successors Ansatz
    let mut min_bump = 0;
    let mut max_bump = 255;
    
    while min_bump <= max_bump {
        let mid_bump = (min_bump + max_bump) / 2;
        bump_seed = mid_bump;
        seeds[5] = &[bump_seed];
        
        if let Ok(key) = Pubkey::create_program_address(&seeds, program_id) {
            if key == *pda_account.key {
                // PDA gefunden!
                
                // Add to cache
                PDA_CACHE.with(|cache| {
                    cache.borrow_mut().add_to_cache(
                        *program_id,
                        *seed,
                        hop_index,
                        split_index,
                        *pda_account.key,
                        bump_seed
                    )
                });
                
                return Ok((key, bump_seed));
            }
            // If the generated PDA doesn't match the expected one,
            // we need to keep searching. This is not a typical case,
            // but we handle it for completeness.
            max_bump = mid_bump - 1;
        } else {
            // This combination doesn't generate a valid PDA
            min_bump = mid_bump + 1;
        }
    }
    
    // 4. If we reach this point, validation has failed
    Err(BlackoutError::InvalidPda.into())
}

/// Optimized version of the Bloom filter check
///
/// This function checks if a split is marked as a fake split in the Bloom filter.
/// It uses optimized bit manipulation for maximum speed.
pub fn optimized_check_bloom_filter(bloom_filter: &[u8; 16], hop_index: u8, split_index: u8) -> bool {
    // Calculate position in the filter with optimized bit manipulation
    let position = ((hop_index as u32) << 16 | split_index as u32) & 127;
    
    // Check if the corresponding bit is set
    // This uses a direct bit mask instead of modulo operations
    let byte_index = (position >> 3) as usize;
    let bit_mask = 1u8 << (position & 7);
    
    (bloom_filter[byte_index] & bit_mask) != 0
}

/// Optimized dual-path validation for PDAs
///
/// This function combines optimized PDA validation with Bloom filter validation
/// for maximum efficiency in validating PDAs.
pub fn optimized_dual_path_validation<'a>(
    program_id: &Pubkey,
    seed: &[u8; 32],
    hop_index: u8,
    split_index: u8,
    pda_account: &AccountInfo<'a>,
    bloom_filter: &[u8; 16],
) -> Result<bool> {
    // 1. Attempt direct PDA validation (optimized with caching)
    let direct_validation = optimized_verify_pda_derivation(
        program_id,
        seed,
        hop_index,
        split_index,
        pda_account
    );
    
    // 2. If direct validation is successful, the PDA is valid
    if direct_validation.is_ok() {
        return Ok(true);
    }
    
    // 3. If direct validation fails, attempt Bloom filter validation
    // This is especially important for fake splits
    let bloom_validation = optimized_check_bloom_filter(
        bloom_filter,
        hop_index,
        split_index
    );
    
    // 4. Return the result of the Bloom filter validation
    Ok(bloom_validation)
}

/// Optimized batch validation of multiple PDAs
///
/// This function validates multiple PDAs in a batch operation with minimal
/// redundant calculations.
pub fn batch_validate_pdas<'a>(
    program_id: &Pubkey,
    seed: &[u8; 32],
    base_hop_index: u8,
    pdas: &[AccountInfo<'a>],
    bloom_filter: &[u8; 16],
) -> Result<Vec<bool>> {
    let mut results = Vec::with_capacity(pdas.len());
    
    // For each PDA in the batch
    for (i, pda) in pdas.iter().enumerate() {
        let hop_index = base_hop_index + i as u8;
        let split_index = i as u8;
        
        // Perform optimized dual-path validation
        let is_valid = optimized_dual_path_validation(
            program_id,
            seed,
            hop_index,
            split_index,
            pda,
            bloom_filter
        )?;
        
        results.push(is_valid);
    }
    
    Ok(results)
}

/// Optimized parallel batch execution with preflight validation
///
/// This function performs batch execution with a prior validation phase
/// to save compute units if any of the PDAs are invalid.
pub fn optimized_parallel_batch_execution<'a>(
    state: &'a AccountInfo<'a>,
    system_program: &'a AccountInfo<'a>,
    splits: &[u64],
    pdas: &'a [AccountInfo<'a>],
    hop_index: u8,
    seed: &[u8; 32],
    bump: u8,
    program_id: &Pubkey,
    bloom_filter: &[u8; 16],
) -> Result<()> {
    // 1. Preflight-Validierung aller PDAs
    let validation_results = batch_validate_pdas(
        program_id,
        seed,
        hop_index,
        pdas,
        bloom_filter
    )?;
    
    // 2. Check if all PDAs are valid
    if validation_results.iter().any(|&valid| !valid) {
        // If at least one PDA is invalid, abort
        return Err(BlackoutError::InvalidPda.into());
    }
    
    // 3. If all PDAs are valid, execute the transfers
    let seeds = &[
        b"transfer".as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds[..]];
    
    for (i, pda) in pdas.iter().enumerate() {
        let amount = splits[i];
        
        // Execute transfer
        invoke_signed(
            &system_instruction::transfer(
                state.key,
                pda.key,
                amount,
            ),
            &[
                state.clone(),
                pda.clone(),
                system_program.clone(),
            ],
            signer_seeds,
        )?;
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Tests for optimized PDA validation
    #[test]
    fn test_pda_cache() {
        let mut cache = PdaCache::default();
        let program_id = Pubkey::new_unique();
        let seed = [1u8; 32];
        let hop_index = 2;
        let split_index = 3;
        let pda = Pubkey::new_unique();
        let bump = 254;
        
        // Initial check should return None
        assert_eq!(
            cache.check_cache(&program_id, &seed, hop_index, split_index, &pda),
            None
        );
        
        // Add to cache
        cache.add_to_cache(program_id, seed, hop_index, split_index, pda, bump);
        
        // Now the cache should have a hit
        assert_eq!(
            cache.check_cache(&program_id, &seed, hop_index, split_index, &pda),
            Some(bump)
        );
        
        // Different hop index should not have a hit
        assert_eq!(
            cache.check_cache(&program_id, &seed, hop_index + 1, split_index, &pda),
            None
        );
    }
}
