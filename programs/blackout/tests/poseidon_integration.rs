// Integrations-Tests für die Poseidon-Bibliothek 
// Diese Tests können mit `cargo test -- --nocapture` ausgeführt werden

// Die direkten Abhängigkeiten einbinden
use solana_poseidon::{hashv, Parameters, Endianness};
use curve25519_dalek::scalar::Scalar;
use num_bigint::BigUint;

// Test für die allgemeine Poseidon-Funktionalität
#[test]
fn test_poseidon_hash_functionality() {
    println!("🧪 Teste grundlegende Poseidon-Hash-Funktionalität...");
    
    // Einfache Eingabe für den Hash
    let mut test_input = [0u8; 32];
    test_input[31] = 0x42; // Ein einfacher Wert für den Test
    
    // Hash berechnen
    let hash_result = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[&test_input]
    ).expect("Hash-Berechnung sollte erfolgreich sein");
    
    println!("✅ Hash erfolgreich berechnet: {:?}", hash_result);
    assert!(!hash_result.to_bytes().iter().all(|&b| b == 0), 
            "Hash sollte nicht nur aus Nullen bestehen");
}

// Test für die Hash-Konsistenz
#[test]
fn test_poseidon_hash_consistency() {
    println!("🔄 Teste Poseidon-Hash-Konsistenz...");
    
    // Testvektor
    let test_input = b"BlackoutSOL-Integration-Test";
    
    // Ersten Hash berechnen
    let hash1 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).expect("Erste Hash-Berechnung sollte erfolgreich sein");
    
    // Zweiten Hash mit gleicher Eingabe berechnen
    let hash2 = hashv(
        Parameters::Bn254X5,
        Endianness::BigEndian,
        &[test_input]
    ).expect("Zweite Hash-Berechnung sollte erfolgreich sein");
    
    println!("✅ Beide Hashes erfolgreich berechnet");
    assert_eq!(hash1, hash2, "Gleiche Eingabe sollte gleichen Hash erzeugen");
}

// Test für Scalar-Umwandlung aus Hex-Strings
#[test]
fn test_scalar_from_hex() {
    println!("🔢 Teste Scalar-Konvertierung aus Hex-Strings...");
    
    // Einige Test-Hex-Strings
    let hex_values = [
        "3d955d6c02fe4d7cb500e12f2b55eff668a7b4386bd27413766713c93f2acfcd",
        "3798866f4e6058035dcf8addb2cf1771fac234bcc8fc05d6676e77e797f224bf",
    ];
    
    // Konvertiere alle Hex-Strings zu Scalar-Werten
    let scalars: Vec<Scalar> = hex_values.iter().map(|&s| scalar_from_hex(s)).collect();
    
    println!("✅ Alle Hex-Strings erfolgreich zu Scalar konvertiert");
    assert_eq!(scalars.len(), hex_values.len(), 
               "Anzahl der konvertierten Scalars sollte mit der Anzahl der Eingaben übereinstimmen");
    
    // Überprüfen, dass die konvertierten Werte nicht trivial sind
    for scalar in &scalars {
        assert!(!scalar.to_bytes().iter().all(|&b| b == 0), 
                "Konvertierter Scalar sollte nicht Null sein");
    }
}

// Helper-Funktion zur Konvertierung von Hex-Strings zu Scalar-Werten
fn scalar_from_hex(s: &str) -> Scalar {
    let s = if s.starts_with("0x") { &s[2..] } else { s };
    
    // Hexadezimal zu BigUint konvertieren
    let big_uint = BigUint::parse_bytes(s.as_bytes(), 16)
        .unwrap_or_else(|| panic!("Failed to parse hex string for Scalar: {}", s));
    
    // BigUint zu bytes konvertieren
    let bytes = big_uint.to_bytes_be();
    
    // Scalar von bytes konvertieren (aktualisierte Methode)
    let mut array = [0u8; 32];
    let len = std::cmp::min(bytes.len(), 32);
    array[32 - len..].copy_from_slice(&bytes[..len]);
    Scalar::from_bytes_mod_order(array)
}
