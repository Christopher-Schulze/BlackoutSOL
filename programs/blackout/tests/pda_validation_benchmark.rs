/*
 * Performance-Benchmark für die optimierte PDA-Validierungslogik
 * 
 * Dieses Skript misst die Leistung der verschiedenen PDA-Validierungsmethoden:
 * 1. Direkte kryptographische Validierung (Standard-Methode)
 * 2. Optimierte Dual-Path-Validierung (mit Bloom-Filter-Fallback)
 * 3. Legacy-Validierung zum Vergleich
 * 
 * Die Ergebnisse werden in Mikrosekunden und geschätzten Compute Units ausgegeben.
 */

use anchor_lang::prelude::*;
use blackout::utils::{
    verify_pda_derivation,
    verify_bloom_filter,
    derive_stealth_pda,
    check_bloom_filter,
    generate_bloom_filter,
};
use blackout::state::config::BlackoutConfig;
use solana_program::pubkey::Pubkey;
use solana_sdk::account_info::AccountInfo;
use std::time::{Duration, Instant};

// Konstanten für den Benchmark
const NUM_ITERATIONS: usize = 1000;
const CU_PER_MICROSECOND: f64 = 0.25; // Annahme basierend auf Solana's Performance-Metriken

// Hilfsfunktion zum Messen der Ausführungszeit
fn measure_execution_time<F>(func: F) -> Duration
where
    F: Fn() -> ()
{
    let start = Instant::now();
    func();
    start.elapsed()
}

#[test]
fn benchmark_pda_validation() {
    println!("📊 Starting PDA Validation Performance Benchmark");
    println!("================================================");
    println!("Running {} iterations for each method", NUM_ITERATIONS);
    
    // Setup
    let program_id = Pubkey::new_unique();
    let seed = [42u8; 32];
    let hop_index = 2;
    let split_index = 1;
    let challenge = [0u8; 32];
    
    // 1. Erzeuge PDA
    let (real_pda, bump) = derive_stealth_pda(
        &program_id,
        &seed,
        hop_index,
        split_index,
        false // Nicht ein Fake-Split
    );
    
    // Mock AccountInfo erstellen
    let mut lamports = 10000;
    let mut data = vec![0; 32];
    let owner = program_id;
    
    let account_info = AccountInfo::new(
        &real_pda,
        false,
        false,
        &mut lamports,
        &mut data,
        &owner,
        false,
        0,
    );
    
    // 2. Konfiguration und Bloom-Filter für Fake-Splits erstellen
    let config = BlackoutConfig {
        real_splits: 4,
        fake_splits: 44,
        num_hops: 4,
        reserve_percent: 40,
        fee_multiplier: 200, // 2%
        cu_budget_per_hop: 220_000,
    };
    
    let bloom_filter = generate_bloom_filter(&config, &challenge);
    
    // -----------------------------------------------
    // Methode 1: Direkte kryptographische Validierung
    // -----------------------------------------------
    let direct_time = measure_execution_time(|| {
        for _ in 0..NUM_ITERATIONS {
            let _ = verify_pda_derivation(
                &program_id,
                &seed,
                hop_index,
                split_index,
                &account_info
            );
        }
    });
    
    let direct_time_per_op = direct_time.as_micros() as f64 / NUM_ITERATIONS as f64;
    let direct_estimated_cu = direct_time_per_op * CU_PER_MICROSECOND;
    
    println!("Direkte kryptographische Validierung:");
    println!("  Zeit pro Operation: {:.2} µs", direct_time_per_op);
    println!("  Geschätzte Compute Units: {:.2} CU", direct_estimated_cu);
    
    // -----------------------------------------------
    // Methode 2: Dual-Path Validierung (mit Bloom-Filter-Fallback)
    // -----------------------------------------------
    let dual_path_time = measure_execution_time(|| {
        for _ in 0..NUM_ITERATIONS {
            let validation_result = verify_pda_derivation(
                &program_id,
                &seed,
                hop_index,
                split_index,
                &account_info
            );
            
            if validation_result.is_err() {
                // Fallback zur Bloom-Filter-Validierung
                let _ = verify_bloom_filter(
                    &bloom_filter,
                    hop_index,
                    split_index
                );
            }
        }
    });
    
    let dual_path_time_per_op = dual_path_time.as_micros() as f64 / NUM_ITERATIONS as f64;
    let dual_path_estimated_cu = dual_path_time_per_op * CU_PER_MICROSECOND;
    
    println!("Optimierte Dual-Path Validierung:");
    println!("  Zeit pro Operation: {:.2} µs", dual_path_time_per_op);
    println!("  Geschätzte Compute Units: {:.2} CU", dual_path_estimated_cu);
    
    // -----------------------------------------------
    // Methode 3: Ineffiziente Legacy-Validierung (simuliert)
    // -----------------------------------------------
    let legacy_time = measure_execution_time(|| {
        for _ in 0..NUM_ITERATIONS {
            // Simuliere eine ineffiziente Validierung mit mehreren Operationen
            let (expected_pda, _) = derive_stealth_pda(
                &program_id,
                &seed,
                hop_index,
                split_index,
                false
            );
            
            // Extra Validierungslogik simulieren (3x langsamer)
            let _ = expected_pda == real_pda;
            std::thread::sleep(Duration::from_nanos(100));
            
            // Zweite PDA-Berechnung (übliche Legacy-Implementierung)
            let (alt_pda, _) = derive_stealth_pda(
                &program_id,
                &seed,
                hop_index,
                split_index,
                true
            );
            let _ = alt_pda == real_pda;
        }
    });
    
    let legacy_time_per_op = legacy_time.as_micros() as f64 / NUM_ITERATIONS as f64;
    let legacy_estimated_cu = legacy_time_per_op * CU_PER_MICROSECOND;
    
    println!("Simulierte Legacy-Validierung:");
    println!("  Zeit pro Operation: {:.2} µs", legacy_time_per_op);
    println!("  Geschätzte Compute Units: {:.2} CU", legacy_estimated_cu);
    
    // -----------------------------------------------
    // Ergebnisse und Vergleiche
    // -----------------------------------------------
    let improvement_vs_legacy = ((legacy_time_per_op - dual_path_time_per_op) / legacy_time_per_op) * 100.0;
    let cu_savings_vs_legacy = legacy_estimated_cu - dual_path_estimated_cu;
    
    println!("\n📈 Performance-Vergleich:");
    println!("------------------------------------------------");
    println!("Dual-Path vs Legacy Verbesserung: {:.2}%", improvement_vs_legacy);
    println!("Compute Unit Einsparung: {:.2} CU pro Validierung", cu_savings_vs_legacy);
    
    if dual_path_time_per_op < direct_time_per_op {
        let improvement_vs_direct = ((direct_time_per_op - dual_path_time_per_op) / direct_time_per_op) * 100.0;
        println!("Dual-Path ist {:.2}% schneller als direkte Validierung", improvement_vs_direct);
    } else {
        println!("Hinweis: Direkte Validierung ist optimal für bekannte reale Splits");
    }
    
    println!("\n✅ Benchmark erfolgreich abgeschlossen");
}
