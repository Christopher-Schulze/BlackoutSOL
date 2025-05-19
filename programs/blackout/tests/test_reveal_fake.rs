/// Tests für die optimierte Reveal-Fake-Funktionalität
///
/// Diese Tests validieren die korrekte Offenlegung von Fake-Splits,
/// die Bloom-Filter-Funktionalität und die Sicherheitsmechanismen.

use anchor_lang::prelude::*;
use std::str::FromStr;
use solana_program_test::*;
use solana_sdk::{
    signature::{Keypair, Signer},
    transaction::Transaction,
    commitment_config::CommitmentLevel,
};

mod test_framework;
use test_framework::BlackoutTestFramework;
use blackout::{
    state::*,
    errors::BlackoutError,
};

// Test für die erfolgreiche Offenlegung eines Fake-Splits
#[tokio::test]
async fn test_reveal_fake_success() {
    // 1. Test-Framework initialisieren
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Transfer mit Fake-Splits initialisieren
    let amount = 500_000_000; // 0.5 SOL in Lamports
    
    // Liste von Fake-Split-Indizes für den Test
    let fake_indices = vec![5, 10, 15, 20, 25, 30, 35, 40];
    
    // Transfer mit definierten Fake-Splits initialisieren
    let (transfer_pda, seed) = framework.initialize_transfer_with_fake_splits(
        amount, 
        10, // 10% Reserve
        &fake_indices
    ).await.expect("Transfer-Initialisierung fehlgeschlagen");
    
    // 3. Ersten Hop ausführen (um den State zu aktivieren)
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Ausführung des ersten Hops fehlgeschlagen");
    
    println!("Transfer initialisiert und erster Hop ausgeführt");
    
    // 4. Reveal-Fake für einen bekannten Fake-Split ausführen
    let fake_hop_index = 1; // zweiter Hop (Index 1)
    let fake_split_index = 10; // Bekannter Fake-Split aus unserer Liste
    
    // PDA für diesen Fake-Split ableiten
    let (fake_pda, _) = framework.derive_split_pda(&seed, fake_hop_index, fake_split_index, true);
    
    // Fake-Split offenlegen
    framework.reveal_fake_split(&transfer_pda, fake_hop_index, fake_split_index, &fake_pda)
        .await
        .expect("Offenlegung des Fake-Splits fehlgeschlagen");
    
    println!("Fake-Split erfolgreich offengelegt: Hop {}, Split {}", 
             fake_hop_index, fake_split_index);
    
    // 5. Prüfen, dass der Transfer-State unverändert ist 
    // (Reveal-Fake sollte den State nicht modifizieren)
    let transfer_state = framework.get_transfer_state(&transfer_pda).await
        .expect("Konnte Transfer-State nicht abrufen");
    
    assert_eq!(transfer_state.current_hop, 1, 
              "Hop-Index sollte nach Reveal-Fake unverändert sein");
    assert!(!transfer_state.completed, 
            "Transfer sollte nach Reveal-Fake nicht als abgeschlossen markiert sein");
}

// Test für den Versuch, einen Real-Split als Fake zu enthüllen (muss fehlschlagen)
#[tokio::test]
async fn test_reveal_real_as_fake() {
    // 1. Test-Framework initialisieren
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Transfer mit Standard-Konfiguration initialisieren
    let amount = 300_000_000; // 0.3 SOL
    
    // Transfer mit definierten Fake-Splits initialisieren
    let (transfer_pda, seed) = framework.initialize_transfer(amount, 10)
        .await
        .expect("Transfer-Initialisierung fehlgeschlagen");
    
    // 3. Ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Ausführung des ersten Hops fehlgeschlagen");
    
    // 4. Versuch, einen Real-Split als Fake zu enthüllen (Indizes 0-3 sind Real)
    let real_hop_index = 1; // zweiter Hop
    let real_split_index = 2; // Ein Real-Split (Index < 4)
    
    // PDA für diesen Real-Split ableiten (als Real, nicht Fake)
    let (real_pda, _) = framework.derive_split_pda(&seed, real_hop_index, real_split_index, false);
    
    // Versuch, den Real-Split als Fake offenzulegen
    let result = framework.reveal_fake_split(&transfer_pda, real_hop_index, real_split_index, &real_pda)
        .await;
    
    // 5. Prüfen, ob der Versuch fehlgeschlagen ist
    assert!(
        result.is_err(),
        "Enthüllung eines Real-Splits als Fake sollte fehlschlagen"
    );
    
    println!("Test erfolgreich: Real-Split konnte nicht als Fake enthüllt werden");
}

// Test für den Versuch, einen nicht im Bloom-Filter enthaltenen Fake zu enthüllen
#[tokio::test]
async fn test_reveal_non_bloom_fake() {
    // 1. Test-Framework initialisieren
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Transfer mit spezifischen Fake-Splits initialisieren
    let amount = 400_000_000; // 0.4 SOL
    
    // Nur bestimmte Fake-Indizes in den Bloom-Filter aufnehmen
    let fake_indices = vec![7, 12, 17, 22];
    
    // Transfer mit begrenzten Fake-Splits initialisieren
    let (transfer_pda, seed) = framework.initialize_transfer_with_fake_splits(
        amount, 
        10, 
        &fake_indices
    ).await.expect("Transfer-Initialisierung fehlgeschlagen");
    
    // 3. Ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Ausführung des ersten Hops fehlgeschlagen");
    
    // 4. Versuch, einen nicht im Bloom-Filter enthaltenen Fake zu enthüllen
    let hop_index = 1;
    let non_bloom_split = 25; // Nicht in unserer Fake-Indizes-Liste
    
    // PDA für diesen nicht-Bloom-Fake ableiten
    let (fake_pda, _) = framework.derive_split_pda(&seed, hop_index, non_bloom_split, true);
    
    // Versuch, einen nicht-Bloom-Fake offenzulegen
    let result = framework.reveal_fake_split(&transfer_pda, hop_index, non_bloom_split, &fake_pda)
        .await;
    
    // 5. Prüfen, ob der Versuch fehlgeschlagen ist
    assert!(
        result.is_err(),
        "Enthüllung eines nicht im Bloom-Filter enthaltenen Fakes sollte fehlschlagen"
    );
    
    println!("Test erfolgreich: Nicht im Bloom-Filter enthaltener Fake konnte nicht enthüllt werden");
}

// Test für die Sicherheitsprüfung der PDA-Ableitung
#[tokio::test]
async fn test_reveal_fake_wrong_pda() {
    // 1. Test-Framework initialisieren
    let mut framework = BlackoutTestFramework::new().await;
    
    // 2. Transfer mit Fake-Splits initialisieren
    let amount = 200_000_000; // 0.2 SOL
    
    // Liste von Fake-Split-Indizes
    let fake_indices = vec![5, 10, 15, 20];
    
    // Transfer initialisieren
    let (transfer_pda, seed) = framework.initialize_transfer_with_fake_splits(
        amount, 
        10, 
        &fake_indices
    ).await.expect("Transfer-Initialisierung fehlgeschlagen");
    
    // 3. Ersten Hop ausführen
    framework.execute_hop(&transfer_pda, &seed, 0, 0, false)
        .await
        .expect("Ausführung des ersten Hops fehlgeschlagen");
    
    // 4. Versuch, einen Fake-Split mit falscher PDA zu enthüllen
    let hop_index = 1;
    let split_index = 10; // Ein gültiger Fake-Split
    
    // Falsche PDA verwenden (eine beliebige andere Adresse)
    let wrong_pda = Pubkey::new_unique();
    
    // Versuch mit falscher PDA
    let result = framework.reveal_fake_split(&transfer_pda, hop_index, split_index, &wrong_pda)
        .await;
    
    // 5. Prüfen, ob der Versuch fehlgeschlagen ist
    assert!(
        result.is_err(),
        "Enthüllung mit falscher PDA sollte fehlschlagen"
    );
    
    println!("Test erfolgreich: Fake-Split konnte nicht mit falscher PDA enthüllt werden");
}

// Erweitere die BlackoutTestFramework um spezifische Hilfsmethoden
impl BlackoutTestFramework {
    // Transfer mit spezifischen Fake-Splits initialisieren
    pub async fn initialize_transfer_with_fake_splits(
        &mut self,
        amount: u64, 
        reserve_percent: u8,
        fake_indices: &[u8],
    ) -> Result<(Pubkey, [u8; 32]), BanksClientError> {
        // Konfiguration mit angepasster Reserve erstellen
        let mut config = BlackoutConfig::new();
        config.reserve_percent = reserve_percent;
        
        // Challenge und Seed generieren
        let challenge = [7u8; 32]; // Fester Wert für einfacheres Testen
        let seed = self.generate_transfer_seed();
        
        // Proof-Daten für Tests erstellen
        let hyperplonk_proof = self.create_test_hyperplonk_proof(&challenge, &[amount / 4; 4]);
        let range_proof = self.create_test_range_proof(&challenge, &[[0; 32]; 8], true);
        
        // Merkle-Root erstellen
        let merkle_root = Keypair::new();
        
        // Transfer-PDA berechnen
        let (transfer_pda, _) = self.find_transfer_pda(&self.user.pubkey());
        
        // Bloom-Filter mit angepassten Fake-Indizes erstellen
        let fake_bloom = self.create_fake_bloom_filter(fake_indices);
        
        // Standardempfänger verwenden (uns selbst)
        let recipient = self.user.pubkey();
        
        // Initialisierungsinstruktion erstellen
        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new(transfer_pda, false),
                AccountMeta::new_readonly(recipient, false),
                AccountMeta::new_readonly(merkle_root.pubkey(), false),
                AccountMeta::new_readonly(system_program::ID, false),
                AccountMeta::new_readonly(solana_program::sysvar::clock::ID, false),
            ],
            data: blackout::instruction::Initialize {
                amount,
                hyperplonk_proof,
                range_proof,
                challenge,
                merkle_proof: vec![],
            }.data(),
        };
        
        // Transaktion ausführen
        self.execute_transaction(&[ix], &[&self.user, &merkle_root]).await?;
        
        // Fake-Bloom und andere Daten einprägen (für Tests)
        // In einer realen Implementierung würde dies durch die Instruktion geschehen
        self.force_set_bloom_filter(&transfer_pda, &fake_bloom).await?;
        
        Ok((transfer_pda, seed))
    }
    
    // Reveal-Fake-Instruktion ausführen
    pub async fn reveal_fake_split(
        &mut self,
        transfer_pda: &Pubkey,
        hop_index: u8,
        split_index: u8,
        fake_pda: &Pubkey,
    ) -> Result<(), BanksClientError> {
        // Reveal-Fake-Instruktion erstellen
        let ix = Instruction {
            program_id: self.program_id,
            accounts: vec![
                AccountMeta::new(self.user.pubkey(), true),
                AccountMeta::new_readonly(transfer_pda.clone(), false),
                AccountMeta::new_readonly(fake_pda.clone(), false),
                AccountMeta::new_readonly(system_program::ID, false),
            ],
            data: blackout::instruction::RevealFake {
                hop_index,
                split_index,
            }.data(),
        };
        
        // Transaktion ausführen
        self.execute_transaction(&[ix], &[&self.user]).await
    }
    
    // Hilfsmethode, um den Bloom-Filter direkt zu setzen (nur für Tests)
    async fn force_set_bloom_filter(
        &mut self,
        transfer_pda: &Pubkey,
        bloom_filter: &[u8; 16],
    ) -> Result<(), BanksClientError> {
        // Diese Methode ist nur für Tests und simuliert das direkte Setzen des Bloom-Filters
        // In einer realen Implementierung würde dies nicht existieren
        
        // Aktuellen Transfer-State abrufen
        let account = self.context.banks_client
            .get_account(*transfer_pda)
            .await?
            .expect("Transfer-State existiert nicht");
        
        let mut data = account.data.clone();
        
        // WARNUNG: Dies ist ein Hack für Tests - in der Realität würde man dies nie tun!
        // Wir kennen die genaue Offset-Position des Bloom-Filters im TransferState
        // und überschreiben ihn direkt (nur für Tests!)
        let bloom_offset = 180; // Beispiel-Offset für den fake_bloom im TransferState
        for i in 0..16 {
            if bloom_offset + i < data.len() {
                data[bloom_offset + i] = bloom_filter[i];
            }
        }
        
        // Modifiziertes Konto zurückschreiben
        let mut modified_account = account.clone();
        modified_account.data = data;
        
        // Konto aktualisieren
        self.context.set_account(transfer_pda, &modified_account);
        
        Ok(())
    }
}
