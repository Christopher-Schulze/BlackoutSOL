// Test-Skript für Poseidon-Funktionalität
// Dieses Skript kann außerhalb des regulären Anchor-Frameworks ausgeführt werden

use blackout::{poseidon_constants, poseidon_validator};
use hex;

fn main() -> Result<(), String> {
    println!("🧪 Starte Poseidon-Tests...");
    
    // Test 1: Poseidon-Parameter-Validierung
    println!("\n🔍 Test 1: Poseidon-Parameter-Validierung");
    let validation_result = poseidon_validator::validate_for_test()
        .map_err(|e| format!("Validierungsfehler: {}", e))?;
    println!("✅ Validierung erfolgreich: {:?}", validation_result);
    
    // Test 2: Konsistente Hash-Generierung
    println!("\n🔢 Test 2: Konsistente Hash-Generierung");
    // Testvektor 1 - einfache Eingabe
    let input1 = b"BlackoutSOL-Test-Input";
    let hash1 = poseidon_validator::generate_hash_for_test(&[input1])
        .map_err(|e| format!("Hash-Fehler: {}", e))?;
    println!("✅ Erfolgreicher Hash für Eingabe 1: 0x{}", hex::encode(hash1));
    
    // Test 3: Hash-Konstanz
    println!("\n🔄 Test 3: Hash-Konstanz (gleiche Eingabe = gleicher Hash)");
    let hash1_repeat = poseidon_validator::generate_hash_for_test(&[input1])
        .map_err(|e| format!("Hash-Fehler: {}", e))?;
    if hash1 == hash1_repeat {
        println!("✅ Hash-Konstanz bestätigt");
    } else {
        return Err("❌ Hash-Konstanz fehlgeschlagen - gleiche Eingabe erzeugt unterschiedliche Hashes".to_string());
    }
    
    // Test 4: MDS-Matrix aus Konstanten abrufen
    println!("\n📊 Test 4: MDS-Matrix und Konstanten laden");
    let mds_matrix = poseidon_constants::get_mds_matrix();
    println!("✅ MDS-Matrix erfolgreich geladen ({} x {} Elemente)", 
        mds_matrix.len(), 
        if mds_matrix.len() > 0 { mds_matrix[0].len() } else { 0 });
    
    let round_constants = poseidon_constants::get_round_constants();
    println!("✅ Rundenkonstanten erfolgreich geladen ({} Elemente)", round_constants.len());
    
    println!("\n🎉 Alle Poseidon-Tests erfolgreich abgeschlossen!");
    Ok(())
}
