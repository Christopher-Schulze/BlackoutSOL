// This file serves as a bridge between the Anchor-specific implementation and
// the pure core implementation of the state objects.

use crate::anchor_types;

// Feature-dependent import conditions
#[cfg(feature = "no-anchor")]
use solana_program::{
    account_info::AccountInfo,
    program_error::ProgramError,
    pubkey::Pubkey,
};

// Feature-dependent public exports for account attributes
#[cfg(not(feature = "no-anchor"))]
pub use anchor_lang::prelude::account;
#[cfg(feature = "no-anchor")]
#[macro_export]
macro_rules! account {
    ($item:item) => {
        #[derive(borsh::BorshDeserialize, borsh::BorshSerialize)]
        $item
    };
}

// Unified error transformation
#[cfg(not(feature = "no-anchor"))]
pub fn map_to_error<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> anchor_lang::Result<T> {
    result.map_err(|e| {
        anchor_lang::prelude::msg!("Error: {:?}", e);
        anchor_lang::error::Error::from(crate::errors::BlackoutError::DeserializationError)
    })
}

#[cfg(feature = "no-anchor")]
pub fn map_to_error<T, E: std::fmt::Debug>(result: std::result::Result<T, E>) -> Result<T, ProgramError> {
    result.map_err(|e| {
        solana_program::msg!("Error: {:?}", e);
        ProgramError::InvalidAccountData
    })
}

// Anchor-independent state validation
pub trait StateValidation {
    fn validate_owner(&self, expected_owner: &Pubkey) -> bool;
    fn validate_state(&self) -> bool;
}

// Helper function for account operations
#[cfg(not(feature = "no-anchor"))]
pub fn init_account<'info, T: anchor_lang::Owner>(
    account: &mut anchor_lang::Account<'info, T>,
    data: T,
) -> anchor_lang::Result<()> {
    *account = anchor_lang::Account::try_from(data)?;
    Ok(())
}

#[cfg(feature = "no-anchor")]
pub fn init_account<T: borsh::BorshSerialize>(
    account: &AccountInfo,
    data: &T,
) -> Result<(), ProgramError> {
    let data_slice = &mut account.data.borrow_mut();
    data.serialize(&mut &mut data_slice[..])
        .map_err(|_| ProgramError::InvalidAccountData)
}

// Re-export of constants for both modes
pub const DISCRIMINATOR_SIZE: usize = 8;
