// Teste verschiedene Importvarianten für solana-poseidon

fn main() {
    println!("Teste solana-poseidon Import-Varianten");
    
    // Versuche 1: Direkt die Bibliothek ohne weiteren Pfad
    println!("solana_poseidon: {:?}", std::any::type_name_of_val(&solana_poseidon::VERSION));
    
    // Zeige die verfügbaren Module/Elemente
    println!("Verfügbare Elemente in solana_poseidon:");
    
    // Wir können nicht direkt Reflection nutzen, aber wir können die Dokumentation anzeigen
    println!("Schaue in der Dokumentation/Source nach");
}
