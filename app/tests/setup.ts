/**
 * Jest Test Setup
 * 
 * This file is loaded automatically by Jest before running tests.
 * It sets up the testing environment with necessary utilities and hooks.
 */

import { afterEach, afterAll } from '@jest/globals';
import { cleanupResources } from './utils/test-cleanup';
import { TimeObfuscator } from '../src/timing/temporal-obfuscation';

// Mock certain globals for testing
// This ensures tests don't produce side effects or long-running operations
const originalSetTimeout = global.setTimeout;
const originalSetInterval = global.setInterval;
const originalClearTimeout = global.clearTimeout;
const originalClearInterval = global.clearInterval;

// Patch the TimeObfuscator constructor to ensure all instances are tracked
const originalTimeObfuscatorConstructor = TimeObfuscator.prototype.constructor;
Object.defineProperty(TimeObfuscator.prototype, 'constructor', {
  value: function(...args: any[]) {
    const result = originalTimeObfuscatorConstructor.apply(this, args);
    // Register for cleanup in afterEach
    cleanupResources.trackObfuscator(this);
    return result;
  }
});

// Clean up any resources after each test
afterEach(() => {
  // Reset mocks
  jest.clearAllMocks();
  // Clean up tracked resources to prevent open handles
  cleanupResources();
});

// Final cleanup after all tests
afterAll(() => {
  // Restore original timers and constructors
  global.setTimeout = originalSetTimeout;
  global.setInterval = originalSetInterval;
  global.clearTimeout = originalClearTimeout;
  global.clearInterval = originalClearInterval;
  
  // Final resource cleanup
  cleanupResources();
});
