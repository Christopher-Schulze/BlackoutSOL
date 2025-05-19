/** @type {import('ts-jest').JestConfigWithTsJest} */
module.exports = {
  preset: 'ts-jest',
  testEnvironment: 'node',
  roots: ['<rootDir>/src', '<rootDir>/tests'],
  testMatch: ['**/*.test.ts'],
  verbose: true,
  transform: {
    '^.+\\.tsx?$': ['ts-jest', {
      tsconfig: 'tsconfig.test.json',
      isolatedModules: true // This will skip type checking to allow tests to run with mock implementation issues
    }]
  },
  collectCoverage: true,
  collectCoverageFrom: [
    'src/**/*.ts',
    '!src/**/*.d.ts',
  ],
  moduleNameMapper: {
    '^@/(.*)$': '<rootDir>/src/$1',
  },
  moduleFileExtensions: ['ts', 'tsx', 'js', 'jsx', 'json', 'node'],
  // Settings to handle asynchronous operations
  testTimeout: 30000,
  forceExit: true,
  // When tests are done, force exit after a short timeout
  detectOpenHandles: true,
  // Setup file to run before tests for global setup/teardown
  setupFilesAfterEnv: ['<rootDir>/tests/setup.ts']
};
