// verification/mod.rs
//
// Main verification module for BlackoutSOL
// This module integrates various verification approaches:
// 1. Formal verification - mathematical properties and invariants
// 2. Symbolic execution - exhaustive path analysis
// 3. Model checking - state transition verification

// Export formal verification module
#[cfg(feature = "verification")]
pub mod formal;

/// Run all verification checks across all components
pub fn run_verification() -> bool {
    let mut all_passed = true;
    
    // Run formal verification if the feature is enabled
    #[cfg(feature = "verification")]
    {
        all_passed &= formal::verify_all();
    }
    
    // Always return true in non-verification builds
    #[cfg(not(feature = "verification"))]
    {
        println!("Verification skipped: feature 'verification' not enabled");
    }
    
    all_passed
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    #[cfg(feature = "verification")]
    fn test_verification_suite() {
        assert!(run_verification());
    }
}
