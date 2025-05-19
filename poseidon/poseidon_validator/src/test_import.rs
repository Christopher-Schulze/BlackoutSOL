fn main() {
    println!("Teste solana-poseidon Import-Struktur");
    
    // Drucke die verfügbaren Module und Typen
    println!("Verfügbare Module:");
    for name in &["solana_poseidon", "solana_poseidon::poseidon"] {
        match std::env::var(name) {
            Ok(_) => println!("  - {} (gefunden)", name),
            Err(_) => println!("  - {} (nicht gefunden)", name),
        }
    }
}
