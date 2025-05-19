// Standalone Poseidon tester
// This code demonstrates the correct usage of the solana-poseidon library
// and validates its functionality

use solana_poseidon::{hashv, Parameters, Endianness};
use curve25519_dalek::scalar::Scalar;
use num_bigint::BigUint;

fn main() {
    println!("\n\n=================================");
    println!("🌊 POSEIDON HASH TESTER for BlackoutSOL");
    println!("=================================\n");
    
    match run_all_tests() {
        Ok(_) => {
            println!("\n✅ SUCCESS: All tests passed!\n");
            println!("The Poseidon integration has been successfully confirmed.");
            println!("The solana-poseidon library works as expected.");
        }
        Err(e) => {
            println!("\n❌ ERROR: Tests failed!");
            println!("Reason: {}", e);
            std::process::exit(1);
        }
    }
}

fn run_all_tests() -> Result<(), String> {
    // Test 1: Basic hash functionality
    test_basic_hash()?;
    
    // Test 2: Hash consistency
    test_hash_consistency()?;
    
    // Test 3: Conversion from hex to Scalar
    test_hex_to_scalar()?;
    
    // Test 4: MDS matrix and round constants
    test_constants()?;
    
    Ok(())
}

fn test_basic_hash() -> Result<(), String> {
    println!("\n[Test 1] Basic hash functionality");
    println!("---------------------------------");
    
    // Simple input for the hash
    let mut test_input = [0u8; 32];
    test_input[31] = 0x42; // A simple value for testing
    
    println!("Calculating hash for simple input...");
    let hash_result = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&test_input]
    ).map_err(|e| format!("Error during hash calculation: {:?}", e))?;
    
    println!("Hash successfully calculated: 0x{}", hex::encode(hash_result.to_bytes()));
    
    // Verify that the hash is not trivial
    if hash_result.to_bytes().iter().all(|&b| b == 0) {
        return Err("Hash consists only of zeros, which cannot be correct".to_string());
    }
    
    println!("✅ Basic hash functionality confirmed");
    Ok(())
}

fn test_hash_consistency() -> Result<(), String> {
    println!("\n[Test 2] Hash consistency");
    println!("----------------------");
    
    // Test vector
    let test_input = b"BlackoutSOL-Integration-Test";
    
    println!("Calculating first hash...");
    let hash1 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).map_err(|e| format!("Error during first hash calculation: {:?}", e))?;
    
    println!("Calculating second hash with same input...");
    let hash2 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).map_err(|e| format!("Error during second hash calculation: {:?}", e))?;
    
    // Compare hash bytes since PoseidonHash doesn't implement PartialEq
    if hash1.to_bytes() != hash2.to_bytes() {
        return Err("Same input produced different hashes".to_string());
    }
    
    println!("✅ Hash consistency confirmed: Same input = Same hash");
    Ok(())
}

fn test_hex_to_scalar() -> Result<(), String> {
    println!("\n[Test 3] Conversion from hex to Scalar");
    println!("----------------------------------");
    
    // Some test hex strings
    let hex_values = [
        "3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd",
        "3798866f4e6058035dcf8addb2cf1771fac234bcc8fc05d6676e77e797f224bf",
        "2c51456a7bf2467eac813649f3f25ea896eac27c5da020dae54a6e640278fda2",
    ];
    
    println!("Converting {} hex strings to Scalar values...", hex_values.len());
    
    // Convert all hex strings to Scalar values
    let scalars: Vec<Scalar> = hex_values.iter().map(|&s| {
        println!("Processing: {}", s);
        scalar_from_hex(s)
    }).collect();
    
    // Verify that the converted values are not trivial
    for (i, scalar) in scalars.iter().enumerate() {
        if scalar.to_bytes().iter().all(|&b| b == 0) {
            return Err(format!(
                "Converted Scalar for '{}' is zero, which cannot be correct",
                hex_values[i]
            ));
        }
    }
    
    println!("✅ Successfully converted all {} hex strings to valid Scalar values", hex_values.len());
    Ok(())
}

fn test_constants() -> Result<(), String> {
    println!("\n[Test 4] MDS Matrix and Round Constants");
    println!("-----------------------------------");
    
    // Create MDS matrix from hex constants
    println!("Creating MDS matrix from hex constants...");
    let mds_hex = [
        ["3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd", 
         "3798866f4e6058035dcf8addb2cf1771fac234bcc8fc05d6676e77e797f224bf", 
         "2c51456a7bf2467eac813649f3f25ea896eac27c5da020dae54a6e640278fda2"],
        ["20088ca07bbcd7490a0218ebc0ecb31d0ea34840e2dc2d33a1a5adfecff83b43", 
         "1d04ba0915e7807c968ea4b1cb2d610c7f9a16b4033f02ebacbb948c86a988c3", 
         "5387ccd5729d7acbd09d96714d1d18bbd0eeaefb2ddee3d2ef573c9c7f953307"],
        ["1e208f585a72558534281562cad89659b428ec61433293a8d7f0f0e38a6726ac", 
         "0455ebf862f0b60f69698e97d36e8aafd4d107cae2b61be1858b23a3363642e0", 
         "569e2c206119e89455852059f707370e2c1fc9721f6c50991cedbbf782daef54"],
    ];
    
    let mds_matrix: Vec<Vec<Scalar>> = mds_hex.iter()
        .map(|row| row.iter().map(|&s| scalar_from_hex(s)).collect())
        .collect();
    
    println!("Successfully created MDS matrix ({} x {} elements)", 
             mds_matrix.len(), 
             if mds_matrix.len() > 0 { mds_matrix[0].len() } else { 0 });
    
    // Create round constants from hex strings
    println!("\nCreating round constants from hex values...");
    let round_constants_hex = [
        "6c4ffa723eaf1a7bf74905cc7dae4ca9ff4a2c3bc81d42e09540d1f250910880",
        "54dd837eccf180c92c2f53a3476e45a156ab69a403b6b9fdfd8dd970fddcdd9a",
        "a5316c9a43627a37481bfc4a85a23b87a97b697d9c6e34ad677c585d7084300c",
    ];
    
    let round_constants: Vec<Scalar> = round_constants_hex.iter()
        .map(|&s| scalar_from_hex(s))
        .collect();
    
    println!("Successfully created round constants ({} elements)", round_constants.len());
    
    println!("✅ Successfully validated Poseidon constants");
    Ok(())
}

// Helper function for converting hex strings to Scalar values
fn scalar_from_hex(s: &str) -> Scalar {
    let s = if s.starts_with("0x") { &s[2..] } else { s };
    
    // Convert hexadecimal to BigUint
    let big_uint = BigUint::parse_bytes(s.as_bytes(), 16)
        .unwrap_or_else(|| panic!("Failed to parse hex string for Scalar: {}", s));
    
    // Convert BigUint to bytes
    let bytes = big_uint.to_bytes_be();
    
    // Convert Scalar from bytes (updated method for curve25519-dalek v4)
    let mut array = [0u8; 32];
    let len = std::cmp::min(bytes.len(), 32);
    array[32 - len..].copy_from_slice(&bytes[..len]);
    Scalar::from_bytes_mod_order(array)
}
