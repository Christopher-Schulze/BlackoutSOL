/**
 * Temporal Obfuscation Module for BlackoutSOL
 * 
 * This module provides time-based obfuscation techniques to prevent
 * timing correlation attacks in privacy-enhanced transfers. By introducing
 * controlled randomness in transaction timing, this further enhances
 * the anonymity guarantees of BlackoutSOL's multi-hop architecture.
 */

import { PublicKey, Connection, TransactionInstruction, Transaction } from '@solana/web3.js';

/**
 * Configuration for temporal obfuscation
 */
export interface TemporalObfuscationConfig {
  /** Minimum delay in milliseconds */
  minDelay: number;
  /** Maximum delay in milliseconds */
  maxDelay: number;
  /** Whether to enable random batch ordering */
  randomBatchOrder: boolean;
  /** Whether to use time-sliced execution */
  timeSlicedExecution: boolean;
  /** Time window size for time-sliced execution (in milliseconds) */
  timeWindowSize: number;
  /** Interval to use for time-slicing (in milliseconds) */
  timeSliceInterval: number;
}

/**
 * Default configuration for temporal obfuscation
 */
export const DEFAULT_TEMPORAL_CONFIG: TemporalObfuscationConfig = {
  minDelay: 500,      // 500ms minimum delay
  maxDelay: 8000,     // 8000ms maximum delay
  randomBatchOrder: true,
  timeSlicedExecution: true,
  timeWindowSize: 60000, // 60s window
  timeSliceInterval: 2000 // 2s interval
};

/**
 * Scheduled transaction for time-sliced execution
 */
interface ScheduledTransaction {
  transaction: Transaction;
  executionTime: number;
  priority: number;
}

/**
 * Records of transaction execution times for pattern analysis
 */
interface ExecutionTimeRecord {
  timestamp: number;
  transactionId: string;
  hopIndex: number;
  executionDuration: number;
}

/**
 * Temporal obfuscation manager for BlackoutSOL transactions
 */
export class TemporalObfuscationManager {
  private config: TemporalObfuscationConfig;
  private scheduledTransactions: ScheduledTransaction[] = [];
  private executionTimeRecords: ExecutionTimeRecord[] = [];
  private connection: Connection;
  private isProcessing: boolean = false;
  private processingInterval: NodeJS.Timeout | null = null;
  
  /**
   * Create a new TemporalObfuscationManager
   * 
   * @param connection Solana connection
   * @param config Temporal obfuscation configuration
   */
  constructor(
    connection: Connection,
    config: TemporalObfuscationConfig = DEFAULT_TEMPORAL_CONFIG
  ) {
    this.connection = connection;
    this.config = config;
  }
  
  /**
   * Start processing scheduled transactions
   */
  public startProcessing(): void {
    if (this.isProcessing) return;
    
    this.isProcessing = true;
    this.processingInterval = setInterval(
      () => this.processScheduledTransactions(),
      500 // Check for scheduled transactions every 500ms
    );
  }
  
  /**
   * Stop processing scheduled transactions
   */
  public stopProcessing(): void {
    this.isProcessing = false;
    if (this.processingInterval) {
      clearInterval(this.processingInterval);
      this.processingInterval = null;
    }
  }
  
  /**
   * Schedule a transaction for delayed execution with time-based privacy
   * 
   * @param transaction Transaction to schedule
   * @param hopIndex Current hop index (0-3) for this transaction
   * @param priority Priority of the transaction (higher = more important)
   * @returns Scheduled execution time in milliseconds since epoch
   */
  public scheduleTransaction(
    transaction: Transaction,
    hopIndex: number,
    priority: number = 1
  ): number {
    // Generate a random delay within the configured range
    const delay = this.generateRandomDelay(hopIndex);
    
    // Calculate execution time
    const executionTime = Date.now() + delay;
    
    // Add to scheduled transactions
    this.scheduledTransactions.push({
      transaction,
      executionTime,
      priority
    });
    
    // Sort by execution time and priority
    this.scheduledTransactions.sort((a, b) => {
      // Sort by execution time first
      if (a.executionTime !== b.executionTime) {
        return a.executionTime - b.executionTime;
      }
      // Then by priority (higher priority first)
      return b.priority - a.priority;
    });
    
    // If not already processing, start
    if (!this.isProcessing) {
      this.startProcessing();
    }
    
    return executionTime;
  }
  
  /**
   * Schedule multiple transactions with randomized order for enhanced privacy
   * 
   * This method introduces randomness in transaction ordering to prevent
   * correlation of related transactions that would otherwise be executed
   * sequentially.
   * 
   * @param transactions Array of transactions to schedule
   * @param hopIndex Current hop index for these transactions
   * @returns Array of scheduled execution times
   */
  public scheduleBatchWithRandomOrder(
    transactions: Transaction[],
    hopIndex: number
  ): number[] {
    if (!this.config.randomBatchOrder || transactions.length <= 1) {
      // If random batch ordering is disabled or unnecessary, schedule sequentially
      return transactions.map(tx => this.scheduleTransaction(tx, hopIndex));
    }
    
    // Create a shuffled copy of the transactions array
    const shuffled = [...transactions].sort(() => Math.random() - 0.5);
    
    // Schedule each transaction in the shuffled order
    return shuffled.map(tx => this.scheduleTransaction(tx, hopIndex));
  }
  
  /**
   * Schedule transactions across time slices for maximum timing obfuscation
   * 
   * This method distributes transactions across specified time slices within
   * a configurable time window. This prevents transaction bunching that could
   * reveal multi-recipient transfers.
   * 
   * @param transactions Array of transactions to schedule
   * @param hopIndex Current hop index for these transactions
   * @returns Map of time slices to scheduled transactions
   */
  public scheduleTimeSlicedBatch(
    transactions: Transaction[],
    hopIndex: number
  ): Map<number, number[]> {
    if (!this.config.timeSlicedExecution || transactions.length <= 1) {
      // If time-sliced execution is disabled or unnecessary, schedule in batch
      const times = this.scheduleBatchWithRandomOrder(transactions, hopIndex);
      return new Map([[0, times]]);
    }
    
    // Calculate how many time slices are available in the configured window
    const numSlices = Math.floor(this.config.timeWindowSize / this.config.timeSliceInterval);
    if (numSlices <= 0) {
      throw new Error('Invalid time window configuration: window size must be greater than slice interval');
    }
    
    // Create map to track scheduled times per slice
    const timeSliceMap = new Map<number, number[]>();
    
    // Shuffle transactions for additional randomness
    const shuffled = [...transactions].sort(() => Math.random() - 0.5);
    
    // Distribute transactions across time slices
    shuffled.forEach((tx, index) => {
      // Assign to a random time slice
      const timeSlice = Math.floor(Math.random() * numSlices);
      
      // Calculate base delay for this time slice
      const baseDelay = timeSlice * this.config.timeSliceInterval;
      
      // Add small random variation within the slice
      const sliceOffset = Math.floor(Math.random() * (this.config.timeSliceInterval * 0.8));
      const totalDelay = baseDelay + sliceOffset;
      
      // Calculate execution time
      const executionTime = Date.now() + totalDelay;
      
      // Schedule with custom execution time
      this.scheduledTransactions.push({
        transaction: tx,
        executionTime,
        priority: 1
      });
      
      // Update the map
      if (!timeSliceMap.has(timeSlice)) {
        timeSliceMap.set(timeSlice, []);
      }
      timeSliceMap.get(timeSlice)!.push(executionTime);
    });
    
    // Sort scheduled transactions
    this.scheduledTransactions.sort((a, b) => a.executionTime - b.executionTime);
    
    // If not already processing, start
    if (!this.isProcessing && this.scheduledTransactions.length > 0) {
      this.startProcessing();
    }
    
    return timeSliceMap;
  }
  
  /**
   * Add entropy to transaction execution times for transfer hop
   * 
   * This method modifies the current hop delay based on previous hop
   * execution times, adding variability that makes transaction correlation
   * more difficult.
   * 
   * @param baseDelay Base delay value in milliseconds
   * @param hopIndex Current hop index (0-3)
   * @returns Adjusted delay with added entropy
   */
  public addTemporalEntropy(baseDelay: number, hopIndex: number): number {
    // Skip for the first hop (no previous hops to analyze)
    if (hopIndex === 0) {
      return baseDelay;
    }
    
    // Get execution time records for previous hops
    const previousHopRecords = this.executionTimeRecords.filter(
      r => r.hopIndex === hopIndex - 1
    );
    
    if (previousHopRecords.length === 0) {
      return baseDelay;
    }
    
    // Calculate mean and standard deviation of previous execution times
    const executionTimes = previousHopRecords.map(r => r.executionDuration);
    const mean = executionTimes.reduce((sum, time) => sum + time, 0) / executionTimes.length;
    
    // Add variability based on previous execution patterns
    // This makes it harder to correlate hops based on timing patterns
    const variabilityFactor = Math.random() * 0.4 + 0.8; // 0.8 - 1.2
    
    return Math.floor(baseDelay * variabilityFactor + mean * 0.25);
  }
  
  /**
   * Get diagnostic information about temporal obfuscation
   * 
   * @returns Diagnostic information about scheduled transactions and timing patterns
   */
  public getDiagnosticInfo(): {
    scheduled: number,
    executed: number,
    meanDelay: number,
    timeWindowUtilization: number[]
  } {
    // Calculate mean delay from execution records
    const delays = this.executionTimeRecords.map(r => r.executionDuration);
    const meanDelay = delays.length > 0 ? 
      delays.reduce((sum, delay) => sum + delay, 0) / delays.length : 0;
    
    // Calculate time window utilization (percentage of slots used)
    const numSlices = Math.floor(this.config.timeWindowSize / this.config.timeSliceInterval);
    const timeWindowUtilization = new Array(numSlices).fill(0);
    
    this.executionTimeRecords.forEach(record => {
      const relativeTime = record.timestamp % this.config.timeWindowSize;
      const sliceIndex = Math.floor(relativeTime / this.config.timeSliceInterval);
      if (sliceIndex >= 0 && sliceIndex < numSlices) {
        timeWindowUtilization[sliceIndex]++;
      }
    });
    
    // Normalize to percentages
    const maxCount = Math.max(...timeWindowUtilization, 1);
    const normalizedUtilization = timeWindowUtilization.map(count => 
      Math.floor((count / maxCount) * 100)
    );
    
    return {
      scheduled: this.scheduledTransactions.length,
      executed: this.executionTimeRecords.length,
      meanDelay,
      timeWindowUtilization: normalizedUtilization
    };
  }
  
  /**
   * Process scheduled transactions that are ready for execution
   * @private
   */
  private async processScheduledTransactions(): Promise<void> {
    const now = Date.now();
    
    // Extract transactions that are ready to execute
    const readyTransactions: ScheduledTransaction[] = [];
    const pendingTransactions: ScheduledTransaction[] = [];
    
    this.scheduledTransactions.forEach(scheduled => {
      if (scheduled.executionTime <= now) {
        readyTransactions.push(scheduled);
      } else {
        pendingTransactions.push(scheduled);
      }
    });
    
    // Update the scheduled transactions list
    this.scheduledTransactions = pendingTransactions;
    
    // Execute ready transactions
    for (const scheduled of readyTransactions) {
      try {
        const startTime = Date.now();
        // Die korrekte Signatur für sendTransaction mit Transaction-Typ verwenden
        const signature = await this.connection.sendTransaction(
          scheduled.transaction,
          [] // leeres Signers-Array, da die Transaktion bereits signiert sein sollte
        );
        
        const endTime = Date.now();
        
        // Record execution time
        this.executionTimeRecords.push({
          timestamp: startTime,
          transactionId: signature,
          hopIndex: -1, // Placeholder, would be set in actual implementation
          executionDuration: endTime - startTime
        });
        
        // Limit the size of execution records to prevent memory growth
        if (this.executionTimeRecords.length > 1000) {
          this.executionTimeRecords = this.executionTimeRecords.slice(-1000);
        }
      } catch (error) {
        console.error('Error executing scheduled transaction:', error);
      }
    }
    
    // If no more transactions and not stopped externally, stop processing
    if (this.scheduledTransactions.length === 0 && this.isProcessing) {
      this.stopProcessing();
    }
  }
  
  /**
   * Generate a random delay with hop-specific characteristics
   * 
   * Each hop has slightly different delay characteristics to prevent
   * correlation of transactions across hops.
   * 
   * @param hopIndex Current hop index (0-3)
   * @returns Random delay in milliseconds
   * @private
   */
  private generateRandomDelay(hopIndex: number): number {
    // Base delay range from config
    let minDelay = this.config.minDelay;
    let maxDelay = this.config.maxDelay;
    
    // Modify delay range based on hop index to add additional entropy
    switch (hopIndex) {
      case 0: // First hop: relatively quick
        maxDelay = Math.min(maxDelay, 3000);
        break;
      case 1: // Second hop: moderate delay
        minDelay = Math.max(minDelay, 1000);
        maxDelay = Math.min(maxDelay, 5000);
        break;
      case 2: // Third hop: variable delay
        minDelay = Math.max(minDelay, 1500);
        break;
      case 3: // Fourth hop: potentially longer delay
        minDelay = Math.max(minDelay, 2000);
        break;
      default:
        // Use default range
        break;
    }
    
    // Generate random delay within the adjusted range
    const baseDelay = Math.floor(Math.random() * (maxDelay - minDelay + 1)) + minDelay;
    
    // Add entropy based on previous execution patterns
    return this.addTemporalEntropy(baseDelay, hopIndex);
  }
}

/**
 * The TimeObfuscator class provides a simpler interface to the TemporalObfuscationManager
 * for use in BlackoutSOL transfers.
 */
export class TimeObfuscator {
  private manager: TemporalObfuscationManager;
  
  /**
   * Create a new TimeObfuscator
   * 
   * @param connection Solana connection
   * @param config Temporal obfuscation configuration
   */
  constructor(
    connection: Connection,
    config?: TemporalObfuscationConfig
  ) {
    this.manager = new TemporalObfuscationManager(connection, config);
  }
  
  /**
   * Obfuscate the timing of a BlackoutSOL transfer
   * 
   * This method applies temporal obfuscation to the specified transaction
   * using the configured time-based privacy enhancements.
   * 
   * @param transaction Transaction to obfuscate
   * @param hopIndex Current hop index (0-3)
   * @returns Promise resolving to the scheduled execution time
   */
  public async obfuscateTransfer(
    transaction: Transaction,
    hopIndex: number
  ): Promise<number> {
    return this.manager.scheduleTransaction(transaction, hopIndex);
  }
  
  /**
   * Obfuscate the timing of a multi-recipient BlackoutSOL transfer
   * 
   * This method applies advanced temporal obfuscation to multi-recipient
   * transfers, distributing them across time slices for maximum privacy.
   * 
   * @param transactions Transactions to obfuscate (one per recipient)
   * @param hopIndex Current hop index (0-3)
   * @returns Promise resolving to a map of time slices and execution times
   */
  public async obfuscateMultiTransfer(
    transactions: Transaction[],
    hopIndex: number
  ): Promise<Map<number, number[]>> {
    return this.manager.scheduleTimeSlicedBatch(transactions, hopIndex);
  }
  
  /**
   * Get diagnostic information about temporal obfuscation
   * 
   * @returns Diagnostic information
   */
  public getDiagnosticInfo() {
    return this.manager.getDiagnosticInfo();
  }
  
  /**
   * Clean up all resources and stop processing
   * 
   * This method should be called to properly clean up resources
   * and prevent memory leaks, especially in testing environments.
   */
  public cleanup(): void {
    if (this.manager) {
      this.manager.stopProcessing();
    }
  }
}
