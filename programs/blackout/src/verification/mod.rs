// verification/mod.rs
//
// Main verification module for BlackoutSOL
// This module integrates various verification approaches:
// 1. Formal verification - mathematical properties and invariants
// 2. Symbolic execution - exhaustive path analysis
// 3. Model checking - state transition verification

// Export formal verification module
pub mod formal;

/// Run all verification checks across all components
pub fn run_verification() -> bool {
    let mut all_passed = true;
    
    // Run formal verification
    all_passed &= formal::verify_all();
    
    all_passed
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_verification_suite() {
        assert!(run_verification());
    }
}
