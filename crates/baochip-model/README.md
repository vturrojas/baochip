# Baochip executable model

This crate is the dependency-free Phase 1 lifecycle model described by `spec/EXECUTABLE_MODEL.md`.

It currently models:

- lifecycle transitions and authorization roles;
- device generation and transition counters;
- update and recovery rules, including abstract update-validation outcomes;
- revocation, recommissioning, and decommissioning;
- measurement epochs;
- abstract receipt issuance and sequence allocation; and
- stable rejection classes.

It does not implement serialization, cryptography, persistence or interruption
emulation, host transport, RTL, or production security. Its update-validation
inputs are deterministic semantic test inputs, not cryptographic verification.
