# BlackoutSOL Client Library and DApp Connector

This directory contains the TypeScript client library and DApp connector for BlackoutSOL - a privacy-enhanced transaction protocol for Solana.

## Overview

BlackoutSOL provides anonymous transfers on Solana with:
- **Multi-Hop Architecture**: 4 sequential hops that break transaction traceability
- **Split Mechanism**: Each hop splits funds into 4 real transfers + 44 fake transfers
- **Zero-Knowledge Proofs**: For integrity verification without revealing sensitive data
- **Cost Efficiency**: 98% efficiency (only 2% overhead)

## Installation

```bash
npm install @blackoutsol/client
```

## DApp Connector Usage

The DApp connector provides a clean API for web applications to integrate with BlackoutSOL:

```typescript
import { BlackoutDAppConnector, DAppConfig } from '@blackoutsol/client';

// 1. Initialize connector
const config: DAppConfig = {
  rpcUrl: 'https://api.mainnet-beta.solana.com',
  commitment: 'confirmed',
  useDevnet: false // Set to true for devnet
};

const connector = new BlackoutDAppConnector(config);
await connector.initialize();

// 2. Execute anonymous transfer
const wallet = /* Get user wallet */;
const response = await connector.executeTransfer({
  amount: 1_500_000_000, // 1.5 SOL in lamports
  recipients: [
    'GsbwXfJraMomkTbU3KjALchLz1UyjjSJcST5zrTQ1Do9',
    'HXk3B5mGNHXDKU9F6RLuNVzUGCc1YP4uwupcFMUe3Qid'
  ],
  showEfficiency: true,
  payerKeypair: wallet.keypair
});

// 3. Handle response
if (response.success) {
  console.log(`Transfer successful: ${response.signature}`);
  console.log(`Efficiency: ${response.efficiency?.efficiency}%`);
} else {
  console.error(`Transfer failed: ${response.error}`);
}

// 4. Calculate efficiency before transfer (optional)
const efficiency = connector.calculateTransferEfficiency(
  1_500_000_000, // amount in lamports
  2              // number of recipients
);

console.log(`Transfer efficiency: ${efficiency.efficiency}%`);
console.log(`Total cost: ${efficiency.totalCost} lamports`);

// 5. Compare with baseline implementation (optional)
const comparison = connector.compareEfficiency(1_500_000_000, 2);
console.log(`Improvement: ${comparison.improvementPercent.toFixed(1)}%`);

// 6. Get anonymity set size (optional)
const anonymitySetSize = connector.getAnonymitySetSize();
console.log(`Anonymity set size: ${anonymitySetSize.toLocaleString()}`);
```

## Privacy Guarantees

BlackoutSOL provides:

- **Sender Privacy**: Original sender cannot be linked to final recipients
- **Recipient Privacy**: Final recipients cannot be linked to original sender
- **Amount Privacy**: Transaction amounts are hidden through ZKPs
- **Transaction Graph Privacy**: Multi-hop architecture breaks transaction graph analysis

## Cost Efficiency

The optimized implementation achieves:

- **98% Efficiency**: Only 2% of the transfer amount is spent on transaction costs
- **70% Rent Reduction**: Compared to baseline implementation
- **43.4% Total Savings**: Across all cost components

## Directory Structure

- `/src/client/` - Core client implementation
- `/src/efficiency/` - Cost efficiency calculation and display
- `/src/connector/` - DApp connector for external integration
- `/src/proof-generator/` - Zero-knowledge proof generation

## For Developers

### Building the Library

```bash
npm run build
```

### Running Tests

```bash
npm test
```

### TypeScript Definitions

Complete type definitions are included for all components:

```typescript
// Main interfaces
interface TransferRequest {
  amount: number;
  recipients: string[];
  showEfficiency?: boolean;
  payerKeypair: Keypair;
}

interface TransferResponse {
  success: boolean;
  signature?: string;
  error?: string;
  efficiency?: EfficiencyResult;
  blockTime?: number;
  slot?: number;
}

interface EfficiencyResult {
  efficiency: number;
  totalCost: number;
  receivedAmount: number;
  costBreakdown: CostBreakdown;
  savingsVsBaseline: number;
  savingsPercent: number;
}
```

## Learn More

For detailed technical information, refer to the [Technical Specification](../TECHNICAL_SPECIFICATION.md) document.
