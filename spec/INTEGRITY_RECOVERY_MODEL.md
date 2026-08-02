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
this increment's recovery policy. It is an abstract oracle, not computed from
record bytes, and makes no claim about cryptography, serialization, atomic
media, or physical durability. Ordinary callers have no public mutation API to
set these verdicts; typed injection is confined to test-only support.

A later encoding and authentication increment must replace this oracle with a
mechanism that binds every security-relevant field, record identity, slot
context, version, and selector metadata. Until then, the model proves only how
the state machine responds after a verdict exists.

The required semantic inputs to that future mechanism are enumerated in
[Canonical Record Model](CANONICAL_RECORD_MODEL.md). Increment 4 defines those
inputs but still produces no canonical bytes or real verdict mechanism.

## Authority-preserving recovery rules

### Clean

- Structural validation requires no pending command outcome.
- A valid selector and valid selected record remain authoritative.
- A corrupted selected record is an explicit failure; an unselected record is
  never promoted merely because it is valid.
- With a corrupted selector, exactly one valid complete record may be selected.
- Zero valid records is unrecoverable.
- Two valid records without trustworthy authority metadata is ambiguous and
  fails closed.

### Prepared

- Structural validation requires a pending command outcome, an in-range
  candidate slot, and a phase commit identifier that is the checked successor
  of the valid previous record. When the selector is valid, the candidate slot
  must be distinct from the selected previous slot. These requirements apply
  even when the candidate record is corrupted or missing.
- The candidate has not crossed the selector-commit boundary.
- A corrupted or incomplete candidate is discarded when the complete previous
  authoritative record remains valid.
- A corrupted previous authoritative record is not replaced by the candidate.
- If the selector verdict is corrupted, structurally valid `Prepared` phase
  metadata identifies the non-candidate slot as the previous authority; the raw
  selector payload is ignored.
- No field of a record whose verdict is `Corrupted`, including its commit
  identifier or lifecycle, is trusted.
- Recovery never releases the prepared command outcome.

### Committed

- Structural validation requires no pending command outcome and requires the
  distinct previous record to be present even when its verdict is `Corrupted`.
- The selected next record is authoritative.
- A corrupted obsolete previous record may be discarded.
- A corrupted selected next record is an explicit failure; recovery never
  rolls back to the obsolete record.
- If the selector verdict is corrupted, structurally valid `Committed` phase
  metadata identifies the selected next slot; the raw selector payload is
  ignored.
- The selected record identifier must agree with phase metadata and, whenever
  the previous record remains valid, be its checked successor.

### Audit provenance

- Successful recovery audit commit identifiers come from the trustworthy
  selected record, not untrusted selector, phase, or corrupted-record fields.
- When a discarded candidate or previous record is `Valid`, its lifecycle is
  the audit's prior lifecycle. When that discarded record is `Corrupted` or
  absent where absence is permitted, its lifecycle is not trusted and the
  trustworthy selected lifecycle is duplicated as the prior lifecycle.

## Mutation rule

Recovery first validates structural phase and outcome metadata, then computes
and validates a complete plan before changing a slot, selector, outcome, or
phase. Every failed recovery returns a stable error and preserves the complete
`DurableModel`.

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
- selector-corrupted recovery ignores the raw selector payload;
- malformed phase/outcome metadata fails before mutation, including when a
  record verdict is corrupted;
- successful recovery audits use only trustworthy lifecycle and commit-ID
  provenance;
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
