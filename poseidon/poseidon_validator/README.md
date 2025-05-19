# Poseidon-Parameter-Validierungstool

Dieses Werkzeug validiert die korrekte Implementierung der Poseidon-Hash-Funktion für Zero-Knowledge-Proof-Anwendungen.

## Funktionen

Das Tool führt verschiedene Tests durch, um die korrekte Funktionalität der Poseidon-Hash-Funktion zu validieren:

1. **Konsistenztest**: Stellt sicher, dass identische Eingaben immer zu identischen Hash-Werten führen.
2. **Verschiedene Eingabedaten**: Testet die Hash-Funktion mit unterschiedlichen Eingabedaten.
3. **Multi-Input-Tests**: Überprüft die Funktionalität bei mehreren Eingaben.
4. **Reihenfolgeabhängigkeit**: Bestätigt, dass die Reihenfolge der Eingabedaten das Ergebnis beeinflusst.
5. **Verteilungstests**: Validiert die gleichmäßige Verteilung der Hash-Werte.

## Verwendung

```bash
# Standard-Konsistenztest ausführen
cargo run

# Vollständige Validierung durchführen
cargo run -- --full-validation

# Nur Konsistenztest ausführen
cargo run -- --consistency-only
```

## Parameter

Die Implementierung verwendet die Poseidon-Hash-Funktion mit den folgenden Parametern:

- **Kurventyp**: BN254 (254-Bit-Primfeld)
- **S-Box**: x^5
- **Volle Runden**: 8
- **Teilrunden**: 56
- **Breite**: 3 (für 2 Eingaben)

## Abhängigkeiten

- `solana-poseidon`: Implementierung der Poseidon-Hash-Funktion
- `clap`: Kommandozeilenargumente
- `hex`: Für die hexadezimale Darstellung von Byte-Arrays

## Integration

Dieses Validierungstool kann in CI/CD-Pipelines integriert werden, um die korrekte Funktionalität der Poseidon-Hash-Funktion nach Änderungen zu überprüfen.
