# Integrity Recovery Model

Status: Phase 1 Increment 3 behavioral specification. This document defines
recovery policy after an abstract integrity verdict. It does not define or
claim a checksum, digest, MAC, signature, encoding, atomic medium, or physical
fault detector.

## Objective

Baochip must never make corrupted persistent state authoritative merely to
continue operating. Recovery must preserve the last authority decision when it
can be established without ambiguity; otherwise it fails closed without
mutating the recoverable model.

## Abstract verdict boundary

Each logical record and the authoritative selector carry an
`IntegrityVerdict` of `Valid` or `Corrupted`. The verdict is a trusted input to
this increment's recovery policy. It is not computed from record bytes.

A later encoding and authentication increment must replace this oracle with a
mechanism that binds every security-relevant field, record identity, slot
context, version, and selector metadata. Until then, the model proves only how
the state machine responds after a verdict exists.

## Authority-preserving recovery rules

### Clean

- A valid selector and valid selected record remain authoritative.
- A corrupted selected record is an explicit failure; an unselected record is
  never promoted merely because it is valid.
- With a corrupted selector, exactly one valid complete record may be selected.
- Zero valid records is unrecoverable.
- Two valid records without trustworthy authority metadata is ambiguous and
  fails closed.

### Prepared

- The candidate has not crossed the selector-commit boundary.
- A corrupted or incomplete candidate is discarded when the complete previous
  authoritative record remains valid.
- A corrupted previous authoritative record is not replaced by the candidate.
- If the selector verdict is corrupted, structurally valid `Prepared` phase
  metadata identifies the non-candidate slot as the previous authority.
- Recovery never releases the prepared command outcome.

### Committed

- The selected next record is authoritative.
- A corrupted obsolete previous record may be discarded.
- A corrupted selected next record is an explicit failure; recovery never
  rolls back to the obsolete record.
- If the selector verdict is corrupted, structurally valid `Committed` phase
  metadata identifies the selected next slot.
- The selected record identifier must agree with phase metadata and, whenever
  the previous record remains valid, be its checked successor.

## Mutation rule

Recovery computes and validates a complete plan before changing a slot,
selector, outcome, or phase. Every failed recovery returns a stable error and
preserves the complete `DurableModel`.

## Typed test injection

The lifecycle crate has a non-default `test-support` feature containing only
typed faults needed to reach otherwise impractical fail-closed paths. The
persistence crate uses it only as a development dependency. Normal builds do
not expose arbitrary model mutation.

The persistence crate also has crate-test-only typed faults for the active
record, inactive record, selector, all records, and an ambiguous duplicate
record fixture. These test verdicts are not runtime corruption detectors.

## Rejected-to-FAULT durability

A command that returns a rejection while intentionally changing lifecycle
state to `FAULT` produces a candidate record. The persistence layer withholds
the rejection until selector commit, then makes the complete `FAULT` snapshot
authoritative. A counter-exhaustion injection directly exercises this path.

## Increment 3 acceptance gate

- corrupted selected state never issues evidence or accepts commands;
- a corrupted prepared candidate is discarded without promotion;
- a corrupted obsolete committed record is discarded while preserving the
  selected next record;
- a corrupted selected committed record never causes rollback;
- selector corruption follows phase authority only when unambiguous;
- zero-valid-record and ambiguous cases fail closed without mutation;
- rejected-to-`FAULT` persistence has a direct bounded test;
- fault injection is typed and excluded from default lifecycle builds;
- no third-party runtime dependency is added; and
- formatting, Clippy with denied warnings, tests, and `git diff --check` pass.

## Deferred work

- canonical record serialization;
- real integrity or authenticity algorithms;
- torn-write and partial-field corruption;
- selector encoding and redundancy;
- maliciously coordinated corruption of record and phase metadata;
- wear, retention, storage geometry, and power timing;
- concurrency and multi-writer arbitration; and
- physical rollback resistance.
