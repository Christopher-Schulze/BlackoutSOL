//! State management for BlackoutSOL
//!
//! This module defines the on-chain state structures used by the BlackoutSOL program.

use anchor_lang::prelude::*;
use std::mem;

/// Configuration for transfer parameters
#[account]
#[derive(Default)]
pub struct TransferConfig {
    /// Number of hops for each transfer
    pub num_hops: u8,
    
    /// Number of real splits per hop
    pub real_splits: u8,
    
    /// Number of fake splits per hop
    pub fake_splits: u8,
    
    /// Fee rate in basis points (1/100 of a percent)
    pub fee_rate: u16,
    
    /// Minimum transfer amount in lamports
    pub min_transfer_amount: u64,
    
    /// Maximum transfer amount in lamports
    pub max_transfer_amount: u64,
    
    /// Bump seed for the config account
    pub bump: u8,
}

impl TransferConfig {
    /// Calculates the total number of possible paths
    pub fn total_paths(&self) -> u64 {
        (self.real_splits as u64).pow(self.num_hops as u32)
    }
    
    /// Validates the configuration parameters
    pub fn validate(&self) -> Result<()> {
        require!(self.num_hops > 0, BlackoutError::InvalidConfig);
        require!(self.real_splits > 0, BlackoutError::InvalidConfig);
        require!(self.fake_splits > 0, BlackoutError::InvalidConfig);
        require!(self.fee_rate <= 10_000, BlackoutError::InvalidConfig); // Max 100%
        require!(self.min_transfer_amount > 0, BlackoutError::InvalidConfig);
        require!(
            self.max_transfer_amount > self.min_transfer_amount,
            BlackoutError::InvalidConfig
        );
        
        Ok(())
    }
}

/// State of a transfer
#[account]
#[derive(Default)]
pub struct TransferState {
    /// Unique identifier for this transfer
    pub transfer_id: u64,
    
    /// Owner of the transfer
    pub owner: Pubkey,
    
    /// Primary recipient of the transfer
    pub primary_recipient: Pubkey,
    
    /// Total amount being transferred (in lamports)
    pub amount: u64,
    
    /// Fee amount (in lamports)
    pub fee_amount: u64,
    
    /// Current state of the transfer
    pub state: TransferStateStatus,
    
    /// Number of hops completed
    pub hops_completed: u8,
    
    /// Timestamp when the transfer was created
    pub created_at: i64,
    
    /// Timestamp when the transfer was last updated
    pub updated_at: i64,
    
    /// Bump seed for the transfer account
    pub bump: u8,
}

/// Status of a transfer
#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum TransferStateStatus {
    /// Transfer has been initialized but not started
    Initialized,
    
    /// Transfer is in progress
    InProgress,
    
    /// Transfer has been completed successfully
    Completed,
    
    /// Transfer failed and was reverted
    Failed,
    
    /// Transfer was cancelled
    Cancelled,
}

impl Default for TransferStateStatus {
    fn default() -> Self {
        Self::Initialized
    }
}

impl TransferState {
    /// Size of the TransferState account
    pub const SIZE: usize = 8 + // discriminator
        mem::size_of::<u64>() + // transfer_id
        mem::size_of::<Pubkey>() * 2 + // owner + primary_recipient
        mem::size_of::<u64>() * 2 + // amount + fee_amount
        mem::size_of::<u8>() * 3 + // state + hops_completed + bump
        mem::size_of::<i64>() * 2; // created_at + updated_at
    
    /// Initializes a new transfer state
    pub fn initialize(
        &mut self,
        transfer_id: u64,
        owner: Pubkey,
        primary_recipient: Pubkey,
        amount: u64,
        fee_amount: u64,
        bump: u8,
        clock: &Clock,
    ) {
        self.transfer_id = transfer_id;
        self.owner = owner;
        self.primary_recipient = primary_recipient;
        self.amount = amount;
        self.fee_amount = fee_amount;
        self.state = TransferStateStatus::Initialized;
        self.hops_completed = 0;
        self.created_at = clock.unix_timestamp;
        self.updated_at = clock.unix_timestamp;
        self.bump = bump;
    }
    
    /// Updates the transfer state
    pub fn update_state(&mut self, new_state: TransferStateStatus, clock: &Clock) {
        self.state = new_state;
        self.updated_at = clock.unix_timestamp;
    }
    
    /// Increments the hop counter
    pub fn increment_hops(&mut self, clock: &Clock) -> Result<()> {
        self.hops_completed = self.hops_completed.checked_add(1).ok_or(BlackoutError::ArithmeticOverflow)?;
        self.updated_at = clock.unix_timestamp;
        Ok(())
    }
}

/// Error type for Blackout program
#[error_code]
pub enum BlackoutError {
    /// Invalid configuration
    #[msg("Invalid configuration")]
    InvalidConfig,
    
    /// Arithmetic overflow
    #[msg("Arithmetic overflow")]
    ArithmeticOverflow,
    
    /// Invalid proof
    #[msg("Invalid proof")]
    InvalidProof,
    
    /// Invalid range proof
    #[msg("Invalid range proof")]
    InvalidRangeProof,
    
    /// Transfer not found
    #[msg("Transfer not found")]
    TransferNotFound,
    
    /// Unauthorized
    #[msg("Unauthorized")]
    Unauthorized,
    
    /// Invalid state transition
    #[msg("Invalid state transition")]
    InvalidStateTransition,
    
    /// Invalid recipient
    #[msg("Invalid recipient")]
    InvalidRecipient,
    
    /// Insufficient funds
    #[msg("Insufficient funds")]
    InsufficientFunds,
    
    /// Transfer already completed
    #[msg("Transfer already completed")]
    TransferCompleted,
    
    /// Transfer expired
    #[msg("Transfer expired")]
    TransferExpired,
}
