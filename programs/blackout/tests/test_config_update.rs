/// Tests for the optimized config update functionality
///
/// These tests validate the correct functionality of configuration changes,
/// including authorization checks and parameter validation.

use anchor_lang::prelude::*;
use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentLevel,
};

mod test_framework;
use test_framework::BlackoutTestFramework;
use blackout::{
    state::*,
    errors::BlackoutError,
    instructions::*,
};

// Test for a successful configuration change by the owner
#[tokio::test]
async fn test_config_update_by_owner() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Initialize transfer with standard configuration
    let amount = 1_000_000_000; // 1 SOL in Lamports
    let initial_reserve = 10; // 10% Reserve as initial value
    
    // Initialize transfer
    let (transfer_pda, _) = framework.initialize_transfer(amount, initial_reserve)
        .await
        .expect("Transfer initialization failed");
    
    // 3. Check initial transfer state
    let initial_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve initial transfer state");
    
    assert_eq!(initial_state.config.reserve_percent, initial_reserve, 
              "Initial reserve was not set correctly");
    assert_eq!(initial_state.config.fee_multiplier, 500, 
              "Initial fee multiplier does not match the default value");
    
    // 4. Change configuration
    let new_reserve = 30; // 30% Reserve
    let new_fee = 800; // 8% fee (800 basis points)
    let new_cu_budget = 450_000; // 450k Compute Units per hop
    
    // Create update parameters
    let update_params = ConfigUpdateParams {
        reserve_percent: Some(new_reserve),
        fee_multiplier: Some(new_fee),
        cu_budget_per_hop: Some(new_cu_budget),
    };
    
    // Update as owner
    framework.update_config(&transfer_pda, update_params, &framework.user)
        .await
        .expect("Configuration change as owner failed");
    
    // 5. Check updated transfer state
    let updated_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve updated transfer state");
    
    assert_eq!(updated_state.config.reserve_percent, new_reserve, 
              "Reserve was not updated correctly");
    assert_eq!(updated_state.config.fee_multiplier, new_fee, 
              "Fee multiplier was not updated correctly");
    assert_eq!(updated_state.config.cu_budget_per_hop, new_cu_budget, 
              "CU budget was not updated correctly");
    
    println!("Test successful: Configuration change by owner works correctly");
}

// Test for a configuration change by an admin (not the owner)
#[tokio::test]
async fn test_config_update_by_admin() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Create admin account
    let admin = Keypair::new();
    
    // Fund admin account with Lamports
    framework.fund_account(&admin.pubkey(), 100_000_000).await
        .expect("Could not fund admin account");
    
    // 3. Initialize transfer with standard configuration
    let amount = 500_000_000; // 0.5 SOL in Lamports
    let initial_reserve = 15; // 15% Reserve as initial value
    
    // Initialize transfer
    let (transfer_pda, _) = framework.initialize_transfer(amount, initial_reserve)
        .await
        .expect("Transfer initialization failed");
    
    // 4. Change configuration
    let new_reserve = 20; // 20% Reserve
    let new_fee = 600; // 6% fee (600 basis points)
    
    // Create update parameters (not changing all fields)
    let update_params = ConfigUpdateParams {
        reserve_percent: Some(new_reserve),
        fee_multiplier: Some(new_fee),
        cu_budget_per_hop: None, // Leave this parameter unchanged
    };
    
    // Update as admin with admin instead of owner
    framework.update_config_with_admin(&transfer_pda, update_params, &admin, &admin)
        .await
        .expect("Configuration change as admin failed");
    
    // 5. Check updated transfer state
    let updated_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve updated transfer state");
    
    assert_eq!(updated_state.config.reserve_percent, new_reserve, 
              "Reserve was not updated correctly");
    assert_eq!(updated_state.config.fee_multiplier, new_fee, 
              "Fee multiplier was not updated correctly");
    assert_eq!(updated_state.config.cu_budget_per_hop, 300_000, 
              "CU budget should remain unchanged");
    
    println!("Test successful: Configuration change by admin works correctly");
}

// Test for attempting a configuration change by an unauthorized user
#[tokio::test]
async fn test_config_update_unauthorized() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Create two different accounts
    let unauthorized_user = Keypair::new();
    let admin = Keypair::new();
    
    // Fund accounts with Lamports
    framework.fund_account(&unauthorized_user.pubkey(), 100_000_000).await
        .expect("Could not fund account");
    framework.fund_account(&admin.pubkey(), 100_000_000).await
        .expect("Could not fund admin account");
    
    // 3. Initialize transfer
    let amount = 200_000_000; // 0.2 SOL
    let initial_reserve = 10; // 10% Reserve
    
    // Initialize transfer
    let (transfer_pda, _) = framework.initialize_transfer(amount, initial_reserve)
        .await
        .expect("Transfer initialization failed");
    
    // 4. Attempt to change configuration by unauthorized user
    let update_params = ConfigUpdateParams {
        reserve_percent: Some(40),
        fee_multiplier: Some(700),
        cu_budget_per_hop: Some(400_000),
    };
    
    // Attempt by unauthorized user (neither owner nor admin)
    let result = framework.update_config_with_admin(
        &transfer_pda, 
        update_params, 
        &unauthorized_user, 
        &admin
    ).await;
    
    // 5. Check if the attempt failed
    assert!(
        result.is_err(),
        "Configuration change by unauthorized user should fail"
    );
    
    // 6. Check original transfer state (should remain unchanged)
    let unchanged_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state");
    
    assert_eq!(unchanged_state.config.reserve_percent, initial_reserve, 
              "Reserve should remain unchanged");
    
    println!("Test successful: Configuration change by unauthorized user was prevented");
}

// Test for attempting to set invalid configuration parameters
#[tokio::test]
async fn test_config_update_invalid_params() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Initialize transfer
    let amount = 300_000_000; // 0.3 SOL
    let initial_reserve = 10; // 10% Reserve
    
    // Initialize transfer
    let (transfer_pda, _) = framework.initialize_transfer(amount, initial_reserve)
        .await
        .expect("Transfer initialization failed");
    
    // 3. Attempt to set invalid configuration parameters
    
    // a) Too high reserve percentage (above 80%)
    let invalid_reserve_params = ConfigUpdateParams {
        reserve_percent: Some(85), // 85% (invalid, max is 80%)
        fee_multiplier: None,
        cu_budget_per_hop: None,
    };
    
    let result_reserve = framework.update_config(&transfer_pda, invalid_reserve_params, &framework.user)
        .await;
    
    assert!(
        result_reserve.is_err(),
        "Update with invalid reserve percentage should fail"
    );
    
    // b) Too high fee multiplier (above 1000 BP)
    let invalid_fee_params = ConfigUpdateParams {
        reserve_percent: None,
        fee_multiplier: Some(1200), // 12% (invalid, max is 10% = 1000 BP)
        cu_budget_per_hop: None,
    };
    
    let result_fee = framework.update_config(&transfer_pda, invalid_fee_params, &framework.user)
        .await;
    
    assert!(
        result_fee.is_err(),
        "Update with invalid fee multiplier should fail"
    );
    
    // c) Invalid CU budget (too low)
    let invalid_cu_params = ConfigUpdateParams {
        reserve_percent: None,
        fee_multiplier: None,
        cu_budget_per_hop: Some(50_000), // 50k (invalid, min is 100k)
    };
    
    let result_cu = framework.update_config(&transfer_pda, invalid_cu_params, &framework.user)
        .await;
    
    assert!(
        result_cu.is_err(),
        "Update with invalid CU budget should fail"
    );
    
    // 4. Check if the original configuration remains unchanged
    let unchanged_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state");
    
    assert_eq!(unchanged_state.config.reserve_percent, initial_reserve, 
              "Reserve should remain unchanged");
    
    println!("Test successful: Invalid configuration parameters were rejected");
}

// Test for attempting to change the configuration after the transfer has started
#[tokio::test]
async fn test_config_update_after_transfer_started() {
    // 1. Initialize test framework
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Initialize transfer
    let amount = 400_000_000; // 0.4 SOL
    let initial_reserve = 10; // 10% Reserve
    
    // Initialize transfer
    let (transfer_pda, seed) = framework.initialize_transfer(amount, initial_reserve)
        .await
        .expect("Transfer initialization failed");
    
    // 3. Execute the first hop
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Execution of the first hop failed");
    
    // 4. Attempt to change the configuration after the transfer has started
    let update_params = ConfigUpdateParams {
        reserve_percent: Some(20),
        fee_multiplier: None,
        cu_budget_per_hop: None,
    };
    
    let result = framework.update_config(&transfer_pda, update_params, &framework.user)
        .await;
    
    // 5. Check if the attempt failed
    assert!(
        result.is_err(),
        "Configuration change after transfer start should fail"
    );
    
    // 6. Check if the original configuration remains unchanged
    let unchanged_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Could not retrieve transfer state");
    
    assert_eq!(unchanged_state.config.reserve_percent, initial_reserve, 
              "Reserve should remain unchanged");
    assert_eq!(unchanged_state.current_hop, 1, 
              "One hop should have been executed successfully");
    
    println!("Test successful: Configuration change after transfer start was prevented");
}

// Extend the BlackoutTestFramework with the config update functionality
impl BlackoutTestFramework {
    // Update configuration as owner
    pub async fn update_config(
        &mut self,
        transfer_pda: &Pubkey,
        params: ConfigUpdateParams,
        authority: &Keypair,
    ) -> Result<(), BanksClientError> {
        // Admin is the same as the owner in this case (self-management)
        self.update_config_with_admin(transfer_pda, params, authority, authority).await
    }
    
    // Update configuration with separate admin
    pub async fn update_config_with_admin(
        &mut self,
        transfer_pda: &Pubkey,
        params: ConfigUpdateParams,
        authority: &Keypair,
        admin: &Keypair,
    ) -> Result<(), BanksClientError> {
        // Create configuration update instruction
        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(authority.pubkey(), true),
                AccountMeta::new(admin.pubkey(), false),
                AccountMeta::new(transfer_pda.clone(), false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: blackout::instruction::ConfigUpdate {
                update_params: params,
            }.data(),
        };
        
        // Transaktion ausführen mit entsprechenden Signaturen
        let mut signers = vec![authority];
        if admin.pubkey() != authority.pubkey() {
            signers.push(admin);
        }
        
        self.execute_transaction(&[ix], &signers).await
    }
}
