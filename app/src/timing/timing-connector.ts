/**
 * Timing Connector for BlackoutSOL
 * 
 * This module provides integration between the BlackoutClient and the
 * temporal obfuscation system, enabling seamless timing privacy for
 * all BlackoutSOL transfers.
 */

import { Connection, Transaction, TransactionInstruction } from '@solana/web3.js';
import { TimeObfuscator, TemporalObfuscationConfig, DEFAULT_TEMPORAL_CONFIG } from './temporal-obfuscation';

/**
 * Timing statistics for transfers
 */
export interface TimingStats {
  /** Average delay between hops in milliseconds */
  averageHopDelay: number;
  /** Time distribution across window (percentage per slice) */
  timeDistribution: number[];
  /** Entropy estimation (0-100, higher is better) */
  entropyScore: number;
  /** Correlation resistance score (0-100, higher is better) */
  correlationResistance: number;
}

/**
 * Strategy for temporal obfuscation
 */
export enum TimingStrategy {
  /** Default strategy with moderate obfuscation */
  STANDARD = 'standard',
  /** Maximum privacy with longer delays */
  MAXIMUM_PRIVACY = 'maximum_privacy',
  /** Balanced approach optimizing for both speed and privacy */
  BALANCED = 'balanced',
  /** Minimal delays for faster transfers with some timing obfuscation */
  MINIMAL = 'minimal',
  /** Custom configuration provided by user */
  CUSTOM = 'custom'
}

/**
 * Custom configuration for temporal strategies
 */
const STRATEGY_CONFIGS: Record<TimingStrategy, TemporalObfuscationConfig> = {
  [TimingStrategy.STANDARD]: DEFAULT_TEMPORAL_CONFIG,
  [TimingStrategy.MAXIMUM_PRIVACY]: {
    minDelay: 2000,      // 2s minimum
    maxDelay: 30000,     // 30s maximum
    randomBatchOrder: true,
    timeSlicedExecution: true,
    timeWindowSize: 180000, // 3 minute window
    timeSliceInterval: 5000 // 5s interval
  },
  [TimingStrategy.BALANCED]: {
    minDelay: 1000,      // 1s minimum
    maxDelay: 10000,     // 10s maximum
    randomBatchOrder: true,
    timeSlicedExecution: true,
    timeWindowSize: 60000, // 1 minute window
    timeSliceInterval: 3000 // 3s interval
  },
  [TimingStrategy.MINIMAL]: {
    minDelay: 200,       // 200ms minimum
    maxDelay: 2000,      // 2s maximum
    randomBatchOrder: true,
    timeSlicedExecution: false, // Disable time slicing for speed
    timeWindowSize: 30000, // 30s window
    timeSliceInterval: 1000 // 1s interval
  },
  [TimingStrategy.CUSTOM]: DEFAULT_TEMPORAL_CONFIG // Placeholder, will be replaced with user config
};

/**
 * Provides integration between BlackoutClient and temporal obfuscation.
 * This component enables advanced timing privacy features in BlackoutSOL
 * transfers by applying temporal obfuscation techniques.
 */
export class TimingConnector {
  private obfuscator: TimeObfuscator;
  private connection: Connection;
  private currentStrategy: TimingStrategy;
  private pendingTransactions: Map<string, number> = new Map(); // txid -> scheduled time
  
  /**
   * Create a new TimingConnector
   * 
   * @param connection Solana connection
   * @param strategy Timing strategy to use
   * @param customConfig Custom configuration (only used with CUSTOM strategy)
   */
  constructor(
    connection: Connection,
    strategy: TimingStrategy = TimingStrategy.STANDARD,
    customConfig?: TemporalObfuscationConfig
  ) {
    this.connection = connection;
    this.currentStrategy = strategy;
    
    // Select configuration based on strategy
    let config = STRATEGY_CONFIGS[strategy];
    
    // If using CUSTOM strategy, use the provided config
    if (strategy === TimingStrategy.CUSTOM && customConfig) {
      config = customConfig;
    }
    
    this.obfuscator = new TimeObfuscator(connection, config);
  }
  
  /**
   * Apply timing obfuscation to a BlackoutSOL transfer
   * 
   * @param transaction Transaction to obfuscate
   * @param hopIndex Current hop index (0-3)
   * @param txid Optional transaction ID for tracking
   * @returns Promise resolving to scheduled execution time
   */
  public async obfuscateTransfer(
    transaction: Transaction,
    hopIndex: number,
    txid?: string
  ): Promise<number> {
    const scheduledTime = await this.obfuscator.obfuscateTransfer(
      transaction, 
      hopIndex
    );
    
    // Track the transaction if ID is provided
    if (txid) {
      this.pendingTransactions.set(txid, scheduledTime);
    }
    
    return scheduledTime;
  }
  
  /**
   * Apply timing obfuscation to a multi-recipient BlackoutSOL transfer
   * 
   * @param transactions Array of transactions (one per recipient)
   * @param hopIndex Current hop index (0-3)
   * @param txids Optional array of transaction IDs for tracking
   * @returns Promise resolving to a map of time slices and scheduled times
   */
  public async obfuscateMultiTransfer(
    transactions: Transaction[],
    hopIndex: number,
    txids?: string[]
  ): Promise<Map<number, number[]>> {
    const timeSliceMap = await this.obfuscator.obfuscateMultiTransfer(
      transactions,
      hopIndex
    );
    
    // Track transactions if IDs are provided
    if (txids && txids.length === transactions.length) {
      // Flatten all scheduled times for tracking
      const allTimes: number[] = [];
      timeSliceMap.forEach(times => allTimes.push(...times));
      
      // Match each txid with a scheduled time
      txids.forEach((txid, i) => {
        if (i < allTimes.length) {
          this.pendingTransactions.set(txid, allTimes[i]);
        }
      });
    }
    
    return timeSliceMap;
  }
  
  /**
   * Get the estimated execution time for a transaction
   * 
   * @param txid Transaction ID
   * @returns Scheduled execution time or undefined if not found
   */
  public getScheduledTime(txid: string): number | undefined {
    return this.pendingTransactions.get(txid);
  }
  
  /**
   * Change the timing strategy
   * 
   * @param strategy New strategy to use
   * @param customConfig Custom configuration (only used with CUSTOM strategy)
   */
  public setStrategy(
    strategy: TimingStrategy,
    customConfig?: TemporalObfuscationConfig
  ): void {
    this.currentStrategy = strategy;
    
    // Select configuration based on strategy
    let config = STRATEGY_CONFIGS[strategy];
    
    // If using CUSTOM strategy, use the provided config
    if (strategy === TimingStrategy.CUSTOM && customConfig) {
      config = customConfig;
    }
    
    // Create a new obfuscator with the selected config
    this.obfuscator = new TimeObfuscator(this.connection, config);
  }
  
  /**
   * Get detailed timing statistics
   * 
   * @returns Timing statistics
   */
  public getTimingStats(): TimingStats {
    const diagnostics = this.obfuscator.getDiagnosticInfo();
    
    // Calculate entropy score based on time window utilization
    // More even distribution = higher entropy
    const distribution = diagnostics.timeWindowUtilization;
    const entropy = this.calculateEntropyScore(distribution);
    
    // Calculate correlation resistance based on mean delay and distribution
    const correlationResistance = this.calculateCorrelationResistance(
      diagnostics.meanDelay,
      distribution
    );
    
    return {
      averageHopDelay: diagnostics.meanDelay,
      timeDistribution: distribution,
      entropyScore: entropy,
      correlationResistance
    };
  }
  
  /**
   * Calculate entropy score from time distribution
   * 
   * Higher scores indicate more even distribution, which is better for privacy
   * as it makes timing analysis more difficult.
   * 
   * @param distribution Time window utilization percentages
   * @returns Entropy score (0-100)
   * @private
   */
  private calculateEntropyScore(distribution: number[]): number {
    if (distribution.length === 0) return 0;
    
    // Normalize distribution to sum to 1
    const sum = distribution.reduce((a, b) => a + b, 0);
    const normalized = distribution.map(val => (val / sum) || 0.0001); // Avoid log(0)
    
    // Calculate Shannon entropy
    const entropy = -normalized.reduce((acc, p) => acc + p * Math.log2(p), 0);
    
    // Max entropy would be log2(distribution.length)
    const maxEntropy = Math.log2(distribution.length);
    
    // Convert to 0-100 scale
    return Math.min(100, Math.max(0, (entropy / maxEntropy) * 100));
  }
  
  /**
   * Calculate correlation resistance score
   * 
   * Higher scores indicate better resistance to timing correlation attacks.
   * 
   * @param meanDelay Average delay in milliseconds
   * @param distribution Time window utilization
   * @returns Correlation resistance score (0-100)
   * @private
   */
  private calculateCorrelationResistance(
    meanDelay: number,
    distribution: number[]
  ): number {
    // Several factors contribute to correlation resistance:
    
    // 1. Mean delay factor (longer = better, up to a point)
    const delayFactor = Math.min(1, meanDelay / 5000) * 0.4; // 40% weight
    
    // 2. Distribution entropy (more even = better)
    const entropy = this.calculateEntropyScore(distribution) / 100;
    const entropyFactor = entropy * 0.4; // 40% weight
    
    // 3. Strategy factor (some strategies are inherently better)
    let strategyFactor = 0;
    switch (this.currentStrategy) {
      case TimingStrategy.MAXIMUM_PRIVACY:
        strategyFactor = 0.2; // 20%
        break;
      case TimingStrategy.BALANCED:
        strategyFactor = 0.15; // 15%
        break;
      case TimingStrategy.STANDARD:
        strategyFactor = 0.1; // 10%
        break;
      case TimingStrategy.MINIMAL:
        strategyFactor = 0.05; // 5%
        break;
      case TimingStrategy.CUSTOM:
        // For custom, estimate based on config (simplified)
        const config = STRATEGY_CONFIGS[this.currentStrategy];
        strategyFactor = Math.min(0.2, (config.maxDelay / 30000) * 0.2);
        break;
    }
    
    // Combine factors and convert to 0-100 scale
    const score = (delayFactor + entropyFactor + strategyFactor) * 100;
    return Math.min(100, Math.max(0, score));
  }
}
