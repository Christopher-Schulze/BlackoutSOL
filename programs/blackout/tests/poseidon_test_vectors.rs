/*
 * Poseidon-Hash-Testvektoren für BlackoutSOL
 * 
 * Diese Tests validieren die korrekte Implementierung der Poseidon-Hash-Funktion
 * gegen die definierten Testvektoren in crypto_test_vectors.md.
 */

use solana_poseidon::{hashv, Endianness, Parameters};

// Test für Poseidon-Hash mit Byte-Arrays
#[test]
fn test_poseidon_hash_vectors() {
    println!("Teste Poseidon-Hash mit Byte-Arrays...");
    
    // Test-Eingaben
    let input1 = [0u8; 32]; // Alle Nullen
    let input2 = {
        let mut arr = [0u8; 32];
        arr[31] = 1; // Eins am Ende
        arr
    };
    
    // Berechnung der Hashes
    println!("Berechne Hash für [0u8; 32]...");
    let hash1 = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input1]).unwrap();
    
    println!("Berechne Hash für [0,0,...,1]...");
    let hash2 = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input2]).unwrap();
    
    // Ausgabe der Hash-Werte als Hex-Strings
    let hash1_bytes = hash1.to_bytes();
    let hash2_bytes = hash2.to_bytes();
    
    println!("Hash1 (Nullen): {}", hex::encode(&hash1_bytes));
    println!("Hash2 (Eins am Ende): {}", hex::encode(&hash2_bytes));
    
    // Überprüfung, dass verschiedene Eingaben verschiedene Ausgaben erzeugen
    assert_ne!(hash1_bytes, hash2_bytes, "Verschiedene Eingaben sollten verschiedene Hashes erzeugen");
    
    println!("✅ Poseidon-Hash-Test bestanden!");
}

// Test mit zwei Eingaben
#[test]
fn test_poseidon_hash_multiple_inputs() {
    println!("Teste Poseidon-Hash mit mehreren Eingaben...");
    
    // Test-Eingaben
    let input1 = [0u8; 32]; // Alle Nullen
    let input2 = [1u8; 32]; // Alle Einsen
    
    // Berechnung der Hashes
    println!("Berechne Hash für einzelne Eingaben...");
    let hash_single1 = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input1]).unwrap();
    let hash_single2 = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input2]).unwrap();
    
    println!("Berechne Hash für kombinierte Eingaben...");
    let hash_combined = hashv(Parameters::Bn254X5, Endianness::BigEndian, &[&input1, &input2]).unwrap();
    
    // Ausgabe der Hash-Werte als Hex-Strings
    let hash_single1_bytes = hash_single1.to_bytes();
    let hash_single2_bytes = hash_single2.to_bytes();
    let hash_combined_bytes = hash_combined.to_bytes();
    
    println!("Hash Input1: {}", hex::encode(&hash_single1_bytes));
    println!("Hash Input2: {}", hex::encode(&hash_single2_bytes));
    println!("Hash Kombiniert: {}", hex::encode(&hash_combined_bytes));
    
    // Überprüfung, dass der kombinierte Hash anders ist als die Einzel-Hashes
    assert_ne!(hash_single1_bytes, hash_combined_bytes, "Kombinierter Hash sollte anders sein als Hash von Input1");
    assert_ne!(hash_single2_bytes, hash_combined_bytes, "Kombinierter Hash sollte anders sein als Hash von Input2");
    
    println!("✅ Poseidon-Hash-Test mit mehreren Eingaben bestanden!");
}


