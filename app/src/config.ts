/**
 * Konfiguration für Blackout.SOL
 */

export const config = {
  // Solana-Netzwerk
  network: {
    mainnet: 'https://api.mainnet-beta.solana.com',
    devnet: 'https://api.devnet.solana.com',
    testnet: 'https://api.testnet.solana.com',
    localnet: 'http://localhost:8899',
  },
  
  // Blackout-Programm-ID
  programId: 'B1ack0utXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX',
  
  // Protokoll-Parameter
  protocol: {
    hops: 3,
    realSplitsPerHop: 8,
    fakeSplitsPerHop: 72,
    totalSplitsPerHop: 80,
  },
  
  // Compute-Unit-Limits
  computeUnits: {
    initializeTransfer: 400_000,
    executeHop: 400_000,
    finalizeTransfer: 400_000,
    minRemainingUnits: 120_000,
  },
  
  // Proof-Parameter
  proof: {
    size: 128, // Bytes
    compressionRatio: 0.5, // 50% Kompression
  },
};