//! Utility modules for BlackoutSOL
//!
//! This module contains various utility modules that provide common functionality
//! used throughout the BlackoutSOL program.

pub mod error_handling;
pub mod logging;
pub mod crypto;

// Re-export crypto functions
pub use crypto::{
    verify_hyperplonk_proof,
    verify_range_proof,
    extract_split_amount,
    derive_stealth_pda,
    calculate_optimized_priority_fees,
    generate_bloom_filter,
    check_bloom_filter,
    extract_splits,
    verify_pda_derivation,
    calculate_fees,
};

// Re-export the most commonly used items
pub use error_handling::{
    ErrorContext,
    ResultExt,
    IntoBlackoutError,
    err,
};

pub use logging::{
    init_logging,
    debug,
    info,
    warn,
    error,
    TransferContext,
    Loggable,
};

/// Common result type for the Blackout program
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

/// Helper macro for creating error contexts
#[macro_export]
macro_rules! context {
    ($msg:expr) => {
        |e| ErrorContext::new(e, $msg)
    };
    ($msg:expr, $($arg:tt)*) => {
        |e| ErrorContext::new(e, format!($msg, $($arg)*))
    };
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn test_context_macro() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        
        let result: Result<(), _> = Err(io_error)
            .map_err(context!("Failed to read config file"));
            
        assert!(result.is_err());
        
        if let Err(e) = result {
            assert_eq!(e.context, "Failed to read config file");
        }
    }
}
