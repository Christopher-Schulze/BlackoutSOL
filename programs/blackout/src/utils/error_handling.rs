//! Enhanced error handling for BlackoutSOL
//! 
//! This module provides utilities for consistent error handling and reporting
//! across the application, including error chaining and context preservation.

use std::fmt::{self, Debug, Display, Formatter};
use std::error::Error as StdError;
use std::backtrace::{Backtrace, BacktraceStatus};
use std::time::{SystemTime, UNIX_EPOCH};

use anchor_lang::prelude::*;
use solana_program::pubkey::Pubkey;

/// A wrapper for errors that includes additional context
#[derive(Debug)]
pub struct ErrorContext {
    source: Box<dyn StdError + Send + Sync + 'static>,
    context: String,
    backtrace: Backtrace,
    timestamp: u64,
    error_code: Option<String>,
    additional_info: Option<String>,
}

impl ErrorContext {
    /// Creates a new error context
    pub fn new<E>(error: E, context: impl Into<String>) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        Self {
            source: Box::new(error),
            context: context.into(),
            backtrace: Backtrace::capture(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
            error_code: None,
            additional_info: None,
        }
    }

    /// Adds an error code to the context
    pub fn with_error_code(mut self, code: impl Into<String>) -> Self {
        self.error_code = Some(code.into());
        self
    }

    /// Adds additional information to the context
    pub fn with_info(mut self, info: impl Into<String>) -> Self {
        self.additional_info = Some(info.into());
        self
    }

    /// Returns the error code if present
    pub fn error_code(&self) -> Option<&str> {
        self.error_code.as_deref()
    }

    /// Returns the additional information if present
    pub fn additional_info(&self) -> Option<&str> {
        self.additional_info.as_deref()
    }

    /// Returns the timestamp when the error occurred
    pub fn timestamp(&self) -> u64 {
        self.timestamp
    }
}

impl Display for ErrorContext {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.context)?;
        
        if let Some(code) = &self.error_code {
            write!(f, " [code: {}]", code)?;
        }
        
        if let Some(info) = &self.additional_info {
            write!(f, " (info: {})", info)?;
        }
        
        Ok(())
    }
}

impl StdError for ErrorContext {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&*self.source)
    }

    fn backtrace(&self) -> Option<&Backtrace> {
        match self.backtrace.status() {
            BacktraceStatus::Captured => Some(&self.backtrace),
            _ => None,
        }
    }
}

/// Extension trait for Result to add context to errors
pub trait ResultExt<T, E> {
    /// Adds context to an error if the result is an error
    fn context<C>(self, context: C) -> Result<T, ErrorContext>
    where
        C: Into<String>;
}

impl<T, E> ResultExt<T, E> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, ErrorContext>
    where
        C: Into<String>,
    {
        self.map_err(|e| ErrorContext::new(e, context.into()))
    }
}

/// Converts an error into a BlackoutError with context
pub trait IntoBlackoutError<T> {
    /// Converts the error into a BlackoutError with the given context
    fn blackout_err(self, context: &str) -> Result<T, ErrorContext>;
}

impl<T, E> IntoBlackoutError<T> for Result<T, E>
where
    E: StdError + Send + Sync + 'static,
{
    fn blackout_err(self, context: &str) -> Result<T, ErrorContext> {
        self.map_err(|e| ErrorContext::new(e, context.to_string()))
    }
}

/// Helper function to create a new error with context
pub fn err<T, E>(error: E, context: impl Into<String>) -> Result<T, ErrorContext>
where
    E: StdError + Send + Sync + 'static,
{
    Err(ErrorContext::new(error, context.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;\n
    #[test]
    fn test_error_context() {
        let io_error = io::Error::new(io::ErrorKind::NotFound, "file not found");
        
        let result: Result<(), _> = Err(io_error)
            .context("Failed to read config file")
            .map_err(|e| e.with_error_code("CONFIG_READ_ERROR"));

        assert!(result.is_err());
        
        if let Err(context) = result {
            assert!(context.source().is_some());
            assert_eq!(context.error_code(), Some("CONFIG_READ_ERROR"));
            assert!(context.backtrace().is_some());
        }
    }

    #[test]
    fn test_into_blackout_error() {
        let result: Result<(), io::Error> = Err(io::Error::new(io::ErrorKind::PermissionDenied, "access denied"));
        
        let result = result.blackout_err("Failed to access file");
        
        assert!(result.is_err());
        if let Err(context) = result {
            assert_eq!(context.context, "Failed to access file");
            assert!(matches!(
                context.source().unwrap().downcast_ref::<io::Error>().unwrap().kind(),
                io::ErrorKind::PermissionDenied
            ));
        }
    }
}
