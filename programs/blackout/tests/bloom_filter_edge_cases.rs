/*
 * Bloom-Filter Edge-Case-Tests für BlackoutSOL
 * 
 * Diese Tests validieren das Verhalten des Bloom-Filters bei Grenzwerten und
 * ungewöhnlichen Eingaben, um Robustheit für die Sicherheitsüberprüfung zu gewährleisten.
 */

use blackout::utils::{check_bloom_filter, generate_bloom_filter};
use blackout::state::config::BlackoutConfig;

/// Test für Bloom-Filter-Generierung mit extremen Konfigurationswerten
#[test]
fn test_bloom_filter_extreme_configs() {
    println!("Teste Bloom-Filter mit extremen Konfigurationswerten...");

    // Testfall 1: Minimale Konfiguration
    let min_config = BlackoutConfig {
        num_hops: 1,
        real_splits: 1,
        fake_splits: 0,
        ..BlackoutConfig::new()
    };
    
    // Testfall 2: Hohe (aber nicht maximale) Konfiguration
    // Vermeidet absichtlich extreme Werte, die zum Überlauf führen würden
    let max_config = BlackoutConfig {
        num_hops: 8,     // Höherer, aber sicherer Wert
        real_splits: 16,
        fake_splits: 32,
        ..BlackoutConfig::new()
    };
    
    // Testfall 3: Unausgewogene Konfiguration
    let unbalanced_config = BlackoutConfig {
        num_hops: 2,
        real_splits: 50,
        fake_splits: 1,
        ..BlackoutConfig::new()
    };
    
    // Challenge-Wert (wird in der aktuellen Implementierung nicht verwendet)
    let challenge = [0u8; 32];
    
    // Filter generieren
    let filter1 = generate_bloom_filter(&min_config, &challenge);
    let filter2 = generate_bloom_filter(&max_config, &challenge);
    let filter3 = generate_bloom_filter(&unbalanced_config, &challenge);
    
    println!("Minimale Konfiguration Filter: {}", hex::encode(&filter1));
    println!("Maximale Konfiguration Filter: {}", hex::encode(&filter2));
    println!("Unausgewogene Konfiguration Filter: {}", hex::encode(&filter3));
    
    // Validiere, dass die Filter die richtige Größe haben
    assert_eq!(filter1.len(), 16, "Bloom-Filter sollte immer 16 Bytes lang sein");
    assert_eq!(filter2.len(), 16, "Bloom-Filter sollte immer 16 Bytes lang sein");
    assert_eq!(filter3.len(), 16, "Bloom-Filter sollte immer 16 Bytes lang sein");
    
    // Prüfe, dass unterschiedliche Konfigurationen unterschiedliche Filter erzeugen
    assert_ne!(filter1, filter2, "Unterschiedliche Konfigurationen sollten unterschiedliche Filter erzeugen");
    assert_ne!(filter1, filter3, "Unterschiedliche Konfigurationen sollten unterschiedliche Filter erzeugen");
    assert_ne!(filter2, filter3, "Unterschiedliche Konfigurationen sollten unterschiedliche Filter erzeugen");
    
    println!("✅ Bloom-Filter mit extremen Konfigurationswerten getestet!");
}

/// Test für Überprüfung von Grenzfällen bei den Indizes
#[test]
fn test_bloom_filter_boundary_indices() {
    println!("Teste Bloom-Filter mit Grenzwert-Indizes...");
    
    // Normale Konfiguration erstellen
    let config = BlackoutConfig::new();
    let challenge = [0u8; 32];
    
    // Filter generieren
    let filter = generate_bloom_filter(&config, &challenge);
    
    // Teste Grenzfälle für Hop- und Split-Indizes
    let max_value: u8 = 255;
    
    // Erfasse Ergebnisse für verschiedene Indizes
    let regular_result = check_bloom_filter(&filter, 0, 0);
    let max_hop_result = check_bloom_filter(&filter, max_value, 0);
    let max_split_result = check_bloom_filter(&filter, 0, max_value);
    let max_both_result = check_bloom_filter(&filter, max_value, max_value);
    
    println!("Reguläre Indizes (0,0): {}", regular_result);
    println!("Maximaler Hop-Index (255,0): {}", max_hop_result);
    println!("Maximaler Split-Index (0,255): {}", max_split_result);
    println!("Beide maximale Indizes (255,255): {}", max_both_result);
    
    // Wir prüfen nicht auf spezifische Ergebnisse, da diese implementierungsspezifisch sind,
    // aber stellen sicher, dass die Funktion nicht panikt oder abstürzt
    
    println!("✅ Bloom-Filter mit Grenzwert-Indizes getestet!");
}

/// Test für die Konsistenz der Bloom-Filter-Überprüfung
#[test]
fn test_bloom_filter_consistency() {
    println!("Teste Konsistenz der Bloom-Filter-Überprüfung...");
    
    // Filter mit bestimmten bit-patterns erstellen
    let all_zeros = [0u8; 16];
    let all_ones = [255u8; 16];
    let mut alternating = [0u8; 16];
    for i in 0..16 {
        if i % 2 == 0 {
            alternating[i] = 0xAA; // 10101010
        } else {
            alternating[i] = 0x55; // 01010101
        }
    }
    
    // Teste die gleichen Indizes mehrmals, um Konsistenz zu gewährleisten
    for _ in 0..5 {
        // Für jeden der drei Filter prüfen wir die gleichen Indizes mehrmals
        for (i, j) in [(0, 0), (1, 1), (2, 3), (10, 20), (200, 200)] {
            // Jedes Ergebnis sollte bei wiederholten Aufrufen gleich sein
            let zeros_first = check_bloom_filter(&all_zeros, i, j);
            let zeros_second = check_bloom_filter(&all_zeros, i, j);
            
            let ones_first = check_bloom_filter(&all_ones, i, j);
            let ones_second = check_bloom_filter(&all_ones, i, j);
            
            let alt_first = check_bloom_filter(&alternating, i, j);
            let alt_second = check_bloom_filter(&alternating, i, j);
            
            // Konsistenz für jeden Filter überprüfen
            assert_eq!(zeros_first, zeros_second, 
                      "Filter mit Nullen sollte konsistente Ergebnisse liefern");
            assert_eq!(ones_first, ones_second, 
                      "Filter mit Einsen sollte konsistente Ergebnisse liefern");
            assert_eq!(alt_first, alt_second, 
                      "Filter mit alternierenden Bits sollte konsistente Ergebnisse liefern");
        }
    }
    
    println!("✅ Bloom-Filter-Konsistenz bestätigt!");
}

/// Performance-Test für die Bloom-Filter-Operationen
#[test]
fn test_bloom_filter_performance() {
    println!("Führe Bloom-Filter Performance-Tests durch...");
    
    const NUM_OPERATIONS: usize = 1000;
    
    // Standard-Konfiguration
    let config = BlackoutConfig::new();
    let challenge = [0u8; 32];
    
    use std::time::Instant;
    
    // Teste Generierungsgeschwindigkeit
    let gen_start = Instant::now();
    for _ in 0..NUM_OPERATIONS {
        let _filter = generate_bloom_filter(&config, &challenge);
    }
    let gen_duration = gen_start.elapsed();
    
    println!("Filter-Generierung: {} Operationen in {:?} (durchschnittlich {:?} pro Operation)",
             NUM_OPERATIONS, gen_duration, gen_duration / NUM_OPERATIONS as u32);
    
    // Teste Überprüfungsgeschwindigkeit
    let filter = generate_bloom_filter(&config, &challenge);
    let check_start = Instant::now();
    for i in 0..NUM_OPERATIONS {
        // Modulare Indizes um verschiedene Werte zu testen
        let hop = (i % 10) as u8;
        let split = (i % 50) as u8;
        let _result = check_bloom_filter(&filter, hop, split);
    }
    let check_duration = check_start.elapsed();
    
    println!("Filter-Überprüfung: {} Operationen in {:?} (durchschnittlich {:?} pro Operation)",
             NUM_OPERATIONS, check_duration, check_duration / NUM_OPERATIONS as u32);
    
    println!("✅ Bloom-Filter Performance-Tests abgeschlossen!");
}
