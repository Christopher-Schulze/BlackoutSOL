/**
 * Tests for the Temporal Obfuscation module for BlackoutSOL
 * 
 * These tests verify that the temporal obfuscation components function
 * correctly and provide the expected privacy guarantees for BlackoutSOL
 * transfers.
 */

import { Connection, Keypair, PublicKey, Transaction } from '@solana/web3.js';
import { trackObfuscator, cleanupResources } from './utils/test-cleanup';
import { 
  TimeObfuscator, 
  TemporalObfuscationManager, 
  DEFAULT_TEMPORAL_CONFIG 
} from '../src/timing/temporal-obfuscation';
import { 
  TimingConnector, 
  TimingStrategy 
} from '../src/timing/timing-connector';
import { 
  TimingEnhancedConnector, 
  TimingEnhancedTransferRequest 
} from '../src/connector/timing-enhanced-connector';

// Mock Connection to avoid real network calls
jest.mock('@solana/web3.js', () => {
  const original = jest.requireActual('@solana/web3.js');
  return {
    ...original,
    Connection: jest.fn().mockImplementation(() => {
      return {
        sendTransaction: jest.fn().mockResolvedValue('mock-signature'),
        getLatestBlockhash: jest.fn().mockResolvedValue({
          blockhash: 'mock-blockhash',
          lastValidBlockHeight: 123456,
        }),
      };
    }),
  };
});

describe('Temporal Obfuscation', () => {
  // Mock setup
  const mockConnection = new Connection('http://localhost:8899', 'confirmed');
  
  describe('TemporalObfuscationManager', () => {
    let manager: TemporalObfuscationManager;
    
    beforeEach(() => {
      manager = new TemporalObfuscationManager(mockConnection);
    });
    
    afterEach(() => {
      // Explicitly stop processing to clear intervals
      if (manager) {
        manager.stopProcessing();
      }
      // Clean up any timers or intervals
      jest.useRealTimers();
    });
    
    test('scheduleTransaction should return a future execution time', async () => {
      const tx = new Transaction();
      const hopIndex = 0;
      
      const executionTime = manager.scheduleTransaction(tx, hopIndex);
      
      // Execution time should be in the future
      expect(executionTime).toBeGreaterThan(Date.now());
    });
    
    test('scheduleBatchWithRandomOrder should randomize transaction order', async () => {
      // Create multiple dummy transactions
      const transactions = [
        new Transaction(),
        new Transaction(),
        new Transaction(),
        new Transaction(),
      ];
      
      // Use fake timers to control execution timing
      jest.useFakeTimers();
      
      // Spy on the scheduleTransaction method
      const scheduleSpy = jest.spyOn(manager, 'scheduleTransaction');
      
      // Schedule batch with random order
      const executionTimes = manager.scheduleBatchWithRandomOrder(transactions, 0);
      
      // Should have called scheduleTransaction for each transaction
      expect(scheduleSpy).toHaveBeenCalledTimes(transactions.length);
      
      // Should have returned an execution time for each transaction
      expect(executionTimes.length).toBe(transactions.length);
      
      // All execution times should be in the future
      for (const time of executionTimes) {
        expect(time).toBeGreaterThan(Date.now());
      }
    });
    
    test('scheduleTimeSlicedBatch should distribute transactions across time slices', async () => {
      // Create multiple dummy transactions
      const transactions = Array(10).fill(0).map(() => new Transaction());
      
      // Use fake timers to control execution timing
      jest.useFakeTimers();
      
      // Schedule time-sliced batch
      const timeSliceMap = manager.scheduleTimeSlicedBatch(transactions, 0);
      
      // timeSliceMap should not be empty
      expect(timeSliceMap.size).toBeGreaterThan(0);
      
      // Total number of scheduled transactions should match input
      let scheduledCount = 0;
      timeSliceMap.forEach(times => {
        scheduledCount += times.length;
      });
      expect(scheduledCount).toBe(transactions.length);
    });
    
    test('getDiagnosticInfo should return valid diagnostic information', () => {
      const diagnostics = manager.getDiagnosticInfo();
      
      expect(diagnostics).toHaveProperty('scheduled');
      expect(diagnostics).toHaveProperty('executed');
      expect(diagnostics).toHaveProperty('meanDelay');
      expect(diagnostics).toHaveProperty('timeWindowUtilization');
      expect(Array.isArray(diagnostics.timeWindowUtilization)).toBe(true);
    });
  });
  
  describe('TimeObfuscator', () => {
    let obfuscator: TimeObfuscator;
    
    beforeEach(() => {
      obfuscator = trackObfuscator(new TimeObfuscator(mockConnection));
    });
    
    test('obfuscateTransfer should schedule a transaction with timing privacy', async () => {
      const tx = new Transaction();
      const hopIndex = 0;
      
      const executionTime = await obfuscator.obfuscateTransfer(tx, hopIndex);
      
      // Execution time should be in the future
      expect(executionTime).toBeGreaterThan(Date.now());
    });
    
    test('obfuscateMultiTransfer should distribute multiple transactions with timing privacy', async () => {
      const transactions = Array(5).fill(0).map(() => new Transaction());
      const hopIndex = 0;
      
      const timeSliceMap = await obfuscator.obfuscateMultiTransfer(transactions, hopIndex);
      
      // Should have at least one time slice
      expect(timeSliceMap.size).toBeGreaterThan(0);
      
      // All scheduled times should be in the future
      let allTimes: number[] = [];
      timeSliceMap.forEach(times => {
        allTimes = [...allTimes, ...times];
      });
      
      for (const time of allTimes) {
        expect(time).toBeGreaterThan(Date.now());
      }
    });
    
    test('getDiagnosticInfo should return valid statistics', () => {
      const stats = obfuscator.getDiagnosticInfo();
      
      expect(stats).toHaveProperty('scheduled');
      expect(stats).toHaveProperty('executed');
      expect(stats).toHaveProperty('meanDelay');
      expect(stats).toHaveProperty('timeWindowUtilization');
    });
  });
  
  describe('TimingConnector', () => {
    let connector: TimingConnector;
    
    beforeEach(() => {
      connector = new TimingConnector(mockConnection);
    });
    
    test('setStrategy should change the timing strategy', () => {
      // Initial strategy is STANDARD (default)
      
      // Get initial timing stats
      const initialStats = connector.getTimingStats();
      
      // Change to MAXIMUM_PRIVACY
      connector.setStrategy(TimingStrategy.MAXIMUM_PRIVACY);
      
      // Get updated timing stats
      const updatedStats = connector.getTimingStats();
      
      // Correlation resistance should be higher with MAXIMUM_PRIVACY
      expect(updatedStats.correlationResistance).toBeGreaterThan(initialStats.correlationResistance);
    });
    
    test('getTimingStats should return valid timing statistics', () => {
      const stats = connector.getTimingStats();
      
      expect(stats).toHaveProperty('averageHopDelay');
      expect(stats).toHaveProperty('timeDistribution');
      expect(stats).toHaveProperty('entropyScore');
      expect(stats).toHaveProperty('correlationResistance');
      
      // Entropy and correlation scores should be between 0-100
      expect(stats.entropyScore).toBeGreaterThanOrEqual(0);
      expect(stats.entropyScore).toBeLessThanOrEqual(100);
      expect(stats.correlationResistance).toBeGreaterThanOrEqual(0);
      expect(stats.correlationResistance).toBeLessThanOrEqual(100);
    });
  });
  
  describe('TimingEnhancedConnector', () => {
    // Mock DApp config
    const dAppConfig = {
      rpcUrl: 'http://localhost:8899',
      commitment: 'confirmed' as 'processed' | 'confirmed' | 'finalized',
      maxHops: 4,
      maxSplits: 4,
      fakeSplitsPerHop: 44,
      programId: '11111111111111111111111111111111'
    };
    
    let connector: TimingEnhancedConnector;
    
    beforeEach(() => {
      connector = new TimingEnhancedConnector(dAppConfig);
      
      // Mock internal methods to avoid actual blockchain interactions
      // Verwende einen allgemeineren Typ für die Mock-Implementierung, um Typprobleme zu vermeiden
      jest.spyOn(connector as any, 'executeTransfer').mockImplementation(
        async (...args: any[]) => {
          return {
            success: true,
            signature: 'mock-signature',
            estimatedExecutionTime: Date.now() + 5000,
            timingStats: connector.getTimingStats()
          };
        }
      );
    });
    
    test('calculateEnhancedAnonymitySet should increase anonymity set size', () => {
      // Base anonymity set for BlackoutSOL is 48^4 = 5,308,416
      const baseAnonymitySet = 5308416;
      
      // Get enhanced anonymity set
      const enhancedSet = connector.calculateEnhancedAnonymitySet(baseAnonymitySet);
      
      // Enhanced set should be larger than the base set
      expect(enhancedSet).toBeGreaterThan(baseAnonymitySet);
    });
    
    test('getRecommendedTimingStrategy should adapt to transfer characteristics', () => {
      // Small transfer, single recipient, low privacy
      const lowPrivacyStrategy = connector.getRecommendedTimingStrategy(
        100000, // 0.0001 SOL
        1,
        30 // 30% privacy level
      );
      expect(lowPrivacyStrategy).toBe(TimingStrategy.MINIMAL);
      
      // Medium transfer, single recipient, medium privacy
      const mediumPrivacyStrategy = connector.getRecommendedTimingStrategy(
        1000000000, // 1 SOL
        1,
        70 // 70% privacy level
      );
      expect(mediumPrivacyStrategy).toBe(TimingStrategy.BALANCED);
      
      // High-value transfer, any privacy level should recommend high privacy
      const highValueStrategy = connector.getRecommendedTimingStrategy(
        50000000000, // 50 SOL
        1,
        30 // Even with low privacy setting
      );
      expect(highValueStrategy).not.toBe(TimingStrategy.MINIMAL);
      
      // Multi-recipient transfer should increase privacy level
      const multiRecipientStrategy = connector.getRecommendedTimingStrategy(
        1000000000, // 1 SOL
        5, // 5 recipients
        30 // Low privacy setting
      );
      expect(multiRecipientStrategy).not.toBe(TimingStrategy.MINIMAL);
    });
  });
});
