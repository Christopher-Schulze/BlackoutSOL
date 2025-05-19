/**
 * Test Utilities for Resource Cleanup
 * 
 * This module provides utilities to track and clean up resources that might
 * cause lingering handles in tests, such as intervals and timeouts.
 */

import { TimeObfuscator } from '../../src/timing/temporal-obfuscation';
import { afterEach, afterAll } from '@jest/globals';

// Store resources that need cleanup
const resources = {
  obfuscators: [] as TimeObfuscator[],
  intervals: [] as NodeJS.Timeout[],
};

/**
 * Register a TimeObfuscator instance for automatic cleanup
 * 
 * @param obfuscator The TimeObfuscator instance to track
 * @returns The same instance for chaining
 */
export function trackObfuscator(obfuscator: TimeObfuscator): TimeObfuscator {
  resources.obfuscators.push(obfuscator);
  return obfuscator;
}

/**
 * Register a timer/interval for automatic cleanup
 * 
 * @param timer The timer/interval to track
 * @returns The same timer for chaining
 */
export function trackTimer(timer: NodeJS.Timeout): NodeJS.Timeout {
  resources.intervals.push(timer);
  return timer;
}

/**
 * Clean up all tracked resources
 */
export function cleanupResources(): void {
  // Clean up obfuscators
  resources.obfuscators.forEach(obfuscator => {
    if (obfuscator && typeof obfuscator.cleanup === 'function') {
      try {
        obfuscator.cleanup();
      } catch (e) {
        console.error('Error cleaning up obfuscator:', e);
      }
    }
  });
  resources.obfuscators = [];

  // Clean up intervals
  resources.intervals.forEach(interval => {
    try {
      clearInterval(interval);
      clearTimeout(interval);
    } catch (e) {
      console.error('Error clearing interval:', e);
    }
  });
  resources.intervals = [];
}

/**
 * Register this module's cleanup with Jest afterEach/afterAll hooks
 * 
 * This should be called in the global setup for Jest or in each test suite
 */
export function registerCleanupHooks(): void {
  // Clean up after each test
  afterEach(() => {
    cleanupResources();
  });

  // Final cleanup after all tests
  afterAll(() => {
    cleanupResources();
  });
}

// Export all functions as properties of the cleanupResources function
// This allows both function call style and object property style usage
cleanupResources.trackObfuscator = trackObfuscator;
cleanupResources.trackTimer = trackTimer;
cleanupResources.registerCleanupHooks = registerCleanupHooks;
