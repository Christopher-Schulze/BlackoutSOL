# BlackoutSOL Poseidon Standalone

*Zuletzt aktualisiert: 15. Mai 2025*

Eine robuste, eigenständige Implementierung der Poseidon-Hash-Funktionalität für das BlackoutSOL-Projekt, die unabhängig vom Anchor-Framework funktioniert und gleichzeitig eine optionale Anchor-Integrationsschicht bietet.

## Status

✅ **Vollständig implementiert** - Die Bibliothek ist vollständig funktionsfähig und alle Tests laufen erfolgreich.
✅ **In BlackoutSOL integriert** - Das eigenständige Paket wird erfolgreich in der BlackoutSOL-Anwendung verwendet.

## Motivation

Nach intensiver Analyse der Integrationsprobleme zwischen Poseidon-Hashing und dem Anchor-Framework wurde diese Bibliothek entwickelt, um:

1. **Stabile Kernfunktionalität** zu bieten, die von Anchor-Kompilierungsproblemen isoliert ist
2. **Saubere Fehlerbehandlung** mit aussagekräftigen Fehlermeldungen zu implementieren
3. **Flexible Integration** sowohl in Anchor-Projekte als auch in eigenständige Anwendungen zu ermöglichen
4. **Umfassende Testabdeckung** zur Gewährleistung korrekter Funktionalität bereitzustellen

## Funktionen

- 🔍 **Vollständige Poseidon-Implementierung** mit BN254-Parametern
- 🛠️ **Unterstützung für einzelne und Batch-Operationen**
- ⚙️ **Robuste Fehlerbehandlung** mit Nutzung von `thiserror`
- 🔄 **Konsistente und validierte Hash-Ergebnisse** über verschiedene Implementierungen hinweg
- 🧪 **Umfangreiche Testabdeckung** für alle Komponenten
- 🔗 **Optionale Anchor-Kompatibilitätsschicht** (über Feature-Flag)

## Nutzung

### Standalone Modus

```rust
use blackout_poseidon::{hash, constants, PoseidonError};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Parameter validieren
    hash::validate_parameters()?;
    
    // Hash berechnen
    let input = b"BlackoutSOL secure input";
    let hash_result = hash::generate_hash(&[input])?;
    println!("Hash Ergebnis: {}", hex::encode(hash_result));
    
    // Batch-Verarbeitung
    let inputs = vec![
        vec![&b"Input1"[..]],
        vec![&b"Input2"[..]],
    ];
    let batch_results = hash::batch_hash(&inputs)?;
    println!("Batch Ergebnisse: {:?}", 
             batch_results.iter()
                         .map(|h| hex::encode(h))
                         .collect::<Vec<_>>());
    
    Ok(())
}
```

### Mit Anchor-Kompatibilität

Aktivieren Sie das `anchor_compat`-Feature in Cargo.toml:

```toml
[dependencies]
blackout_poseidon = { path = "../poseidon_standalone", features = ["anchor_compat"] }
```

Dann im Anchor-Programmcode:

```rust
use anchor_lang::prelude::*;
use blackout_poseidon::anchor as poseidon;

#[program]
mod my_program {
    use super::*;
    
    pub fn process_data(ctx: Context<ProcessData>, input: Vec<u8>) -> Result<()> {
        // Poseidon-Hash mit Anchor-kompatibler Fehlerbehandlung
        let hash_result = poseidon::generate_hash(&[&input])?;
        msg!("Hash Ergebnis: {:?}", hash_result);
        
        Ok(())
    }
}
```

## Integration mit BlackoutSOL

### Option 1: Direkte Nutzung

```rust
// In BlackoutSOL-Programmen außerhalb von Anchor
use blackout_poseidon::hash;

fn process_data() -> std::result::Result<(), PoseidonError> {
    let input = b"BlackoutSOL Data";
    let hash = hash::generate_hash(&[input])?;
    // Weitere Verarbeitung...
    Ok(())
}
```

### Option 2: Anchor-Integration

```rust
// In Anchor-Programmcode
use anchor_lang::prelude::*;
use blackout_poseidon::anchor;

#[program]
fn my_instruction(ctx: Context<MyContext>, data: Vec<u8>) -> Result<()> {
    anchor::generate_hash(&[&data])?;
    // Weitere Programmlogik
    Ok(())
}
```

## Fehlerbehebung

Die Bibliothek enthält drei Hauptfehlertypen:

- **HashingError**: Probleme bei der eigentlichen Hash-Berechnung
- **ValidationError**: Fehler bei der Validierung von Parametern oder Konsistenzprüfungen
- **ConversionError**: Probleme bei der Datentypkonvertierung, z.B. Hex-Strings zu Skalaren

Für detaillierte Fehlerbehandlung:

```rust
match hash::generate_hash(&[input]) {
    Ok(hash) => { /* Erfolg */ },
    Err(PoseidonError::HashingError(msg)) => { /* Hash-Operation fehlgeschlagen */ },
    Err(PoseidonError::ValidationError(msg)) => { /* Validierung fehlgeschlagen */ },
    Err(PoseidonError::ConversionError(msg)) => { /* Konvertierung fehlgeschlagen */ },
}
```

## Nächste Schritte

1. **Performance-Optimierungen** für große Datenmengen oder Batch-Operationen
2. **Weitere Parameter-Sets** neben BN254 hinzufügen
3. **Zero-Knowledge Integrations** für spezielle ZK-Anwendungsfälle ausbauen
