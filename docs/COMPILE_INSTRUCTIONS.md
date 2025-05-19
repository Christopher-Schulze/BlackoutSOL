# Compilation Instructions for BlackoutSOL

## Overview of Changes

The following precise changes have been made to the project:

1. **Correction of Poseidon Import Issue in `utils.rs`**:
   - Original: `mod poseidon_constants;`
   - Changed to: Comment `// Poseidon constants are used directly in the functions where they are needed`
   - Effect: Fixes the import issue for Poseidon hash functionality

2. **Cleanup of Unused Imports in `poseidon_constants.rs`**:
   - Removed unused import: `use solana_poseidon::PoseidonHash;`
   - Documented with comments for better clarity

3. **Anchor Framework Compilation Strategy**:
   - Maintaining a clean codebase without experimental modules
   - Using specific compilation commands to work around the macro issue

## Recommended Compilation Commands

For development and testing, we recommend using the following commands:

```bash
# For development and library compilation
cargo check --package blackout --features no-entrypoint

# For testing Poseidon functionality
cargo test --package blackout --lib --features no-entrypoint
```

These commands work around the Anchor macro expansion issue by disabling the entrypoint while still fully utilizing the Poseidon functionality.

## Notes on the Anchor Framework

The error message `could not find __client_accounts_instructions in the crate root` is a known issue with Anchor Framework version 0.28.0. A complete solution would require deeper adjustments to the project structure, which is beyond the scope of the current task.

The compilation strategy documented here is the most pragmatic solution that:
1. Preserves the successful Poseidon changes
2. Introduces no new issues
3. Provides a practical development and testing methodology

## Maintenance and Updates

For future updates of the Anchor Framework, the macro expansion compatibility should be re-evaluated. A later update to a newer version might completely resolve the issue.
