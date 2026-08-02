# Executable Model Test Plan

Status: Phase 0 plan for the first Rust model increment.

## Test levels

- **Unit tests:** individual state predicates, authorization decisions, counter comparisons, measurement extension, and rejection classification.
- **Transition-table tests:** every lifecycle state crossed with every command category.
- **Property tests:** generated command sequences preserve invariants and never reuse protected counter scopes.
- **Interruption tests:** reset or persistent-write failure at every injection point.
- **Vector tests:** validated candidate-neutral positive and negative semantic
  objects, cross-object authority-release bindings, lifecycle/persistence
  conformance adapters in `baochip-semantic-fixtures`, stable abstract receipt
  claims, and later candidate-specific encoded positive/negative vectors.
- **Model exploration:** bounded enumeration of short command sequences from every reachable state.

## Requirement-to-test mapping

| Requirement group | First model evidence |
|---|---|
| `BCR-EVI-001`–`005` | abstract receipt mutation/omission tests; typed evidence/appraisal separation; later encoding vectors |
| `BCR-ID-001`–`004` | opaque key-handle API; prohibited export commands; revocation/recommission tests; linkability metadata tests |
| `BCR-MEA-001`–`005` | order permutation, domain substitution, context mismatch, commitment binding, and prohibited-correctness-inference cases |
| `BCR-FRE-001`–`004` | challenge reuse, sequence reuse, rollback tuples, reset/interruption, and unknown-history cases |
| `BCR-LIF-001`–`004` | complete state/command matrix, authority-policy tests, update/recovery downgrade tests, decommission terminality |
| `BCR-VER-001`–`004` | offline fixtures, layered outcomes, stable rejection codes, positive/negative vector coverage |
| `BCR-ASR-001`–`004` | generated coverage report, claim-to-test ledger, evidence-type labels, explicit unsupported-claim assertions |

## Mandatory invariant tests

1. Only `OPERATIONAL` issues operational receipts.
2. `DECOMMISSIONED` is terminal.
3. A revoked identity never becomes valid again.
4. Recommissioning creates a new identity and advances `device_generation`.
5. Protected counters never decrease or wrap.
6. Interrupted persistence never yields undocumented mixed state.
7. Update and recovery obey version and authority policy.
8. Measurement order and domain affect the accumulator.
9. Receipt challenges and sequences are bound to the authenticated claim set.
10. Unknown or unsupported critical semantics fail closed.
11. A corrupted prepared candidate is never promoted over the previous authority.
12. A corrupted committed selection is never rolled back to an obsolete record.
13. Integrity-recovery errors preserve every slot, selector, outcome, and phase field.
14. A rejection that intentionally enters `FAULT` is committed as a complete durable snapshot.
15. Semantic authority fixtures bind phase identifiers to the exact previous
    and next record identifiers without wrap.
16. A semantic receipt release binds a required key-generation context and the
    matching committed authority subject and snapshot identifier.
17. Every stable semantic validation error has one unique negative fixture with
    an exact expected classification.
18. Public lifecycle-state, rejection, receipt, counter, commit, and
    persistence-phase surfaces remain conformant with the frozen semantic
    projections exercised by the adapter gate.

## First increment acceptance gate

- all lifecycle states and stable rejection classes compile as typed Rust values;
- every state/command category has an explicit outcome test;
- no external dependencies are required for the core state transition crate;
- `cargo fmt --check`, `cargo clippy --all-targets --all-features -- -D warnings`, and `cargo test --all-targets` pass;
- property-test and cryptographic dependencies remain deferred to separate reviewed commits; and
- documentation names the exact implemented subset and unimplemented requirements.
