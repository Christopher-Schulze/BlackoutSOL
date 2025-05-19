// Use Anchor import directly to ensure compatibility
use anchor_lang::prelude::*;

/// Error codes for the Blackout system
#[error_code]
pub enum BlackoutError {
    /// Proof has an invalid size
    #[msg("Invalid proof size. Expected: 128 bytes")]
    InvalidProofSize,
    
    /// Proof verification failed
    #[msg("Zero-Knowledge proof verification failed")]
    ProofVerificationFailed,
    
    /// Not enough compute units available
    #[msg("Insufficient compute units. At least 120k required")]
    InsufficientComputeUnits,
    
    /// Hop index is invalid or outside the expected range
    #[msg("Invalid hop index for the current transfer state")]
    InvalidHopIndex,
    
    /// Transfer has not been fully completed yet
    #[msg("Transfer not ready for finalization. All hops must be completed")]
    TransferNotComplete,
    
    /// The submitted amount is invalid
    #[msg("Invalid amount. Must be greater than 0 and below the maximum")]
    InvalidAmount,
    
    /// The recipient is invalid
    #[msg("Invalid recipient. Cannot be identical to the sender")]
    InvalidRecipient,
    
    /// Transfer has already been completed
    #[msg("Transfer already completed. No further hops possible")]
    TransferAlreadyCompleted,
    
    /// Range proof verification failed
    #[msg("Range proof verification failed. Split amounts may be invalid")]
    RangeProofVerificationFailed,
    
    /// The batch configuration is invalid
    #[msg("Invalid batch configuration. Check num_hops and splits")]
    InvalidBatchConfiguration,
    
    /// Error creating the PDA
    #[msg("Error creating the stealth PDA. Internal program error")]
    PdaCreationError,
    
    /// Error closing the PDA
    #[msg("Error closing the stealth PDA. Internal program error")]
    PdaCloseError,
    
    /// Merkle proof verification failed
    #[msg("Merkle proof verification failed. Wallet may not be in the set")]
    MerkleProofVerificationFailed,
    
    /// Refund already triggered
    #[msg("Refund has already been triggered. No further action possible")]
    RefundAlreadyTriggered,
    
    /// Bloom filter error
    #[msg("Error creating or verifying the bloom filter for fake splits")]
    BloomFilterError,
    
    /// The challenge data is invalid
    #[msg("Invalid challenge data for ZK proofs")]
    InvalidChallenge,
    
    /// Not enough lamports available
    #[msg("Insufficient lamports for the operation")]
    InsufficientLamports,
    
    /// The maximum number of batch runs has been exceeded
    #[msg("Maximum number of batch runs exceeded")]
    MaxBatchesExceeded,
    
    /// Error deserializing data
    #[msg("Error deserializing data")]
    DeserializationError,
    
    /// The commitment data is invalid
    #[msg("Invalid commitment data")]
    InvalidCommitment,
    
    /// Internal error in the proof system
    #[msg("Internal error in the Zero-Knowledge proof system")]
    ProofSystemError,
    
    /// Missing or invalid authorization
    #[msg("Missing or invalid authorization for this operation")]
    UnauthorizedAccess,
    
    /// Transfer of split funds failed
    #[msg("Failed to transfer funds during split operation")]
    SplitTransferFailed,
    
    /// Transfer has been refunded and cannot proceed
    #[msg("This transfer has been refunded and cannot be processed further")]
    TransferRefunded,
    
    /// Batch hop execution failed
    #[msg("Batch hop execution failed. Check logs for details")]
    BatchExecutionFailed,
    
    /// Invalid PDA ownership
    #[msg("PDA account is not owned by the expected program")]
    InvalidPdaOwnership,
    
    /// Invalid PDA derivation
    #[msg("PDA was not correctly derived from the expected seeds")]
    InvalidPdaDerivation,
    
    /// Hashing operation failed
    #[msg("Fehler bei kryptografischer Hash-Operation")]
    HashingError,
    
    /// Invalid parameters provided
    #[msg("Ungültige Parameter angegeben")]
    InvalidParameters,
    
    /// Invalid PDA account
    #[msg("Ungültige PDA-Adresse oder -Daten")]
    InvalidPda,
}