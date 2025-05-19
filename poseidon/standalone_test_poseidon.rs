// Standalone Poseidon test without dependency on the BlackoutSOL library
// This script can be run directly with cargo run

use solana_poseidon::{hashv, Parameters, Endianness};
use curve25519_dalek::scalar::Scalar;
use num_bigint::BigUint;

fn main() {
    println!("🧪 Starting standalone Poseidon tests...");
    
    test_poseidon_functionality().unwrap_or_else(|e| {
        eprintln!("❌ Test failed: {}", e);
        std::process::exit(1);
    });
}

// ==== Simple Test Functions ====

fn test_poseidon_functionality() -> Result<(), String> {
    println!("\n🔍 Test 1: Simple Hash Calculation");
    test_simple_hash()?;
    
    println!("\n🔄 Test 2: Consistent Results");
    test_consistent_hash()?;
    
    println!("\n📊 Test 3: Poseidon Constants");
    test_constants()?;
    
    println!("\n🎉 All tests completed successfully!");
    Ok(())
}

fn test_simple_hash() -> Result<(), String> {
    // Simple input for the hash
    let mut test_input = [0u8; 32];
    test_input[31] = 0x42; // A simple value for testing
    
    println!("   Calculating hash for simple input...");
    let hash_result = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&test_input]
    ).map_err(|e| format!("Error during hash calculation: {:?}", e))?;
    
    println!("   ✅ Hash successfully calculated: {:?}", hash_result);
    Ok(())
}

fn test_consistent_hash() -> Result<(), String> {
    // We should get the same hash for the same input
    let test_input = b"BlackoutSOL-Test-Input";
    
    println!("   Calculating first hash...");
    let hash1 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).map_err(|e| format!("Error during first hash calculation: {:?}", e))?;
    
    println!("   Calculating second hash with same input...");
    let hash2 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).map_err(|e| format!("Error during second hash calculation: {:?}", e))?;
    
    if hash1 == hash2 {
        println!("   ✅ Hash consistency confirmed");
        Ok(())
    } else {
        Err("   ❌ Hash consistency check failed - same input produced different hashes".to_string())
    }
}

fn test_constants() -> Result<(), String> {
    // Just a few test constants to verify functionality
    let mds_hex = [
        ["3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd", 
         "3798866f4e6058035dcf8addb2cf1771fac234bcc8fc05d6676e77e797f224bf"],
        ["20088ca07bbcd7490a0218ebc0ecb31d0ea34840e2dc2d33a1a5adfecff83b43", 
         "1d04ba0915e7807c968ea4b1cb2d610c7f9a16b4033f02ebacbb948c86a988c3"],
    ];
    
    println!("   Converting hex strings to Scalar values...");
    let mds_matrix: Vec<Vec<Scalar>> = mds_hex.iter()
        .map(|row| row.iter().map(|&s| scalar_from_hex(s)).collect())
        .collect();
    
    println!("   ✅ Successfully created MDS matrix ({} x {} elements)", 
             mds_matrix.len(), 
             if mds_matrix.len() > 0 { mds_matrix[0].len() } else { 0 });
    Ok(())
}

// ==== Helper Functions ====

fn scalar_from_hex(s: &str) -> Scalar {
    let s = if s.starts_with("0x") { &s[2..] } else { s };
    
    // Convert hexadecimal to BigUint
    let big_uint = BigUint::parse_bytes(s.as_bytes(), 16)
        .expect(&format!("Failed to parse hex string for Scalar: {}", s));
    
    // BigUint zu bytes konvertieren
    let bytes = big_uint.to_bytes_be();
    
    // Convert bytes to Scalar (updated method)
    let mut array = [0u8; 32];
    let len = std::cmp::min(bytes.len(), 32);
    array[32 - len..].copy_from_slice(&bytes[..len]);
    Scalar::from_bytes_mod_order(array)
}
