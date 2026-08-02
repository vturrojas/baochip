# Phase 1 Increment 2 Adversarial Review

## Scope

This independent review examined the abstract two-slot persistence model in
`baochip-persistence-model` against the project charter, decision log,
persistence and executable-model specifications, lifecycle and counter models,
test plan, traceability matrix, lifecycle implementation, and persistence
implementation.

The review attacked every public persistence operation and malformed internal
phase/selector/record relationship. It did not select physical storage or add
cryptography, serialization, FPGA behavior, RTL, runtime dependencies, or CI.

## Baseline

- Lifecycle model: 34 tests.
- Persistence model: 8 tests.
- Workspace total: 42 tests.
- Persistence phases: `Clean`, `Prepared`, and `Committed`.
- Storage abstraction: two logical record slots and one authoritative selector.

## Defects and severity

### High — malformed metadata could erase authority or panic

Persistence operations trusted raw phase slot indices and slot relationships.
A malformed `Prepared` phase could alias its candidate to the authoritative
slot, and a malformed `Committed` phase could alias its previous slot to the
selected slot. Recovery or cleanup could then clear the authoritative record.
Out-of-range selector or phase indices indexed the two-element array directly
and could panic instead of returning a stable error.

### High — preparation released outcomes before selector commit

`prepare` returned `CommandOutcome::Applied`, including receipt claims, while
the previous record was still authoritative. A prepare/crash/retry sequence
could therefore release the same receipt sequence twice even though neither
candidate had become authoritative. It also let callers infer completion before
the selector-commit boundary.

### High — phase and record commit identifiers were not cross-checked

`commit`, `cleanup`, and recovery trusted the identifier stored in phase
metadata without checking the candidate or selected record or requiring the
identifier to be the checked successor of the previous authoritative record.
Malformed state could therefore produce inconsistent audits or authoritative
identifier regression.

### Medium — recovery validation occurred after destructive mutation

Prepared recovery cleared the candidate before validating the active record.
Committed recovery cleared the previous record before validating the selected
record. A later error could leave the only recoverable state destroyed and the
phase partially changed.

### Medium — missing-record and cleanup audit semantics were ambiguous

A missing previous record was reported as `MissingActiveRecord`, conflating an
obsolete-record failure with loss of the selected record. Cleanup also reported
the selected lifecycle as both the prior and resulting lifecycle, hiding the
lifecycle of the record it removed.

No critical defect, hardcoded secret, third-party runtime dependency, or use of
unsafe Rust was found.

## Fixes

- Added one complete internal invariant validator and invoke it before every
  persistence read or mutation.
- Validate selector and phase indices, slot distinctness, required and
  unexpected records, prepared-outcome binding, selector agreement, phase and
  record identifier agreement, and checked successor ordering.
- Added stable, distinct errors for invalid indices, slot conflicts, selector
  mismatch, commit-ID mismatch, missing previous records, missing or unexpected
  prepared outcomes, and untracked records.
- Validate all required records before cleanup or recovery clears either slot;
  every error preserves the complete `DurableModel`.
- Withhold command outcomes during preparation. `commit` now returns a
  `CommitResult` only after the abstract selector advances.
- Correct cleanup audits to bind the obsolete prior lifecycle and selected
  resulting lifecycle.
- Clarified in the canonical persistence specification that transaction and
  audit names describe logical authority, not physical write completion.

## Malformed-state cases tested

- `Prepared` references an empty candidate slot.
- `Prepared.slot` equals the active selector.
- `Prepared.commit_id` differs from the candidate record.
- Prepared record and phase agree with each other but are not the active
  record's checked successor.
- `Prepared` has no bound candidate outcome.
- Prepared recovery has a missing active record.
- `Committed.active_slot` differs from the actual selector.
- `Committed.previous_slot` equals the active slot.
- `Committed.commit_id` differs from the selected record.
- Selected committed identifier regresses behind the previous record.
- `Committed` has a missing previous record.
- Committed recovery has a missing selected record.
- `Committed` retains an impossible prepared outcome.
- The active selector references an empty record.
- The active selector, prepared slot, or committed previous slot has an
  out-of-range index.
- `Clean` contains an untracked inactive record.

Every malformed-state operation asserts its exact stable error where applicable
and complete-model equality after failure. The index cases return errors rather
than panicking.

## Transaction, rejection, and identifier evidence

Tests preserve the exact previous snapshot after a prepared crash and the exact
next snapshot after a committed crash. They cover prepare/crash/prepare,
prepare/commit/crash/new-transaction, cleanup followed by multiple additional
transactions, and invalid ordering followed by valid recovery. Authoritative
commit IDs advance monotonically, never wrap, and do not regress when a prepared
candidate is discarded.

An unchanged lifecycle rejection stages no record and its audit binds the prior
and resulting lifecycle plus the active commit ID. The generic preparation path
also retains a changed candidate when lifecycle execution returns `Err`, so a
fault-producing rejection is structurally stageable. A direct cross-crate test
cannot currently construct that case through finite public commands: valid
public lifecycle paths preserve private invariants, and exhaustion requires an
impractical number of operations. Adding a production state-mutation hook only
for this test would weaken the model boundary. A future explicitly typed,
test-controlled fault-injection event should close this coverage limitation.

## Exact test totals

- Lifecycle model: 34 tests (unchanged).
- Persistence model: 32 tests (8 baseline plus 24 adversarial tests).
- Workspace total: 66 tests.

## Remaining limitations

- This model does not cover torn writes, corrupted record integrity metadata,
  corrupted physical selectors, wear, retention, power timing, storage
  geometry, concurrency, serialization, checksums, or physical rollback
  resistance.
- It does not prove that any physical platform supplies an atomic selector.
- Fault-producing rejected-command durability is implemented generically but
  lacks a direct public-path test until a bounded fault-injection event exists.
- `CommandOutcome::Applied` describes candidate lifecycle execution; only
  `CommitResult` makes that outcome caller-visible after logical selector
  commit.

## Validation evidence

The final tree is validated with:

```text
cargo fmt
cargo fmt --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-targets
cargo tree --workspace --edges normal
git diff --check
```

Changed files are also scanned for secret-like material and `unsafe` Rust. The
dependency tree remains limited to the two local workspace crates.

## Durability boundary

This review and implementation establish deterministic logical
selector/record semantics in an executable model. **They are not a claim of
physical durability, atomic media behavior, tamper resistance, or production
hardware security.**
