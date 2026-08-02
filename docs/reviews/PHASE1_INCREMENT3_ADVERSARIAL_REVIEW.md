# Phase 1 Increment 3 Adversarial Review

## Scope

This review examined the abstract integrity-recovery policy and its two-slot
executable model against baseline commit
`0b8c4cae2be85a399d9b26a89d4a2157cb9b32ed`. It covered recovery authority,
structural phase and outcome invariants, selector corruption, record verdicts,
mutation ordering, audit provenance, commit identifiers, and the bounded
rejected-to-`FAULT` interruption path.

The review did not select or assess physical storage, atomic media behavior,
record serialization, checksums, hashes, MACs, signatures, key management,
FPGA or RTL behavior, wear, retention, power timing, concurrency, production
fault detection, runtime dependencies, or CI infrastructure.

## Baseline

- Lifecycle model: 34 tests.
- Persistence model: 42 tests.
- Workspace total: 76 tests.
- Integrity verdict: an abstract trusted oracle input, not a byte-derived
  mechanism.

Ordinary callers have no public mutation API for setting integrity verdicts.
Typed verdict injection remains confined to test-only support.

## Defects and severity

### High — structural invariant bypass during integrity recovery

Integrity recovery did not consistently validate phase and pending-outcome
metadata before mutation. A corrupted candidate could mask a forged
`Prepared.commit_id` or missing outcome; corrupted committed metadata could
mask an unexpected outcome or missing previous record. These gaps allowed
malformed state to bypass the normal structural invariant and risk mutation
based on untrusted or incomplete authority metadata.

### Medium — audit provenance collapsed distinct lifecycles

Successful integrity recovery duplicated the selected lifecycle for both audit
lifecycle fields. This lost the trustworthy lifecycle of a valid discarded
candidate or previous record. Commit IDs also needed an explicit rule tying
successful audits to the trustworthy selected record rather than phase or
corrupted-record metadata.

### Low — rejected-to-`FAULT` crash coverage gap

The bounded counter-exhaustion path proved that a rejection can durably stage
and commit a `FAULT` snapshot, but did not directly cover crashes on both sides
of selector commit with exact snapshot, withheld-outcome, audit, and identifier
assertions.

No critical defect, hardcoded secret, third-party runtime dependency, or use of
unsafe Rust was identified in this review scope.

## Exact fixes

- Validate structural phase, slot, required-record, pending-outcome, and phase
  commit-ID metadata before any integrity-recovery mutation.
- Require `Clean` and `Committed` to have no pending outcome and `Prepared` to
  have exactly one.
- Require `Prepared.commit_id` to be the checked successor of the valid
  previous record even when its candidate is corrupted or missing. Never trust
  any field from a corrupted record.
- Require a `Committed` previous record to be structurally present even when
  that record is corrupted.
- When selector integrity is corrupted, ignore its raw slot payload and use
  structurally validated phase authority, or sole-valid-record authority in
  `Clean`.
- Preserve failed recovery as a stable, whole-model non-mutation.
- Use a discarded record's lifecycle as audit provenance only when that record
  is `Valid`; otherwise duplicate the trustworthy selected lifecycle.
- Source every successful recovery audit commit ID from the trustworthy
  selected record.
- Add direct before-commit and after-commit crash tests for the bounded
  rejected-to-`FAULT` transition.

## Required adversarial cases

- A corrupted prepared candidate does not mask a forged phase commit ID.
- A corrupted prepared candidate does not mask a missing pending outcome.
- A corrupted committed previous record does not mask an unexpected pending
  outcome.
- A corrupted selector in `Clean` does not mask an unexpected pending outcome.
- A corrupted selector in `Committed` does not permit a missing previous
  record.
- Selector-corrupted `Prepared` recovery audits the valid discarded candidate
  lifecycle and selected previous lifecycle.
- Selector-corrupted `Committed` recovery audits the valid discarded previous
  lifecycle and selected next lifecycle.
- Selector-corrupted `Prepared` and `Committed` phases reject out-of-range
  indices without panic or mutation.
- Corrupted discarded candidate and previous records never contribute
  lifecycle or commit-ID fields to successful recovery audits.
- A corrupted selector's deliberately out-of-range raw slot payload is ignored
  in `Clean`, `Prepared`, and `Committed` when sole-valid or phase metadata
  establishes unambiguous authority.
- A rejected-to-`FAULT` crash before selector commit restores the exact prior
  snapshot and does not release the rejection outcome.
- A rejected-to-`FAULT` crash after selector commit preserves the exact `FAULT`
  snapshot, audit, and authoritative commit ID.
- A valid prepared candidate never replaces a corrupted previous authority.
- A valid selector in `Clean` never promotes a valid duplicate over its
  corrupted selected record.
- A corrupted selector in `Prepared` does not mask a malformed phase commit ID.
- A corrupted selector in `Committed` does not mask a malformed phase commit
  ID.
- A corrupted committed previous record does not mask disagreement between the
  phase and selected record commit IDs.
- A corrupted selector in `Committed` does not mask a previous/selected slot
  conflict.
- A missing selected committed record fails stably when either the previous
  record or selector is corrupted.

Each malformed case requires its stable error where applicable and complete
model equality after failure.

## Exact test totals after correction

- Lifecycle model: 34 tests (unchanged).
- Persistence model: 62 tests (42 baseline plus 20 adversarial tests).
- Workspace total: 96 tests.

## Remaining limitations

- Integrity verdicts remain trusted abstract oracle inputs; the model does not
  compute them from a canonical representation.
- There is no canonical serialization, checksum, digest, MAC, signature,
  selector encoding, redundancy scheme, or malicious-corruption model.
- Torn writes, partial-field corruption, physical atomicity, wear, retention,
  storage geometry, power timing, concurrency, multi-writer arbitration, and
  physical rollback resistance remain outside scope.
- Test-only typed injection demonstrates policy branches, not a production
  corruption detector or public mutation interface.

## Validation checklist

Final local validation passed:

```text
cargo fmt                                                         PASS
cargo fmt --check                                                 PASS
cargo check --workspace --all-targets                             PASS
cargo check --workspace --all-targets --all-features              PASS
cargo clippy --workspace --all-targets --all-features -- -D warnings
                                                                  PASS
cargo test --workspace --all-targets                              PASS
cargo tree --workspace --edges normal                             PASS
git diff --check                                                  PASS
git status --short                                                PASS
```

The test run reported exactly 34 lifecycle, 62 persistence, and 96 workspace
tests with no failure. Default and all-feature compilation passed. The normal
dependency tree remains limited to the two local workspace crates. Scans of
all changed files found no secret-like values or `unsafe` block, and review of
security-claim terms confirmed that each occurrence is an explicit exclusion,
limitation, or denial rather than an unsupported claim.

## Evidence boundary

The corrected model establishes deterministic recovery decisions after a
trusted abstract verdict and preserves logical authority under the tested
faults. **These results are neither physical durability nor cryptographic
integrity evidence.** They also make no serialization, atomic-media, tamper
resistance, or production hardware-security claim.
