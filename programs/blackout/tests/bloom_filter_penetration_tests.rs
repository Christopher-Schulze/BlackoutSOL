/*
 * Bloom-Filter-Penetrationstests für BlackoutSOL
 * 
 * Diese Tests überprüfen die Robustheit des Bloom-Filter-Mechanismus gegen
 * potenzielle Angriffe und Manipulationsversuche.
 */

use blackout::utils::{check_bloom_filter, generate_bloom_filter};
use blackout::state::config::BlackoutConfig;
use std::collections::HashSet;

/// Test für Manipulationsversuche mit extremen Konfigurationswerten
#[test]
fn test_bloom_filter_overflow_resistance() {
    println!("Teste Overflow-Resistenz des Bloom-Filters...");

    // Versuche mit verschiedenen extremen Konfigurationswerten
    let configs = [
        // Extrem hohe Werte, um Überläufe zu provozieren
        BlackoutConfig {
            num_hops: 255,
            real_splits: 255,
            fake_splits: 255,
            ..BlackoutConfig::new()
        },
        // Unausgewogene Konfiguration
        BlackoutConfig {
            num_hops: 100,
            real_splits: 0,
            fake_splits: 255,
            ..BlackoutConfig::new()
        },
        // Grenzfall: keine Fake-Splits
        BlackoutConfig {
            num_hops: 10,
            real_splits: 50,
            fake_splits: 0,
            ..BlackoutConfig::new()
        }
    ];

    let challenge = [0u8; 32];
    
    // Für jede Konfiguration
    for (i, config) in configs.iter().enumerate() {
        println!("Teste Konfiguration {}: num_hops={}, real_splits={}, fake_splits={}", 
                i, config.num_hops, config.real_splits, config.fake_splits);
        
        // Diese Funktionsaufrufe sollten nicht abstürzen oder überfluten
        let filter = generate_bloom_filter(config, &challenge);
        
        // Überprüfe, ob der Filter die richtige Länge hat
        assert_eq!(filter.len(), 16, "Filter sollte immer 16 Bytes lang sein");
        
        // Versuche einige Überprüfungen, um sicherzustellen, dass der Filter 
        // konsistent funktioniert
        let _result1 = check_bloom_filter(&filter, 0, 0);
        let _result2 = check_bloom_filter(&filter, 254, 254);
    }
    
    println!("✅ Bloom-Filter ist resistent gegen Overflow-Angriffe!");
}

/// Test für Kollisionsangriffe auf den Bloom-Filter
#[test]
fn test_bloom_filter_collision_resistance() {
    println!("Teste Kollisionsresistenz des Bloom-Filters...");
    
    // Erstelle eine Standard-Konfiguration
    let config = BlackoutConfig {
        num_hops: 4,
        real_splits: 4,
        fake_splits: 12,
        ..BlackoutConfig::new()
    };
    
    let challenge = [0u8; 32];
    let filter = generate_bloom_filter(&config, &challenge);
    
    // Menge zur Erkennung von Kollisionen
    let mut positions_set = HashSet::new();
    
    // Speichere die Indizes, die als fake markiert wurden
    let mut fake_indices = Vec::new();
    
    // Finde heraus, welche Indizes als fake markiert wurden
    for hop in 0..config.num_hops {
        for split in 0..(config.real_splits + config.fake_splits) {
            if check_bloom_filter(&filter, hop, split) {
                fake_indices.push((hop, split));
                
                // Berechne Position im Filter
                let position = ((hop as u32) << 8 | split as u32) % 128;
                positions_set.insert(position);
                
                println!("Fake-Split gefunden: hop={}, split={}, position={}", 
                        hop, split, position);
            }
        }
    }
    
    // Prüfe, ob die Anzahl der als fake markierten Splits mit der 
    // Konfiguration übereinstimmt (real_splits..real_splits+fake_splits)
    let expected_fake_count = (config.num_hops as usize) * (config.fake_splits as usize);
    assert_eq!(fake_indices.len(), expected_fake_count, 
              "Anzahl der gefundenen Fake-Splits sollte der Konfiguration entsprechen");
    
    // Die Anzahl der genutzten Positionen kann aufgrund von Kollisionen 
    // kleiner sein als die Anzahl der Fake-Splits
    println!("Anzahl der gefundenen Fake-Splits: {}", fake_indices.len());
    println!("Anzahl der genutzten Positionen im Filter: {}", positions_set.len());
    println!("Kollisionsrate: {:.2}%", 
            (1.0 - (positions_set.len() as f64 / fake_indices.len() as f64)) * 100.0);
    
    println!("✅ Bloom-Filter-Kollisionseigenschaften analysiert!");
}

/// Test für Timing-Angriffe auf den Bloom-Filter
#[test]
fn test_bloom_filter_timing_resistance() {
    println!("Teste Resistenz gegen Timing-Angriffe...");
    
    use std::time::{Instant, Duration};
    
    // Erstelle einen Filter mit bekannten Eigenschaften
    let config = BlackoutConfig {
        num_hops: 2,
        real_splits: 2,
        fake_splits: 2,
        ..BlackoutConfig::new()
    };
    
    let challenge = [0u8; 32];
    let filter = generate_bloom_filter(&config, &challenge);
    
    // Liste von zu prüfenden Kombinationen
    let test_cases = [
        // Bekannte Einträge (sollten gefunden werden)
        (0, 2), (0, 3), (1, 2), (1, 3),
        // Unbekannte Einträge (sollten nicht gefunden werden)
        (0, 0), (0, 1), (1, 0), (1, 1),
        // Extreme Werte
        (255, 255), (0, 255), (255, 0)
    ];
    
    const ITERATIONS: usize = 1000;
    let mut timings = vec![Duration::new(0, 0); test_cases.len()];
    
    // Führe jede Prüfung mehrmals durch und messe die Zeit
    for _ in 0..ITERATIONS {
        for (i, &(hop, split)) in test_cases.iter().enumerate() {
            let start = Instant::now();
            let _result = check_bloom_filter(&filter, hop, split);
            timings[i] += start.elapsed();
        }
    }
    
    // Berechne durchschnittliche Zeiten
    let avg_timings: Vec<f64> = timings.iter()
        .map(|t| t.as_nanos() as f64 / ITERATIONS as f64)
        .collect();
    
    // Berechne Standard-Abweichung
    let mean = avg_timings.iter().sum::<f64>() / avg_timings.len() as f64;
    let variance = avg_timings.iter()
        .map(|t| {
            let diff = mean - *t;
            diff * diff
        })
        .sum::<f64>() / avg_timings.len() as f64;
    let std_dev = variance.sqrt();
    
    // Prüfe, ob die Timing-Unterschiede statistisch signifikant sind
    // Wir erwarten, dass alle Prüfungen ungefähr gleich lange dauern
    println!("Durchschnittliche Zeit pro Prüfung: {:.2} ns", mean);
    println!("Standardabweichung: {:.2} ns", std_dev);
    
    // Eine grobe Schätzung: Wenn die Standardabweichung weniger als 50% des Mittelwertes 
    // beträgt, betrachten wir die Timing-Unterschiede als nicht signifikant
    // Dies ist eine vereinfachte Heuristik für den Test
    if std_dev < (mean * 0.5) {
        println!("✅ Bloom-Filter ist resistent gegen einfache Timing-Angriffe!");
    } else {
        println!("⚠️ Bloom-Filter zeigt signifikante Timing-Unterschiede!");
        
        // Drucke detaillierte Informationen zu den Timing-Unterschieden
        for (i, &(hop, split)) in test_cases.iter().enumerate() {
            println!("  Hop {}, Split {}: {:.2} ns", hop, split, avg_timings[i]);
        }
    }
}

/// Test für Manipulationen am Bloom-Filter-Speicher
#[test]
fn test_bloom_filter_tampering_resistance() {
    println!("Teste Resistenz gegen Manipulationen am Filter...");
    
    // Erstelle einen Filter mit bekannten Eigenschaften
    let config = BlackoutConfig {
        num_hops: 2,
        real_splits: 2,
        fake_splits: 2,
        ..BlackoutConfig::new()
    };
    
    let challenge = [0u8; 32];
    let filter = generate_bloom_filter(&config, &challenge);
    
    // Speichere originale Ergebnisse
    let mut original_results = Vec::new();
    for hop in 0..4 {
        for split in 0..4 {
            original_results.push(check_bloom_filter(&filter, hop, split));
        }
    }
    
    // Manipuliere den Filter auf verschiedene Weise
    let manipulations = [
        // Alle Bits auf 0 setzen
        [0u8; 16],
        // Alle Bits auf 1 setzen
        [255u8; 16],
        // Einzelne Bits umkehren
        {
            let mut f = filter;
            // Invertiere das erste Byte
            f[0] = !f[0];
            f
        },
        // Bytes vertauschen
        {
            let mut f = filter;
            f.swap(0, 15);
            f.swap(1, 14);
            f
        }
    ];
    
    // Teste jede Manipulation
    for (i, manipulated) in manipulations.iter().enumerate() {
        println!("Teste Manipulation {}", i);
        
        let mut manipulated_results = Vec::new();
        for hop in 0..4 {
            for split in 0..4 {
                manipulated_results.push(check_bloom_filter(manipulated, hop, split));
            }
        }
        
        // Zähle, wie viele Ergebnisse sich geändert haben
        let changes = original_results.iter().zip(manipulated_results.iter())
            .filter(|&(o, m)| o != m)
            .count();
        
        println!("Manipulation {} änderte {} von {} Ergebnissen", 
                i, changes, original_results.len());
        
        // Wir erwarten, dass Änderungen am Filter die Ergebnisse verändern
        assert!(changes > 0, "Manipulation sollte Ergebnisse ändern");
    }
    
    println!("✅ Bloom-Filter-Ergebnisse ändern sich erwartungsgemäß bei Manipulationen!");
}
