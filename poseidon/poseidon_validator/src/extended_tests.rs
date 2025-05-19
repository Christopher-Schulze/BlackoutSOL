/// Erweiterte Testfälle für den Poseidon-Validator
///
/// Diese Testfälle erweitern die Standardvalidierung mit zusätzlichen 
/// Edge-Cases und Spezialfällen.

use solana_poseidon::{Parameters, Endianness, hashv};

/// Führt erweiterte Tests des Poseidon-Hashing-Algorithmus durch
pub fn run_extended_validation() -> Result<(), String> {
    println!("\n--- Erweiterte Poseidon-Validierungstests ---");
    
    // 1. Test mit verschiedenen Eingabelängen
    println!("\nTeste verschiedene Eingabelängen...");
    test_different_input_sizes()?;
    
    // 2. Grenzfall-Tests
    println!("\nTeste Grenzfälle und Extremwerte...");
    test_edge_cases()?;
    
    // 3. Kollisionstests
    println!("\nFühre einfache Kollisionstests durch...");
    test_collision_resistance()?;
    
    // 4. Performancemessung
    println!("\nFühre Performancemessung durch...");
    measure_performance()?;
    
    println!("\n✅ Alle erweiterten Tests erfolgreich abgeschlossen!");
    
    Ok(())
}

/// Testet die Verarbeitung verschiedener Eingabelängen
fn test_different_input_sizes() -> Result<(), String> {
    // Test mit 1 Eingabe
    let single_input = {
        let mut arr = [0u8; 32];
        arr[0] = 0x01;
        arr
    };
    
    let hash_one_input = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&single_input]
    ).map_err(|e| format!("Fehler beim Hashing mit 1 Eingabe: {:?}", e))?;
    
    println!("✓ Hash mit 1 Eingabe erfolgreich");
    
    // Test mit 2 Eingaben
    let input_a = {
        let mut arr = [0u8; 32];
        arr[0] = 0x01;
        arr
    };
    
    let input_b = {
        let mut arr = [0u8; 32];
        arr[0] = 0x02;
        arr
    };
    
    let hash_two_inputs = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&input_a, &input_b]
    ).map_err(|e| format!("Fehler beim Hashing mit 2 Eingaben: {:?}", e))?;
    
    println!("✓ Hash mit 2 Eingaben erfolgreich");
    
    // Vergleich der beiden Hashes (sollten unterschiedlich sein)
    if hash_one_input.to_bytes() == hash_two_inputs.to_bytes() {
        return Err("Kritischer Fehler: Hash ändert sich nicht bei unterschiedlicher Eingabeanzahl!".to_string());
    }
    
    println!("✓ Hashes für unterschiedliche Eingabeanzahlen sind korrekt unterschiedlich");
    
    Ok(())
}

/// Testet Grenzfälle und Extremwerte
fn test_edge_cases() -> Result<(), String> {
    // Test mit minimal möglichem Wert (0)
    let min_input = [0u8; 32];
    
    let hash_min = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&min_input]
    ).map_err(|e| format!("Fehler beim Hashing mit Minimalwert: {:?}", e))?;
    
    println!("✓ Hash mit Minimalwert (0) erfolgreich: {}", hex::encode(&hash_min.to_bytes()[..]));
    
    // Test mit einem kleinstmöglichen Nicht-Null-Wert
    let mut almost_min_input = [0u8; 32];
    almost_min_input[31] = 1; // LSB auf 1 setzen
    
    let hash_almost_min = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&almost_min_input]
    ).map_err(|e| format!("Fehler beim Hashing mit Fast-Minimalwert: {:?}", e))?;
    
    println!("✓ Hash mit Fast-Minimalwert erfolgreich");
    
    // Vergleich der Hashes (sollten unterschiedlich sein)
    if hash_min.to_bytes() == hash_almost_min.to_bytes() {
        return Err("Kritischer Fehler: Hash unterscheidet nicht zwischen 0 und Fast-Null!".to_string());
    }
    
    // Test mit unter dem Modulus liegenden größeren Werten (aber immer noch klein genug)
    let mut large_input = [0u8; 32];
    large_input[31] = 0x7F; // Ein größerer Wert, aber am least significant byte
    
    let hash_large = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&large_input]
    ).map_err(|e| format!("Fehler beim Hashing mit größerem Wert: {:?}", e))?;
    
    println!("✓ Hash mit größerem Wert erfolgreich: {}", hex::encode(&hash_large.to_bytes()[..]));
    
    // Test mit sicheren Werten (Kenntnis der BN254-Modulus-Einschränkungen)
    println!("Hinweis: BN254 hat einen 254-Bit Modulus, daher müssen Werte entsprechend begrenzt sein.");
    
    Ok(())
}

/// Führt einen einfachen Test auf Kollisionsresistenz durch
fn test_collision_resistance() -> Result<(), String> {
    use std::collections::HashSet;
    
    // Generiere viele Hashes mit leicht unterschiedlichen Eingaben
    let mut hashes = HashSet::new();
    let mut collisions = 0;
    
    for i in 0..20 {
        let mut input = [0u8; 32];
        input[0] = i;
        
        let hash_result = hashv(
            Parameters::Bn254X5,
            Endianness::BigEndian,
            &[&input]
        ).map_err(|e| format!("Fehler beim Hashing für Kollisionstest: {:?}", e))?;
        
        let hash_bytes = hash_result.to_bytes();
        
        if !hashes.insert(hash_bytes) {
            collisions += 1;
            println!("⚠️ Kollision entdeckt für Wert {}!", i);
        }
    }
    
    if collisions > 0 {
        return Err(format!("Kollisionstest fehlgeschlagen: {} Kollisionen in 20 Hashes", collisions));
    }
    
    println!("✓ Keine Kollisionen in 20 Hashes mit unterschiedlichen Eingaben");
    
    Ok(())
}

/// Misst die Performance des Hashing-Algorithmus
fn measure_performance() -> Result<(), String> {
    use std::time::{Instant, Duration};
    
    let iterations = 100;
    let mut total_time = Duration::new(0, 0);
    
    for i in 0..iterations {
        // BN254-kompatible Eingaben verwenden (nur kleine Werte in den least significant Bytes)
        let mut input = [0u8; 32];
        input[31] = (i % 100) as u8;  // Verwende nur Werte von 0-99 und platziere sie am least significant byte
        
        let start = Instant::now();
        let _hash_result = hashv(
            Parameters::Bn254X5, 
            Endianness::BigEndian,
            &[&input]
        ).map_err(|e| format!("Fehler beim Performance-Test: {:?}", e))?;
        let elapsed = start.elapsed();
        
        total_time += elapsed;
    }
    
    let avg_time_micros = total_time.as_micros() / iterations as u128;
    println!("✓ Performance: Average {} µs per hash (over {} iterations)", 
             avg_time_micros, iterations);
    
    Ok(())
}
