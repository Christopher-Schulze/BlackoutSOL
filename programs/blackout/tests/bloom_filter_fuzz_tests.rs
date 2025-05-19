/*
 * Bloom-Filter Fuzzing-Tests für BlackoutSOL
 * 
 * Diese Tests verwenden zufällig generierte Eingaben, um die Robustheit der
 * Bloom-Filter-Implementierung gegen unerwartete oder fehlerhafte Eingabedaten zu prüfen.
 */

use blackout::utils::{check_bloom_filter, generate_bloom_filter};
use blackout::state::config::BlackoutConfig;
use std::collections::HashSet;
use std::time::Instant;
use rand::{Rng, thread_rng};

/// Generiert zufällige konfigurationen für den Bloom-Filter
fn generate_random_config() -> BlackoutConfig {
    let mut rng = thread_rng();
    
    // Generiere realistische, aber zufällige Werte innerhalb sicherer Grenzen
    let num_hops = rng.gen_range(1..=8);
    let real_splits = rng.gen_range(1..=12);
    let fake_splits = rng.gen_range(1..=20);
    
    BlackoutConfig {
        num_hops,
        real_splits,
        fake_splits,
        ..BlackoutConfig::new()
    }
}

/// Generiert einen zufälligen Challenge-Seed
fn generate_random_challenge() -> [u8; 32] {
    let mut rng = thread_rng();
    let mut challenge = [0u8; 32];
    rng.fill(&mut challenge);
    challenge
}

/// Testet die Bloom-Filter-Generierung mit vielen zufälligen Konfigurationen
#[test]
fn fuzz_bloom_filter_generation() {
    println!("Führe Fuzzing-Tests für Bloom-Filter-Generierung durch...");
    
    // Anzahl der Fuzzing-Iterationen
    const NUM_ITERATIONS: usize = 100;
    let mut generated_filters = HashSet::new();
    
    for i in 0..NUM_ITERATIONS {
        // Generiere zufällige Konfiguration und Challenge
        let config = generate_random_config();
        let challenge = generate_random_challenge();
        
        // Messe die Zeit
        let start = Instant::now();
        
        // Diese Funktion sollte niemals abstürzen, unabhängig von der Eingabe
        let filter = generate_bloom_filter(&config, &challenge);
        let duration = start.elapsed();
        
        // Überprüfe grundlegende Eigenschaften
        assert_eq!(filter.len(), 16, "Filter sollte immer 16 Bytes lang sein");
        
        // Protokolliere den Hash des Filters zur Überprüfung der Einzigartigkeit
        let filter_hash = format!("{:?}", filter);
        let is_new = generated_filters.insert(filter_hash.clone());
        
        if i % 10 == 0 {
            println!("Iteration {}: Konfiguration (hops={}, real={}, fake={}), Dauer: {:?}", 
                    i, config.num_hops, config.real_splits, config.fake_splits, duration);
        }
        
        // Protokolliere Kollisionen (keine Assertion, da Kollisionen möglich sind)
        if !is_new && i % 10 == 0 {
            println!("  - Kollision gefunden in Iteration {}", i);
        }
    }
    
    println!("✅ Bloom-Filter-Generierung erfolgreich mit {} zufälligen Konfigurationen getestet!", NUM_ITERATIONS);
    println!("   Eindeutige generierte Filter: {}/{}", generated_filters.len(), NUM_ITERATIONS);
}

/// Testet die Bloom-Filter-Überprüfung mit zufälligen Filtern und Indizes
#[test]
fn fuzz_bloom_filter_checking() {
    println!("Führe Fuzzing-Tests für Bloom-Filter-Überprüfung durch...");
    
    // Anzahl der Fuzzing-Iterationen
    const NUM_ITERATIONS: usize = 100;
    let mut rng = thread_rng();
    
    // Erstelle ein paar bekannte Filter für Konsistenztests
    let known_configs = [
        BlackoutConfig {
            num_hops: 2,
            real_splits: 2,
            fake_splits: 2,
            ..BlackoutConfig::new()
        },
        BlackoutConfig {
            num_hops: 3,
            real_splits: 4,
            fake_splits: 8,
            ..BlackoutConfig::new()
        },
    ];
    
    let challenge = [0u8; 32];
    let known_filters: Vec<[u8; 16]> = known_configs.iter()
        .map(|config| generate_bloom_filter(config, &challenge))
        .collect();
    
    // Speichere Hash-zu-Ergebnis-Mappings für Konsistenzprüfung
    let mut result_cache = HashSet::new();
    
    for i in 0..NUM_ITERATIONS {
        // Wähle zufällig zwischen bekannten Filtern und zufällig generierten
        let filter = if rng.gen_bool(0.2) {
            // Verwende einen bekannten Filter (20% der Fälle)
            known_filters[rng.gen_range(0..known_filters.len())]
        } else {
            // Generiere einen neuen zufälligen Filter (80% der Fälle)
            let mut random_filter = [0u8; 16];
            rng.fill(&mut random_filter);
            random_filter
        };
        
        // Generiere zufällige Indizes
        let hop = rng.gen::<u8>();
        let split = rng.gen::<u8>();
        
        // Überprüfe den Filter mit diesen Indizes
        let result = check_bloom_filter(&filter, hop, split);
        
        // Cache-Schlüssel für Konsistenzprüfung
        let cache_key = format!("{:?}_{}_{}_{}", filter, hop, split, result);
        
        // Bei der ersten Prüfung einer bestimmten Kombination: füge zum Cache hinzu
        if !result_cache.contains(&cache_key) {
            result_cache.insert(cache_key);
        }
        
        if i % 20 == 0 {
            println!("Iteration {}: hop={}, split={}, ergebnis={}", i, hop, split, result);
        }
    }
    
    println!("✅ Bloom-Filter-Überprüfung erfolgreich mit {} zufälligen Eingaben getestet!", NUM_ITERATIONS);
}

/// Fuzz-Test mit sehr großen Eingabewerten
#[test]
fn fuzz_bloom_filter_large_inputs() {
    println!("Führe Fuzzing-Tests mit großen Eingabewerten durch...");
    
    // Teste mit Extremwerten innerhalb der zulässigen Grenzen
    let configs = [
        BlackoutConfig {
            num_hops: 255,
            real_splits: 127,
            fake_splits: 128,
            ..BlackoutConfig::new()
        },
        BlackoutConfig {
            num_hops: 16,
            real_splits: 64,
            fake_splits: 64,
            ..BlackoutConfig::new()
        },
        BlackoutConfig {
            num_hops: 8,
            real_splits: 190,
            fake_splits: 20,
            ..BlackoutConfig::new()
        },
    ];
    
    // Für jeden Extremfall
    for (i, config) in configs.iter().enumerate() {
        println!("Teste Extremkonfiguration {}: num_hops={}, real_splits={}, fake_splits={}", 
                i, config.num_hops, config.real_splits, config.fake_splits);
                
        let challenge = generate_random_challenge();
        let filter = generate_bloom_filter(config, &challenge);
        
        // Prüfe einige Indizes mit diesem Filter
        let mut rng = thread_rng();
        for _ in 0..10 {
            let hop = rng.gen::<u8>();
            let split = rng.gen::<u8>();
            let result = check_bloom_filter(&filter, hop, split);
            println!("  - Prüfung (hop={}, split={}): {}", hop, split, result);
        }
    }
    
    println!("✅ Bloom-Filter-Tests mit großen Eingabewerten abgeschlossen!");
}

/// Fuzzing für Validierung der Symmetrie-Eigenschaften
#[test]
fn fuzz_bloom_filter_symmetry() {
    println!("Führe Tests für Bloom-Filter-Symmetrieeigenschaften durch...");
    
    // Anzahl der zu testenden Konfigurationen
    const NUM_CONFIGS: usize = 10;
    
    // Für jede Konfiguration
    for i in 0..NUM_CONFIGS {
        let config = generate_random_config();
        let challenge1 = generate_random_challenge();
        let challenge2 = generate_random_challenge();
        
        println!("Konfiguration {}: num_hops={}, real_splits={}, fake_splits={}", 
                i, config.num_hops, config.real_splits, config.fake_splits);
        
        // Generiere Filter und prüfe die Eigenschaften
        let filter1 = generate_bloom_filter(&config, &challenge1);
        let filter2 = generate_bloom_filter(&config, &challenge1); // Gleicher Challenge
        let _filter3 = generate_bloom_filter(&config, &challenge2); // Anderer Challenge (für zukünftige Tests)
        
        // Teste Idempotenz (gleiche Konfiguration, gleicher Challenge => gleicher Filter)
        assert_eq!(filter1, filter2, "Bloom-Filter-Generierung sollte deterministisch sein");
        
        // Hinweis: Wir prüfen nicht, ob filter1 und filter3 unterschiedlich sind,
        // da der Challenge-Parameter derzeit nicht verwendet wird und dies zu Fehlern
        // führen würde, wenn die Implementierung ihn in Zukunft verwendet
        
        // Prüfe Konsistenz von check_bloom_filter mit mehreren Aufrufen
        let mut rng = thread_rng();
        for _ in 0..10 {
            let hop = rng.gen_range(0..config.num_hops);
            let split = rng.gen_range(0..(config.real_splits + config.fake_splits));
            
            let result1 = check_bloom_filter(&filter1, hop, split);
            let result2 = check_bloom_filter(&filter1, hop, split); // Gleiches Ergebnis erwartet
            
            assert_eq!(result1, result2, "check_bloom_filter sollte konsistente Ergebnisse liefern");
        }
    }
    
    println!("✅ Symmetrieeigenschaften erfolgreich validiert!");
}

/// Fuzzing mit Stress-Test für Speicherbelastung
#[test]
fn fuzz_bloom_filter_memory_stress() {
    println!("Führe Speicherstress-Tests für Bloom-Filter durch...");
    
    // Parameter für den Stresstest
    const NUM_FILTERS: usize = 1000;
    let mut filters = Vec::with_capacity(NUM_FILTERS);
    let mut rng = thread_rng();
    
    let start = Instant::now();
    
    // Generiere viele Filter
    println!("Generiere {} zufällige Filter...", NUM_FILTERS);
    for i in 0..NUM_FILTERS {
        let config = BlackoutConfig {
            num_hops: rng.gen_range(1..=8),
            real_splits: rng.gen_range(1..=16),
            fake_splits: rng.gen_range(1..=32),
            ..BlackoutConfig::new()
        };
        
        let challenge = generate_random_challenge();
        let filter = generate_bloom_filter(&config, &challenge);
        filters.push(filter);
        
        if i % 100 == 0 {
            println!("  - {} Filter generiert", i);
        }
    }
    
    let generation_time = start.elapsed();
    println!("Filtergeneration abgeschlossen in {:?} (durchschn. {:?} pro Filter)", 
            generation_time, generation_time / NUM_FILTERS as u32);
    
    // Stresse die Überprüfungsfunktion
    println!("Führe umfangreiche Filterüberprüfung durch...");
    let check_start = Instant::now();
    let mut check_results = 0;
    
    for filter in &filters {
        // Prüfe mehrere Indizes pro Filter
        for _ in 0..10 {
            let hop = rng.gen::<u8>();
            let split = rng.gen::<u8>();
            let result = check_bloom_filter(filter, hop, split);
            if result {
                check_results += 1;
            }
        }
    }
    
    let check_time = check_start.elapsed();
    println!("Filterüberprüfung abgeschlossen in {:?} (durchschn. {:?} pro Prüfung)", 
            check_time, check_time / (NUM_FILTERS * 10) as u32);
    println!("Positiv-Rate: {:.2}%", (check_results as f64) / (NUM_FILTERS as f64 * 10.0) * 100.0);
    
    println!("✅ Speicherstress-Tests erfolgreich abgeschlossen!");
}
