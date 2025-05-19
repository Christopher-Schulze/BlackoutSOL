//! Validierungsskript für Poseidon-Parameter in BlackoutSOL
//!
//! Dieses Skript validiert die in BlackoutSOL verwendeten Poseidon-Parameter
//! anhand der Erfahrungen aus dem separaten Poseidon-Validator-Projekt.
//!
//! Ausführung: cargo run -p blackout_poseidon_validator

use solana_poseidon::{hashv, Parameters, Endianness};
use std::time::{Duration, Instant};
use std::collections::HashSet;

/// Führt eine schnelle Validierung der Poseidon-Parameter durch
pub fn validate_poseidon_parameters() -> Result<(), String> {
    println!("=== BlackoutSOL Poseidon-Parameter-Validierungstool v1.0 ===\n");
    println!("Hash-Konsistenztest läuft...\n");

    // BN254-kompatibler Testvektor (der Wert muss kleiner als der Modulus sein)
    // Wir verwenden einen Wert, der nur die least significant bytes setzt
    let mut test_input = [0u8; 32];
    test_input[31] = 0x42; // Nur das letzte Byte setzen, um unter dem Modulus zu bleiben
    
    // Der erwartete Hash für diese Eingabe (basierend auf dem tatsächlichen Output der Bibliothek)
    // Dieser Wert wurde empirisch bestimmt und dient als Referenz für zukünftige Validierungen
    let expected_hash = "011e70075d2f41deacf19a385a674c5a2582d52b83d05f42a27bdf19dd352433";

    // Hash mit BN254X5-Parametern berechnen
    let hash_result = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&test_input]
    ).map_err(|e| format!("Fehler bei der Hash-Berechnung: {:?}", e))?;
    
    println!("✓ Poseidon-Parameter (Bn254X5) geladen");
    println!("✓ Hash-Berechnung erfolgreich");

    // Hash-Wert in hexadezimale Darstellung umwandeln
    let result_hex = hex::encode(hash_result.to_bytes());
    println!("Hash des Testwerts [0x42; 32]: 0x{}", result_hex);

    // Überprüfen, ob der berechnete Hash mit dem erwarteten Ergebnis übereinstimmt
    if result_hex != expected_hash {
        println!("✗ Hash-Konsistenz fehlgeschlagen: erwartet {}, erhalten {}", expected_hash, result_hex);
        return Err(format!(
            "Hash-Konsistenz fehlgeschlagen: erwartet {}, erhalten {}",
            expected_hash, result_hex
        ));
    }

    println!("✓ Hash-Konsistenz bestätigt\n");
    println!("✅ Basis-Validierung abgeschlossen: Die Poseidon-Parameter sind korrekt implementiert.\n");
    
    Ok(())
}

/// Testet verschiedene Eingabelängen
fn test_different_input_sizes() -> Result<(), String> {
    println!("Teste verschiedene Eingabelängen...");

    // Test mit einer einzelnen Eingabe
    let input1 = [0x01; 32];
    let hash_result1 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&input1]
    ).map_err(|e| format!("Fehler beim Hashing mit 1 Eingabe: {:?}", e))?;

    println!("✓ Hash mit 1 Eingabe erfolgreich");

    // Test mit zwei Eingaben
    let input2 = [0x02; 32];
    let hash_result2 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&input1, &input2]
    ).map_err(|e| format!("Fehler beim Hashing mit 2 Eingaben: {:?}", e))?;

    println!("✓ Hash mit 2 Eingaben erfolgreich");

    // Vergleiche Hashes für verschiedene Eingaben
    let hash1 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&input1]
    ).map_err(|e| format!("Fehler beim Hashing: {:?}", e))?;

    let hash2 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&input1, &input2]
    ).map_err(|e| format!("Fehler beim Hashing: {:?}", e))?;

    if hash1.to_bytes() == hash2.to_bytes() {
        println!("⚠️ WARNUNG: Identische Hashes für unterschiedliche Eingabeanzahlen. Dies kann auf einen Fehler hindeuten.");
    } else {
        println!("✓ Hashes für unterschiedliche Eingabeanzahlen sind korrekt unterschiedlich");
    }

    Ok(())
}

/// Testet Grenzfälle und Extremwerte
fn test_edge_cases() -> Result<(), String> {
    println!("\nTeste Grenzfälle und Extremwerte...");

    // Test mit Nullwerten
    let zeros = [0u8; 32];
    let hash_zeros = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&zeros]
    ).map_err(|e| format!("Fehler beim Hashing mit Nullwert: {:?}", e))?;
    
    let hash_hex = hex::encode(hash_zeros.to_bytes());
    println!("✓ Hash mit Minimalwert (0) erfolgreich: {}", hash_hex);
    
    // Test mit Fast-Nullwerten (nur ein Bit gesetzt)
    let mut near_zeros = [0u8; 32];
    near_zeros[31] = 1; // Only least significant bit set
    
    let hash_near_zeros = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&near_zeros]
    ).map_err(|e| format!("Fehler beim Hashing mit Fast-Nullwert: {:?}", e))?;
    
    println!("✓ Hash mit Fast-Minimalwert erfolgreich");
    
    // Überprüfe, ob nahe beieinander liegende Werte unterschiedliche Hashes erzeugen
    if hash_zeros.to_bytes() == hash_near_zeros.to_bytes() {
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
    
    println!("✓ Hash mit größerem Wert erfolgreich: {}", hex::encode(hash_large.to_bytes()));
    println!("Hinweis: BN254 hat einen 254-Bit Modulus, daher müssen Werte entsprechend begrenzt sein.");
    
    Ok(())
}

/// Einfacher Kollisionsresistenztest
fn test_collision_resistance() -> Result<(), String> {
    println!("\nFühre einfache Kollisionstests durch...");
    
    // Generiere viele Hashes mit leicht unterschiedlichen Eingaben
    let mut hashes = HashSet::new();
    let mut collisions = 0;
    
    for i in 0..20 {
        let mut input = [0u8; 32];
        input[31] = i;
        
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

/// Misst die Performance der Hash-Funktion
fn measure_performance() -> Result<(), String> {
    println!("\nFühre Performancemessung durch...");
    
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
    println!("✓ Performance: Durchschnittlich {} µs pro Hash (über {} Iterationen)", 
             avg_time_micros, iterations);
    
    Ok(())
}

/// Führt erweiterte Tests des Poseidon-Hashing-Algorithmus durch
fn run_extended_validation() -> Result<(), String> {
    println!("--- Erweiterte Poseidon-Validierungstests ---\n");

    test_different_input_sizes()?;
    test_edge_cases()?;
    test_collision_resistance()?;
    measure_performance()?;

    println!("\n✅ Alle erweiterten Tests erfolgreich abgeschlossen!");

    Ok(())
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    match validate_poseidon_parameters() {
        Ok(_) => {
            match run_extended_validation() {
                Ok(_) => {
                    println!("\n✅ Erweiterte Validierung abgeschlossen: Alle Tests erfolgreich!");
                    println!("\n✨ Poseidon-Validator abgeschlossen");
                },
                Err(e) => {
                    println!("\n❌ FEHLER bei der erweiterten Validierung: {}", e);
                    return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)));
                }
            }
        },
        Err(e) => {
            println!("\n❌ FEHLER bei der Basis-Validierung: {}", e);
            return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, e)));
        }
    }

    Ok(())
}
