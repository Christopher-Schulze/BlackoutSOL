#!/bin/bash

# Extrahiere alle Hex-Werte aus der Rohdatei und formatiere sie für Rust
cat /Users/christopher/CODE2/BlackoutSOL/src/utils/poseidon_params_t3.txt | 
    grep -o "0x[0-9a-f]\+" | 
    sed 's/^0x/scalar_from_hex("/g' | 
    sed 's/$/"),/g' > /Users/christopher/CODE2/BlackoutSOL/poseidon_constants_rust.txt

echo "Umwandlung abgeschlossen. Die formatierten Konstanten befinden sich in poseidon_constants_rust.txt"
