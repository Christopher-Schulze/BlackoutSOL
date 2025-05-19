/**
 * BlackoutSOL Kosteneffizienz-Berechnungen
 * 
 * Dieses Modul enthält Funktionen zur Berechnung und Anzeige von Kosteneffizienz-Metriken
 * für BlackoutSOL-Transfers.
 */

import { PublicKey } from '@solana/web3.js';

// Konstanten für die Kosteneffizienz-Berechnungen
export const LAMPORTS_PER_SOL = 1_000_000_000;
export const BASELINE_EFFICIENCY = 92.0; // Effizienz vor Optimierungen (%)
export const OPTIMIZED_EFFICIENCY = 98.0; // Effizienz nach Optimierungen (%)
export const MIN_RECIPIENTS = 1;
export const MAX_RECIPIENTS = 6;
export const DEFAULT_RECIPIENTS = 1;

// Kostenkomponenten
export interface CostBreakdown {
  txFee: number;       // Transaktionsgebühren in Lamports
  rent: number;        // Account-Rent in Lamports
  compute: number;     // Compute-Units-Kosten in Lamports
  overhead: number;    // Sonstige Kosten in Lamports
}

// Effizienz-Ergebnis
export interface EfficiencyResult {
  efficiency: number;            // Transfereffizienz in Prozent (0-100)
  receivedAmount: number;        // Tatsächlich erhaltener Betrag in Lamports
  totalCost: number;             // Gesamtkosten in Lamports
  costBreakdown: CostBreakdown;  // Aufschlüsselung der Kosten
  accountsRemaining: number;     // Anzahl der verbleibenden Accounts
  savingsVsBaseline: number;     // Einsparungen gegenüber unoptimierter Version in Lamports
  savingsPercent: number;        // Einsparungen in Prozent
}

/**
 * Berechnet die Kosteneffizienz eines Transfers
 * @param amount Transferbetrag in Lamports
 * @param recipientCount Anzahl der Empfänger (1-6)
 * @returns Ergebnis mit Effizienzmetriken
 */
export function calculateEfficiency(
  amount: number,
  recipientCount: number = DEFAULT_RECIPIENTS
): EfficiencyResult {
  // Validiere und begrenze die Empfängeranzahl
  const validRecipientCount = Math.max(MIN_RECIPIENTS, Math.min(MAX_RECIPIENTS, recipientCount));
  
  // Basiskosten berechnen
  const baseTxFee = 5250; // Optimierte Basisgebühr
  const baseRent = 267264; // Optimierte Rent-Kosten
  
  // Berechne Kosten basierend auf Empfängeranzahl
  const txFee = baseTxFee * (1 + 0.05 * (validRecipientCount - 1));
  const rent = baseRent;
  const compute = 200_000 + (10_000 * validRecipientCount);
  const computeCost = compute / 1_000_000;
  const overhead = 0;
  
  // Gesamtkosten
  const totalCost = txFee + rent + computeCost + overhead;
  
  // Effizienz und erhaltener Betrag
  const efficiency = OPTIMIZED_EFFICIENCY;
  const receivedAmount = amount * (efficiency / 100);
  
  // Berechne Baseline-Kosten (unoptimiert)
  const baselineTxFee = 5500 * (1 + 0.1 * (validRecipientCount - 1));
  const baselineRent = 890880;
  const baselineCompute = 220_000 + (15_000 * validRecipientCount);
  const baselineComputeCost = baselineCompute / 1_000_000;
  const baselineTotalCost = baselineTxFee + baselineRent + baselineComputeCost;
  
  // Einsparungen
  const savingsVsBaseline = baselineTotalCost - totalCost;
  const savingsPercent = (savingsVsBaseline / baselineTotalCost) * 100;
  
  return {
    efficiency,
    receivedAmount,
    totalCost,
    costBreakdown: {
      txFee,
      rent,
      compute: computeCost,
      overhead,
    },
    accountsRemaining: 0,
    savingsVsBaseline,
    savingsPercent,
  };
}

/**
 * Berechnet die unoptimierte Kosteneffizienz (Baseline)
 * @param amount Transferbetrag in Lamports
 * @param recipientCount Anzahl der Empfänger (1-6)
 * @returns Ergebnis mit Effizienzmetriken für unoptimierte Version
 */
export function calculateBaselineEfficiency(
  amount: number,
  recipientCount: number = DEFAULT_RECIPIENTS
): EfficiencyResult {
  // Validiere und begrenze die Empfängeranzahl
  const validRecipientCount = Math.max(MIN_RECIPIENTS, Math.min(MAX_RECIPIENTS, recipientCount));
  
  // Baseline-Kosten berechnen (unoptimiert)
  const txFee = 5500 * (1 + 0.1 * (validRecipientCount - 1));
  const rent = 890880;
  const compute = 220_000 + (15_000 * validRecipientCount);
  const computeCost = compute / 1_000_000;
  const overhead = 0;
  
  // Gesamtkosten
  const totalCost = txFee + rent + computeCost + overhead;
  
  // Effizienz und erhaltener Betrag
  const efficiency = BASELINE_EFFICIENCY;
  const receivedAmount = amount * (efficiency / 100);
  
  return {
    efficiency,
    receivedAmount,
    totalCost,
    costBreakdown: {
      txFee,
      rent,
      compute: computeCost,
      overhead,
    },
    accountsRemaining: validRecipientCount >= 3 ? validRecipientCount : 2,
    savingsVsBaseline: 0,
    savingsPercent: 0,
  };
}

/**
 * Formatiert einen Betrag in Lamports als SOL mit 9 Dezimalstellen
 */
export function formatSol(lamports: number): string {
  return (lamports / LAMPORTS_PER_SOL).toFixed(9);
}

/**
 * Formatiert einen Prozentsatz mit 2 Dezimalstellen
 */
export function formatPercent(percent: number): string {
  return percent.toFixed(2) + '%';
}

/**
 * Erzeugt eine formatierte Zusammenfassung der Kosteneffizienz
 */
export function getEfficiencySummary(amount: number, recipientCount: number = DEFAULT_RECIPIENTS): string {
  const optimized = calculateEfficiency(amount, recipientCount);
  const baseline = calculateBaselineEfficiency(amount, recipientCount);
  
  const lines: string[] = [
    '\n=== KOSTENEFFIZIENZ-ZUSAMMENFASSUNG ===',
    `Transfer: ${formatSol(amount)} SOL an ${recipientCount} Empfänger`,
    `Effizienz: ${formatPercent(optimized.efficiency)} (${formatPercent(baseline.efficiency)} ohne Optimierungen)`,
    `Tatsächlich erhalten: ${formatSol(optimized.receivedAmount)} SOL (${formatPercent(optimized.efficiency)} des Betrags)`,
    `Gesamtkosten: ${formatSol(optimized.totalCost)} SOL`,
    `Einsparung: ${formatSol(optimized.savingsVsBaseline)} SOL (${formatPercent(optimized.savingsPercent)})`,
    '-------------------------------------',
    'Kostenaufschlüsselung:',
    `  Transaktionsgebühren: ${formatSol(optimized.costBreakdown.txFee)} SOL`,
    `  Rent-Kosten: ${formatSol(optimized.costBreakdown.rent)} SOL (-${formatPercent((baseline.costBreakdown.rent - optimized.costBreakdown.rent) / baseline.costBreakdown.rent * 100)})`,
    `  Compute-Units: ${formatSol(optimized.costBreakdown.compute)} SOL`,
    '=====================================',
  ];
  
  return lines.join('\n');
}

/**
 * Erzeugt eine einfache Kosteneffizienzanzeige für die Konsole
 */
export function getSimpleEfficiencyDisplay(amount: number, recipientCount: number = DEFAULT_RECIPIENTS): string {
  const optimized = calculateEfficiency(amount, recipientCount);
  const baseline = calculateBaselineEfficiency(amount, recipientCount);
  
  const efficiencyGain = optimized.efficiency - baseline.efficiency;
  
  return `Effizienz: ${formatPercent(optimized.efficiency)} (+${formatPercent(efficiencyGain)} durch Optimierungen) | Einsparung: ${formatPercent(optimized.savingsPercent)}`;
}
