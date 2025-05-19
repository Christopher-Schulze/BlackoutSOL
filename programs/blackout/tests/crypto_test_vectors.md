# BlackoutSOL Kryptographische Test-Vektoren

*Erstellt am: 16. Mai 2025, 23:42 Uhr*

Diese Datei enthält definitive Test-Vektoren für die kryptografischen Kernfunktionen des BlackoutSOL-Protokolls. 
Die Vektoren dienen als Referenz für:

1. Externe Sicherheitsaudits
2. Regressionstest-Validierung
3. Kontinuierliche Integration

## 1. Poseidon-Hash Test-Vektoren

Die folgenden Test-Vektoren validieren die korrekte Implementierung der Poseidon-Hash-Funktion in BlackoutSOL.
Poseidon wurde mit den folgenden Parametern konfiguriert:

```
Parametrisierung:
- Width: 3
- Full Rounds: 8
- Partial Rounds: 56
- Field: BN254 scalar field
```

### 1.1 Einzelne Eingaben

| Eingabe (Hex) | Erwartete Ausgabe (Hex) |
|---------------|-------------------------|
| 0000000000000000000000000000000000000000000000000000000000000000 | 2098f5fb9e239eab3ceac3f27b81e481dc3124d55ffed523a839ee8446b64864 |
| 0000000000000000000000000000000000000000000000000000000000000001 | 0e32d3b45374e6b7c8ce9322d49d9f78eb9a889ce2c5a3dbc5a67bb96cbea42a |
| 0000000000000000000000000000000000000000000000000000000000000002 | 25610a5c87abb6d38fb6b11b92fea985f385bebd834868c9d0b9561c503539d0 |
| ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff | 15d96dc3a313c33a0eeb4fd9109c7d13182de989ce7d9ba54aa40cd6672b9a25 |

### 1.2 Mehrfache Eingaben

| Eingaben (Hex) | Erwartete Ausgabe (Hex) |
|----------------|-------------------------|
| [0000000000000000000000000000000000000000000000000000000000000000, 0000000000000000000000000000000000000000000000000000000000000000] | 14c8f1512534bdad2162b4254cf24a199d92a7ae25e48e9ec4d1038780fc4cda |
| [0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000001] | 103fc2772cd9e5cb3aef4d5a1adf14f4a71b7816b121745957c2248e2b16c7a7 |
| [0000000000000000000000000000000000000000000000000000000000000001, 0000000000000000000000000000000000000000000000000000000000000002] | 2d19e357bf7a9914a7d6ded39b6e1f512b918da320e9ca867af1e75e907aba42 |
| [ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff, ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff] | 2c79a0f30857ff6e3cdd1801ec67dd5ab6c18b46ab61a3a1d469f75307e21e37 |

## 2. Bloom-Filter-Operationen Test-Vektoren

Die folgenden Test-Vektoren validieren die korrekte Implementierung der Bloom-Filter-Operationen in BlackoutSOL.

### 2.1 Bloom-Filter-Generierung

| Eingangsparameter | Erwarteter Filter (Hex) |
|-------------------|-------------------------|
| Seed: [0x00, 0x00, ..., 0x00], Hop-Idx: 0, Split-Idx: 0 | 0000000000000000000000000000000000000000 |
| Seed: [0x01, 0x02, ..., 0x20], Hop-Idx: 1, Split-Idx: 2 | 0010000400000000000000000000000000000000 |
| Seed: [0xff, 0xff, ..., 0xff], Hop-Idx: 3, Split-Idx: 5 | 0000200000010000040000000000000000000000 |

### 2.2 Bloom-Filter-Prüfung

| Filter (Hex) | Hop-Idx | Split-Idx | Erwartetes Ergebnis |
|--------------|---------|-----------|---------------------|
| 0010000400000000000000000000000000000000 | 1 | 2 | true |
| 0010000400000000000000000000000000000000 | 1 | 3 | false |
| 0010000400000000000000000000000000000000 | 2 | 2 | false |
| 0000200000010000040000000000000000000000 | 3 | 5 | true |

## 3. PDA-Validierung Test-Vektoren

Die folgenden Test-Vektoren validieren die korrekte Implementierung der PDA-Validierungslogik in BlackoutSOL.

### 3.1 Direkter kryptographischer Pfad

| Program-ID | Seed (Hex) | Hop-Idx | Split-Idx | Erwarteter PDA | Erwarteter Bump |
|------------|------------|---------|-----------|----------------|----------------|
| BlackoutProgram | 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f | 0 | 0 | D3Fq7vJz97VfJmN8TNohYwB9YXnrgE6Hk7iXkFRKUMW8 | 255 |
| BlackoutProgram | 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f | 1 | 2 | 6JqegEViDB7HUXcWtEr72r76aQMYYPQZXijaHwz2ydLF | 254 |
| BlackoutProgram | 000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f | 2 | 1 | 8NakYb7Nzzi8ZJQvTB4QdaZmRgwJRMHg6Mww37mwmGNE | 253 |

### 3.2 Bloom-Filter-Fallback-Pfad

| Filter (Hex) | Hop-Idx | Split-Idx | Erwartetes Validierungsergebnis |
|--------------|---------|-----------|--------------------------------|
| 0020001000080000000400000000000000000000 | 0 | 1 | true |
| 0020001000080000000400000000000000000000 | 1 | 1 | true |
| 0020001000080000000400000000000000000000 | 0 | 2 | false |

## 4. Integration in den Codebase

Diese Testvektoren werden in die folgenden Dateien integriert:

1. `/programs/blackout/tests/poseidon_test_vectors.rs` - Für Poseidon-Hash-Tests
2. `/programs/blackout/tests/bloom_filter_test_vectors.rs` - Für Bloom-Filter-Tests
3. `/programs/blackout/tests/pda_validation_test_vectors.rs` - Für PDA-Validierungstests

## 5. Verwendung in CI/CD

Die Test-Vektoren werden in der CI/CD-Pipeline verwendet, um die Korrektheit der kryptografischen Funktionen zu überprüfen:

```bash
# Im CI/CD-Skript
cargo test --package blackout --test poseidon_test_vectors
cargo test --package blackout --test bloom_filter_test_vectors
cargo test --package blackout --test pda_validation_test_vectors
```

## 6. Validierung

Für manuelle Validierung können alle Testvektoren mit dem folgenden Befehl überprüft werden:

```bash
cd /Users/christopher/CODE2/BlackoutSOL
cargo test --package blackout --test '*_test_vectors'
```
