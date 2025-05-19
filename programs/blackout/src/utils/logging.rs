//! Structured logging utilities for BlackoutSOL
//! 
//! This module provides a consistent way to log events and errors throughout
//! the application, with support for structured logging and context propagation.

use std::fmt::Debug;
use std::time::{SystemTime, UNIX_EPOCH};
use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

/// Initializes the logging subsystem
/// 
/// # Errors
/// Returns an error if the logging subsystem fails to initialize
pub fn init_logging() -> Result<()> {
    // In a real implementation, this would set up the logging backend
    // For now, we'll just print a message
    println!("Logging initialized");
    Ok(())
}

/// Logs a debug message with structured context
#[macro_export]
macro_rules! debug {
    ($($arg:tt)*) => {
        #[cfg(debug_assertions)]
        println!("[DEBUG] {}", format_args!($($arg)*));
    };
}

/// Logs an info message with structured context
#[macro_export]
macro_rules! info {
    ($($arg:tt)*) => {
        println!("[INFO] {}", format_args!($($arg)*));
    };
}

/// Logs a warning with structured context
#[macro_export]
macro_rules! warn {
    ($($arg:tt)*) => {
        eprintln!("[WARN] {}", format_args!($($arg)*));
    };
}

/// Logs an error with structured context
#[macro_export]
macro_rules! error {
    ($($arg:tt)*) => {
        eprintln!("[ERROR] {}", format_args!($($arg)*));
    };
}

/// Context for logging transfer-related events
#[derive(Debug, Clone, serde::Serialize)]
pub struct TransferContext {
    /// Unique identifier for the transfer
    pub transfer_id: String,
    
    /// Amount being transferred
    pub amount: u64,
    
    /// Sender's public key
    pub sender: Option<Pubkey>,
    
    /// Recipient's public key
    pub recipient: Option<Pubkey>,
    
    /// Additional context data
    pub metadata: std::collections::HashMap<String, String>,
    
    /// Timestamp of the transfer
    pub timestamp: u64,
}

impl TransferContext {
    /// Creates a new TransferContext
    /// 
    /// # Arguments
    /// * `transfer_id` - Unique identifier for the transfer
    /// * `amount` - Amount being transferred
    pub fn new(transfer_id: &str, amount: u64) -> Self {
        Self {
            transfer_id: transfer_id.to_string(),
            amount,
            sender: None,
            recipient: None,
            metadata: std::collections::HashMap::new(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        }
    }

    /// Sets the sender
    pub fn with_sender(mut self, sender: Pubkey) -> Self {
        self.sender = Some(sender);
        self
    }

    /// Sets the recipient
    pub fn with_recipient(mut self, recipient: Pubkey) -> Self {
        self.recipient = Some(recipient);
        self
    }
    
    /// Adds metadata to the context
    pub fn with_metadata<K: Into<String>, V: Into<String>>(mut self, key: K, value: V) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }
    
    /// Gets a metadata value by key
    pub fn get_metadata(&self, key: &str) -> Option<&String> {
        self.metadata.get(key)
    }
    
    /// Converts the context to a JSON string
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| "{}".to_string())
    }

    /// Logs a transfer-related info message
    pub fn info(&self, message: &str) {
        info!(
            "[Transfer {}] {} (Amount: {}, Sender: {}, Recipient: {})",
            self.transfer_id,
            message,
            self.amount,
            self.sender.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()),
            self.recipient.map(|r| r.to_string()).unwrap_or_else(|| "unknown".to_string())
        );
    }

    /// Logs a transfer-related warning
    pub fn warn(&self, message: &str) {
        warn!(
            "[Transfer {}] {} (Amount: {}, Sender: {}, Recipient: {})",
            self.transfer_id,
            message,
            self.amount,
            self.sender.map(|s| s.to_string()).unwrap_or_else(|| "unknown".to_string()),
            self.recipient.map(|r| r.to_string()).unwrap_or_else(|| "unknown".to_string())
        );
    }

    /// Logs a transfer-related error
    pub fn error(&self, error: &(impl Debug + ?Sized), context: &str) {
        error!(
            "[Transfer {}] {}: {:?} (Amount: {}, Sender: {:?}, Recipient: {:?})",
            self.transfer_id, message, error, self.amount, self.sender, self.recipient
        );
    }
}

/// Trait for types that can be converted to a loggable format
pub trait Loggable {
    /// Returns a string representation suitable for logging
    fn to_log_string(&self) -> String;
}

impl Loggable for Pubkey {
    fn to_log_string(&self) -> String {
        self.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use solana_program::pubkey;

    #[test]
    fn test_transfer_context() {
        let sender = pubkey!("11111111111111111111111111111111");
        let recipient = pubkey!("22222222222222222222222222222222");
        
        let ctx = TransferContext::new("test123", 1000)
            .with_sender(sender)
            .with_recipient(recipient);
            
        // These will print during tests but that's fine for demonstration
        ctx.info("Test message");
        ctx.warn("Test warning");
        ctx.error(&std::io::Error::new(std::io::ErrorKind::Other, "test error"), "Test error");
    }
}
