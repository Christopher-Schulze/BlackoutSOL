import {
  Connection,
  Keypair,
  PublicKey,
  Transaction,
  TransactionInstruction,
  sendAndConfirmTransaction
} from '@solana/web3.js';
import { Program, AnchorProvider, web3, BN, Idl } from '@project-serum/anchor';
import { BlackoutProofGenerator } from '../proof-generator';
import { IDL } from '../idl/blackout';
import {
  calculateEfficiency,
  getEfficiencySummary,
  getSimpleEfficiencyDisplay,
  EfficiencyResult
} from '../efficiency/cost-efficiency';

export class BlackoutClient {
  private connection: Connection;
  private wallet: Keypair;
  private program: Program;
  private proofGenerator: BlackoutProofGenerator;
  private showEfficiencyInfo: boolean = true; // Standardmäßig aktiviert
  
  constructor(connection: Connection, wallet: Keypair, programId: PublicKey) {
    this.connection = connection;
    this.wallet = wallet;
    
    // Anchor-Provider und Program initialisieren
    const provider = new AnchorProvider(
      connection,
      {
        publicKey: wallet.publicKey,
        signTransaction: async (tx: Transaction) => {
          tx.partialSign(wallet);
          return tx;
        },
        signAllTransactions: async (txs: Transaction[]) => {
          return txs.map(tx => {
            tx.partialSign(wallet);
            return tx;
          });
        }
      },
      { commitment: 'confirmed' }
    );
    
    // Typ-Assertion für IDL, um Kompatibilität mit Anchor-Programm sicherzustellen
    this.program = new Program(IDL as Idl, programId, provider);
    
    // Proof-Generator initialisieren
    this.proofGenerator = new BlackoutProofGenerator();
  }
  
  /**
   * Aktiviert oder deaktiviert die Kosteeffizienz-Informationen
   */
  setEfficiencyInfoDisplay(show: boolean): void {
    this.showEfficiencyInfo = show;
  }
  
  /**
   * Berechnet die Kosteneffizienz eines Transfers
   * @param amount Betrag in Lamports
   * @param recipientCount Anzahl der Empfänger (1-6)
   */
  getTransferEfficiency(amount: number, recipientCount: number = 1): EfficiencyResult {
    return calculateEfficiency(amount, recipientCount);
  }
  
  /**
   * Gibt eine Zusammenfassung der Kosteneffizienz zurück
   * @param amount Betrag in Lamports
   * @param recipientCount Anzahl der Empfänger (1-6)
   */
  getEfficiencySummary(amount: number, recipientCount: number = 1): string {
    return getEfficiencySummary(amount, recipientCount);
  }
  
  /**
   * Führt einen anonymen Transfer durch
   * @param amount Betrag in Lamports
   * @param recipient Empfänger-Pubkey
   * @param additionalRecipients Weitere Empfänger-Wallets für Multi-Wallet-Transfers (max. 5 zusätzliche)
   */
  async executeAnonymousTransfer(
    amount: number,
    recipient: PublicKey,
    additionalRecipients: PublicKey[] = []
  ): Promise<string> {
    // Begrenzen auf maximal 6 Empfänger (Haupt + 5 weitere)
    const validAdditionalRecipients = additionalRecipients.slice(0, 5);
    const totalRecipients = 1 + validAdditionalRecipients.length;
    
    console.log(`Starte anonymen Transfer: ${amount} Lamports an ${recipient.toBase58()}`);
    
    // Zeige Kosteneffizienz-Informationen, wenn aktiviert
    if (this.showEfficiencyInfo) {
      console.log(getSimpleEfficiencyDisplay(amount, totalRecipients));
    }
    
    // 1. Initialen Proof generieren
    console.log('Generiere initialen Proof...');
    const initialProof = await this.proofGenerator.generateInitialProof(
      BigInt(amount),
      this.wallet.publicKey,
      recipient
    );
    
    // 2. Transfer-State-PDA ableiten
    const [transferStatePda] = await PublicKey.findProgramAddress(
      [Buffer.from('transfer'), this.wallet.publicKey.toBuffer()],
      this.program.programId
    );
    
    // 3. Transfer initialisieren
    console.log('Initialisiere Transfer...');
    const initTx = await this.program.methods
      .initializeTransfer(new BN(amount), initialProof)
      .accounts({
        payer: this.wallet.publicKey,
        transferState: transferStatePda,
        systemProgram: web3.SystemProgram.programId,
      })
      .transaction();
    
    const initSignature = await sendAndConfirmTransaction(
      this.connection,
      initTx,
      [this.wallet]
    );
    console.log(`Transfer initialisiert: ${initSignature}`);
    
    // 4. Transfer-State abrufen
    const transferState = await this.program.account.transferState.fetch(transferStatePda);
    
    // Typkorrektur für den Seed (von unknown zu ArrayBuffer oder Array<number>)
    const seedData = transferState.seed as ArrayBuffer;
    const seed = new Uint8Array(seedData);
    
    // 5. Drei Hops ausführen
    const hopSeeds: Uint8Array[] = [];
    for (let hopIndex = 0; hopIndex < 3; hopIndex++) {
      console.log(`Führe Hop ${hopIndex} aus...`);
      
      // Echte Splits generieren (8 Stück)
      const realSplits = this.generateRealSplits(amount, 8);
      
      // Hop-Proof generieren
      const hopProof = await this.proofGenerator.generateHopProof(
        hopIndex,
        seed,
        realSplits.map(s => BigInt(s))
      );
      
      // Hop-Seed für finalen Proof speichern
      hopSeeds.push(seed);
      
      // Hop ausführen
      const hopTx = await this.program.methods
        .executeHop(hopIndex, hopProof)
        .accounts({
          authority: this.wallet.publicKey,
          transferState: transferStatePda,
          systemProgram: web3.SystemProgram.programId,
        })
        .transaction();
      
      const hopSignature = await sendAndConfirmTransaction(
        this.connection,
        hopTx,
        [this.wallet]
      );
      console.log(`Hop ${hopIndex} abgeschlossen: ${hopSignature}`);
    }
    
    // 6. Finalen Proof generieren
    console.log('Generiere finalen Proof...');
    const finalProof = await this.proofGenerator.generateFinalProof(
      BigInt(amount),
      this.wallet.publicKey,
      recipient,
      hopSeeds
    );
    
    // 7. Transfer finalisieren
    console.log('Finalisiere Transfer...');
    const finalizeTx = await this.program.methods
      .finalizeTransfer(finalProof)
      .accounts({
        authority: this.wallet.publicKey,
        transferState: transferStatePda,
        recipient: recipient,
        systemProgram: web3.SystemProgram.programId,
      })
      .transaction();
    
    const finalizeSignature = await sendAndConfirmTransaction(
      this.connection,
      finalizeTx,
      [this.wallet]
    );
    console.log(`Transfer finalisiert: ${finalizeSignature}`);
    
    // Zeige detaillierte Effizienz-Zusammenfassung nach Transfer, wenn aktiviert
    if (this.showEfficiencyInfo) {
      console.log(this.getEfficiencySummary(amount, 1 + (additionalRecipients?.length || 0)));
    }
    
    return finalizeSignature;
  }
  
  /**
   * Generiert echte Splits für einen Hop
   */
  private generateRealSplits(totalAmount: number, count: number): number[] {
    const splits: number[] = [];
    let remaining = totalAmount;
    
    // Erste (count-1) Splits zufällig generieren
    for (let i = 0; i < count - 1; i++) {
      // Zufälligen Anteil des verbleibenden Betrags nehmen
      const splitAmount = Math.floor(remaining * Math.random() * 0.5);
      splits.push(splitAmount);
      remaining -= splitAmount;
    }
    
    // Letzten Split mit dem Rest füllen
    splits.push(remaining);
    
    return splits;
  }
}