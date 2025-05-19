/**
 * BlackoutSOL Terminal Dashboard for Cost Efficiency
 * 
 * This module provides a simple, visual representation of cost efficiency metrics
 * in the terminal without external dependencies.
 */

import { EfficiencyResult, calculateEfficiency, calculateBaselineEfficiency } from './cost-efficiency';

// ASCII bars for percentage visualizations
const ASCII_BAR_FULL = '█';
const ASCII_BAR_EMPTY = '░';

/**
 * Creates an ASCII progress bar
 * @param percent Percentage (0-100)
 * @param length Length of the bar
 */
function createProgressBar(percent: number, length: number = 20): string {
  const filledLength = Math.floor(length * (percent / 100));
  const emptyLength = length - filledLength;
  
  return ASCII_BAR_FULL.repeat(filledLength) + ASCII_BAR_EMPTY.repeat(emptyLength);
}

/**
 * Formats an amount in lamports for display
 * @param lamports Amount in lamports
 */
function formatAmount(lamports: number): string {
  const SOL = lamports / 1_000_000_000;
  
  if (SOL >= 0.01) {
    return `${SOL.toFixed(SOL < 0.1 ? 3 : 2)} SOL`;
  } else {
    return `${lamports.toLocaleString()} Lamports`;
  }
}

/**
 * Renders a terminal dashboard for cost efficiency
 * @param amount Transfer amount in lamports
 * @param recipientCount Number of recipients (1-6)
 */
export function renderEfficiencyDashboard(amount: number, recipientCount: number = 1): string {
  const optimized = calculateEfficiency(amount, recipientCount);
  const baseline = calculateBaselineEfficiency(amount, recipientCount);
  
  // Terminal width
  const width = 70;
  const divider = '─'.repeat(width);
  const padding = ' '.repeat(2);
  
  // Dashboard header
  let output = '\n';
  output += `┌${'─'.repeat(width - 2)}┐\n`;
  output += `│${' '.repeat((width - 26) / 2)}BLACKOUT KOSTENEFFIZIENZ${' '.repeat((width - 26) / 2)}│\n`;
  output += `├${divider}┤\n`;
  
  // Transfer information
  output += `│${padding}Transfer: ${formatAmount(amount)} with ${recipientCount} recipient${recipientCount > 1 ? 's' : ' '}${' '.repeat(width - 40 - recipientCount.toString().length)}│\n`;
  output += `├${divider}┤\n`;
  
  // Efficiency comparison
  const efficiencyDiff = optimized.efficiency - baseline.efficiency;
  output += `│${padding}Transfer efficiency:${' '.repeat(10)}${optimized.efficiency.toFixed(1)}%${' '.repeat(width - 36)}│\n`;
  output += `│${padding}${createProgressBar(optimized.efficiency)}  vs  ${createProgressBar(baseline.efficiency)}${' '.repeat(width - 53)}│\n`;
  output += `│${padding}Improvement: +${efficiencyDiff.toFixed(1)} percentage points${' '.repeat(width - 39 - efficiencyDiff.toString().length)}│\n`;
  output += `├${divider}┤\n`;
  
  // Cost comparison
  const savingsPercent = optimized.savingsPercent;
  output += `│${padding}Total costs:${' '.repeat(15)}${formatAmount(optimized.totalCost)}${' '.repeat(width - 39 - optimized.totalCost.toString().length)}│\n`;
  output += `│${padding}Savings: ${formatAmount(optimized.savingsVsBaseline)} (${savingsPercent.toFixed(1)}%)${' '.repeat(width - 40 - optimized.savingsVsBaseline.toString().length - savingsPercent.toString().length)}│\n`;
  output += `├${divider}┤\n`;
  
  // Cost breakdown
  output += `│${padding}Cost breakdown:${' '.repeat(width - 25)}│\n`;
  output += `│${padding}├─ Tx fees: ${formatAmount(optimized.costBreakdown.txFee)}${' '.repeat(width - 35 - optimized.costBreakdown.txFee.toString().length)}│\n`;
  
  // Rent costs with improvement
  const rentReduction = ((baseline.costBreakdown.rent - optimized.costBreakdown.rent) / baseline.costBreakdown.rent) * 100;
  output += `│${padding}├─ Rent costs: ${formatAmount(optimized.costBreakdown.rent)} (-${rentReduction.toFixed(1)}%)${' '.repeat(width - 44 - optimized.costBreakdown.rent.toString().length - rentReduction.toString().length)}│\n`;
  
  // Compute units
  output += `│${padding}└─ Compute:    ${formatAmount(optimized.costBreakdown.compute)}${' '.repeat(width - 35 - optimized.costBreakdown.compute.toString().length)}│\n`;
  output += `└${'─'.repeat(width - 2)}┘\n`;
  
  // Tips
  output += `\n💡 TIP: Use multi-wallet transfers to increase your anonymity.`;
  if (recipientCount === 1) {
    output += `\n   The optimized implementation results in only minimal additional costs.`;
    output += `\n   Use --multi=addr1,addr2,... for multi-wallet transfers.`;
  }
  
  return output;
}

/**
 * Displays the terminal dashboard for cost efficiency
 * @param amount Transfer amount in lamports
 * @param recipientCount Number of recipients
 */
export function displayEfficiencyDashboard(amount: number, recipientCount: number = 1): void {
  console.log(renderEfficiencyDashboard(amount, recipientCount));
}
