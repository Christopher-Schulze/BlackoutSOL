/*
 * Bloom-Filter-Testvektoren für BlackoutSOL
 * 
 * Diese Tests validieren die korrekte Implementierung der Bloom-Filter-Operationen
 * gegen die definierten Testvektoren in crypto_test_vectors.md.
 */

use blackout::utils::{check_bloom_filter, generate_bloom_filter};
use blackout::state::config::BlackoutConfig;

/// Test für die Generierung von Bloom-Filtern
#[test]
fn test_bloom_filter_generation() {
    println!("Teste Bloom-Filter-Generierung...");
    
    // Erstellen von BlackoutConfigs mit unterschiedlichen Werten
    let config1 = BlackoutConfig::new(); // Standardwerte
    let mut config2 = BlackoutConfig::new();
    config2.real_splits = 2;  // Andere Konfiguration für andere Filterform
    
    let challenge = [0u8; 32]; // Challenge (wird tatsächlich nicht verwendet)
    
    // Bloom-Filter generieren
    let filter1 = generate_bloom_filter(&config1, &challenge);
    let filter2 = generate_bloom_filter(&config2, &challenge);
    
    // Ausgabe der generierten Filter als Hex-Strings
    println!("Filter 1 (Standardconfig): {}", hex::encode(&filter1));
    println!("Filter 2 (Angepasste Config): {}", hex::encode(&filter2));
    
    // Validiere, dass unterschiedliche Konfigurationen unterschiedliche Filter erzeugen
    // (Die Challenge hat keinen Einfluss, da der Parameter nicht verwendet wird)
    assert_ne!(filter1, filter2, "Unterschiedliche Konfigurationen sollten unterschiedliche Filter erzeugen");
    
    // Überprüfe, dass die Filter die richtige Länge haben
    assert_eq!(filter1.len(), 16, "Bloom-Filter sollte 16 Bytes lang sein");
    
    println!("✅ Bloom-Filter-Generierungstest bestanden!");
}

/// Test für die Bloom-Filter-Überprüfung
#[test]
fn test_bloom_filter_verification() {
    println!("Teste Bloom-Filter-Überprüfung...");
    
    // Erstellen einer BlackoutConfig mit Standardwerten
    let config = BlackoutConfig::new();
    let challenge = [0u8; 32]; // Null-Challenge für einfache Reproduzierbarkeit
    
    // Bloom-Filter generieren
    let filter = generate_bloom_filter(&config, &challenge);
    println!("Generierter Filter: {}", hex::encode(&filter));
    
    // Überprüfe, dass Fake-Splits als fake erkannt werden
    // Nach der Konfiguration: num_hops=4, real_splits=4, fake_splits=44
    for hop in 0..config.num_hops {
        // Reale Splits sollten als nicht-fake erkannt werden
        for split in 0..config.real_splits {
            let result = check_bloom_filter(&filter, hop, split);
            assert!(!result, "Split {},{} sollte nicht als fake erkannt werden", hop, split);
        }
        
        // Fake-Splits sollten als fake erkannt werden
        for split in config.real_splits..(config.real_splits + config.fake_splits) {
            let result = check_bloom_filter(&filter, hop, split);
            assert!(result, "Split {},{} sollte als fake erkannt werden", hop, split);
        }
    }
    
    println!("✅ Bloom-Filter-Überprüfungstest bestanden!");
}

/// Test für die End-to-End-Funktionalität des Bloom-Filters
#[test]
fn test_bloom_filter_end_to_end() {
    println!("Teste Bloom-Filter End-to-End...");
    
    // Erstellen einer angepassten BlackoutConfig mit kleineren Werten für schnellere Tests
    let mut config = BlackoutConfig::new();
    config.num_hops = 2;
    config.real_splits = 2;
    config.fake_splits = 4;
    
    // Verschiedene Challenges
    let challenges = [
        [0u8; 32],
        [1u8; 32],
        [0xffu8; 32],
        {
            let mut arr = [0u8; 32];
            for i in 0..32 {
                arr[i] = (i * 7) as u8;
            }
            arr
        }
    ];
    
    for challenge in challenges.iter() {
        // Filter generieren
        let filter = generate_bloom_filter(&config, challenge);
        
        // Überprüfen, dass alle realen Splits als nicht-fake erkannt werden
        for hop in 0..config.num_hops {
            for split in 0..config.real_splits {
                let result = check_bloom_filter(&filter, hop, split);
                assert!(!result, "Realer Split {},{} fälschlicherweise als fake erkannt", hop, split);
            }
        }
        
        // Überprüfen, dass alle fake Splits als fake erkannt werden
        for hop in 0..config.num_hops {
            for split in config.real_splits..(config.real_splits + config.fake_splits) {
                let result = check_bloom_filter(&filter, hop, split);
                assert!(result, "Fake Split {},{} nicht als fake erkannt", hop, split);
            }
        }
    }
    
    println!("✅ Alle Bloom-Filter End-to-End-Tests bestanden!");
}
