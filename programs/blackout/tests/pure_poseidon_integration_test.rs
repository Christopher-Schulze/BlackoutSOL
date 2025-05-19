#[cfg(test)]
mod integration_tests {
    use blackout::{
        pure_poseidon::{self, hash},
        poseidon_validator,
    };
    
    /// Testet alle drei Schichten der Poseidon-Integration auf korrekte Validierung
    #[test]
    fn test_poseidon_validation_across_layers() {
        // 1. Layer: Reine Implementierung testen
        let pure_validation = hash::validate_parameters();
        assert!(pure_validation.is_ok(), "Pure Poseidon-Validierung fehlgeschlagen: {:?}", pure_validation);
        
        // 2. Layer: Bridge testen (ohne Anchor-Kontext)
        let bridge_test = blackout::pure_poseidon::bridge::validate_poseidon_parameters();
        assert!(bridge_test.is_ok(), "Bridge-Validierung fehlgeschlagen");
        
        // 3. Layer: Validator-Schnittstelle testen (ohne Anchor-Kontext)
        let validator_test = poseidon_validator::validate_for_test();
        assert!(validator_test.is_ok(), "Validator-Validierung fehlgeschlagen: {:?}", validator_test);
    }
    
    /// Testet die Konsistenz der Hash-Ergebnisse zwischen den Implementierungen
    #[test]
    fn test_hash_consistency_across_implementations() {
        // Standardisierte Testeingaben
        let simple_input = b"BlackoutSOL Integration Test";
        let complex_input = [0x42; 64]; // Längere Eingabe mit definierten Werten
        
        // Hash über die verschiedenen Schichten generieren
        let results = [
            // Layer 1: Pure-Implementierung
            hash::generate_hash(&[simple_input]).expect("Pure hash (simple) failed"),
            hash::generate_hash(&[&complex_input[..]]).expect("Pure hash (complex) failed"),
            
            // Layer 3: Validator-API (delegiert über Bridge zur puren Implementierung)
            poseidon_validator::generate_hash_for_test(&[simple_input]).expect("Validator hash (simple) failed"),
            poseidon_validator::generate_hash_for_test(&[&complex_input[..]]).expect("Validator hash (complex) failed"),
        ];
        
        // Alle Implementierungen sollten identische Ergebnisse liefern
        assert_eq!(results[0], results[2], "Hash-Inkonsistenz bei einfacher Eingabe");
        assert_eq!(results[1], results[3], "Hash-Inkonsistenz bei komplexer Eingabe");
        
        // Verschiedene Eingaben sollten verschiedene Hashes erzeugen
        assert_ne!(results[0], results[1], "Hash-Kollision zwischen einfacher und komplexer Eingabe");
    }
    
    /// Testet Batch-Operationen über alle Implementierungsschichten
    #[test]
    fn test_batch_operations_consistency() {
        // Standardisierte Batch-Testeingaben
        let test_inputs = vec![
            vec![&b"Input1"[..], &b"Salt1"[..]],      // Multi-input im ersten Batch
            vec![&b"Input2"[..]],                    // Single-input im zweiten Batch
            vec![&b"Input3"[..], &b"Salt3"[..]],      // Multi-input im dritten Batch
        ];
        
        // Batch-Verarbeitung über verschiedene Schichten
        let pure_results = hash::batch_hash(&test_inputs)
            .expect("Pure batch processing failed");
            
        let validator_results = poseidon_validator::batch_hash_for_test(&test_inputs)
            .expect("Validator batch processing failed");
        
        // Ergebniskonsistenz prüfen
        assert_eq!(pure_results.len(), validator_results.len(), 
                   "Unterschiedliche Batch-Ergebnisanzahl");
                   
        for (i, (pure, validator)) in pure_results.iter()
                                      .zip(validator_results.iter())
                                      .enumerate() {
            assert_eq!(pure, validator, 
                      "Batch-Hash-Ergebnis {} stimmt nicht überein", i);
        }
    }
    
    /// Testet die Hex-Konvertierung mit verschiedenen Eingabeformaten
    #[test]
    fn test_hex_conversions() {
        use blackout::pure_poseidon::constants::scalar_from_hex;
        
        // Validiere verschiedene Hex-Formate
        let test_cases = [
            "3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd", // Ohne Präfix
            "0x3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd", // Mit Präfix
        ];
        
        // Konvertierungsresultate prüfen
        let results: Vec<_> = test_cases.iter()
            .map(|&hex| scalar_from_hex(hex).expect("Konvertierung fehlgeschlagen"))
            .collect();
            
        // Alle Konvertierungen des gleichen Wertes sollten identisch sein
        for i in 1..results.len() {
            assert_eq!(results[0], results[i], 
                      "Unterschiedliche Skalare für gleichen Hex-Wert mit verschiedenen Formaten");
        }
    }
    
    /// Testet die MDS-Matrix-Generierung und -Verwendung
    #[test]
    fn test_mds_matrix() {
        use blackout::pure_poseidon::constants::get_mds_matrix;
        
        // Matrix generieren und prüfen
        let matrix = get_mds_matrix().expect("MDS-Matrix-Generierung fehlgeschlagen");
        
        // Dimensionen prüfen (Poseidon WIDTH = 3)
        assert_eq!(matrix.len(), 3, "Falsche Anzahl von Zeilen in der MDS-Matrix");
        for row in &matrix {
            assert_eq!(row.len(), 3, "Falsche Anzahl von Spalten in der MDS-Matrix");
        }
        
        // Einfacher Matrix-Struktur-Check: Keine leeren Einträge
        for row in &matrix {
            for scalar in row {
                let bytes = scalar.to_bytes();
                // Ein gültiger Skalar sollte nicht nur Nullen enthalten
                assert!(!bytes.iter().all(|&b| b == 0), "Ungültiger Skalar in der MDS-Matrix");
            }
        }
    }
    
    /// Testet die Fehlerbehandlung bei ungültigen Eingaben
    #[test]
    fn test_error_handling() {
        use blackout::pure_poseidon::constants::scalar_from_hex;
        
        // Ungültiger Hex-String (zu kurz)
        let invalid_hex = "abc";
        let result = scalar_from_hex(invalid_hex);
        assert!(result.is_err(), "Fehlerhafte Eingabe wurde akzeptiert");
        
        // Ungültiger Hex-String (ungültige Zeichen)
        let invalid_chars = "0xXYZ12345";
        let result = scalar_from_hex(invalid_chars);
        assert!(result.is_err(), "Ungültige Hex-Zeichen wurden akzeptiert");
        
        // Prüfen, ob die Fehlermeldungen sinnvoll sind
        match scalar_from_hex(invalid_hex) {
            Err(pure_poseidon::PurePoseidonError::ConversionError(_)) => { /* Erwarteter Fehlertyp */ },
            Err(e) => panic!("Falscher Fehlertyp: {:?}", e),
            Ok(_) => panic!("Fehlerhafte Eingabe wurde akzeptiert"),
        }
    }
}
