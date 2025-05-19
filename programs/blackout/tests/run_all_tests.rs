// Complete integration tests for BlackoutSOL
// This file executes all tests in a coordinated sequence

#[cfg(test)]
mod tests {
    use solana_program_test::*;
    use solana_sdk::{
        signature::{Keypair, Signer},
        transport::TransportError,
    };
    
    mod test_framework;
    
    // Import all test modules
    #[path = "test_init_finalize.rs"]
    mod test_init_finalize;
    
    #[path = "test_execute_hop.rs"]
    mod test_execute_hop;
    
    #[path = "test_batch_hop.rs"]
    mod test_batch_hop;
    
    #[path = "test_crypto_components.rs"]
    mod test_crypto_components;
    
    // New optimized test modules
    #[path = "test_refund_functionalities.rs"]
    mod test_refund_functionalities;
    
    #[path = "test_config_update.rs"]
    mod test_config_update;
    
    #[path = "test_reveal_fake.rs"]
    mod test_reveal_fake;
    
    // Performance-Tests
    #[path = "test_performance.rs"]
    mod test_performance;
    
    #[tokio::test]
    async fn run_all_blackout_tests() -> Result<(), TransportError> {
        println!("=== BLACKOUT SOL COMPLETE TEST SUITE ===");
        println!("Running all tests in optimal sequence...");
        
        // Initialization and finalization tests
        println!("\n>> Test: Initialization and configuration");
        test_init_finalize::test_initialize_basic().await?;
        test_init_finalize::test_initialize_edge_cases().await?;
        
        // Configuration change tests (new optimized suite)
        println!("\n>> Test: Optimized configuration changes");
        test_config_update::test_config_update_by_owner().await?;
        test_config_update::test_config_update_by_admin().await?;
        test_config_update::test_config_update_unauthorized().await?;
        test_config_update::test_config_update_invalid_params().await?;
        test_config_update::test_config_update_after_transfer_started().await?;
        
        // Backward compatibility test for configuration changes
        test_init_finalize::test_update_config().await?;
        
        // Single-Hop Tests
        println!("\n>> Test: Single hop execution");
        test_execute_hop::test_single_hop_execution().await?;
        test_execute_hop::test_hop_error_conditions().await?;
        test_execute_hop::test_fake_split_handling().await?;
        
        // Complete hop sequence and finalization
        println!("\n>> Test: Complete hop sequence and finalization");
        test_execute_hop::test_complete_hop_sequence().await?;
        test_init_finalize::test_finalize_after_hop().await?;
        test_init_finalize::test_finalize_with_zero_reserve().await?;
        
        // Optimized refund functionality (new optimized suite)
        println!("\n>> Test: Optimized refund functionality");
        test_refund_functionalities::test_refund_full_process().await?;
        test_refund_functionalities::test_refund_immediately_after_init().await?;
        test_refund_functionalities::test_finalize_with_reserve().await?;
        
        // Backward compatibility test for refunds
        println!("\n>> Test: Legacy refund functionality");
        test_execute_hop::test_refund_functionality().await?;
        
        // Fake-split disclosure tests (new optimized suite)
        println!("\n>> Test: Optimized fake split disclosure");
        test_reveal_fake::test_reveal_fake_success().await?;
        test_reveal_fake::test_reveal_real_as_fake().await?;
        test_reveal_fake::test_reveal_non_bloom_fake().await?;
        test_reveal_fake::test_reveal_fake_wrong_pda().await?;
        
        // Batch-Hop Tests
        println!("\n>> Test: Batch hop basic functionality");
        test_batch_hop::test_batch_hop_basic().await?;
        test_batch_hop::test_batch_hop_multi_batch().await?;
        
        // Advanced Batch-Hop Tests
        println!("\n>> Test: Advanced batch hop functions");
        test_batch_hop::test_batch_hop_fake_splits().await?;
        test_batch_hop::test_batch_hop_error_conditions().await?;
        test_batch_hop::test_batch_hop_maximum_capacity().await?;
        
        // Cryptographic component tests
        println!("\n>> Test: Cryptographic components");
        test_crypto_components::test_hyperplonk_proof_verification().await?;
        test_crypto_components::test_range_proof_verification().await?;
        test_crypto_components::test_bloom_filter_functionality().await?;
        
        // Performance tests (optional - can be disabled if needed)
        println!("\n>> Test: Performance optimization (may take some time)");
        test_performance::test_compute_unit_efficiency().await?;
        test_performance::test_batch_processing_efficiency().await?;
        
        // Advanced performance tests can be enabled on demand
        if std::env::var("RUN_EXTENDED_PERF_TESTS").is_ok() {
            println!("\n>> Test: Advanced performance tests");
            test_performance::test_parallel_execution_under_load().await?;
            test_performance::test_priority_fee_optimization().await?;
        }
        
        println!("\n=== ALL TESTS SUCCESSFULLY COMPLETED ===");
        Ok(())
    }
}
