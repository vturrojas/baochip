# Persistence and Interruption Model

Status: Phase 1 Increment 3 behavioral specification. This is an abstract atomic-storage model, not a claim about flash, EEPROM, fuses, filesystems, or FPGA hardware.

## Objective

For every security-relevant state change, interruption recovery yields exactly one of:

1. the complete previously committed state;
2. the complete next committed state; or
3. an explicit integrity failure that cannot issue operational evidence.

An undocumented mixture of old and new fields is never authoritative.

## Two-slot model

The first model uses two logical records and one authoritative selector:

- `active`: the slot selected as authoritative;
- `inactive`: the slot available for the next complete candidate record; and
- `commit_id`: a monotonically increasing logical record identifier.

Each record contains one complete `StateMachine` snapshot, its `commit_id`, and
an abstract trusted integrity verdict. Increment 3 recovery policy is defined
in [Integrity Recovery Model](INTEGRITY_RECOVERY_MODEL.md). The verdict is an
oracle input, not a result computed from serialized bytes, and implies no
cryptographic integrity or physical durability. Ordinary callers have no public
mutation API for setting verdicts.

## Transaction phases

### Clean

The authoritative selector identifies a complete active record. No candidate transition is pending.

### Prepared

The command has executed against a clone of the active state and a complete candidate record has been written to the inactive slot. The authoritative selector still identifies the previous record.

A crash in this phase discards the candidate and recovers the previous state.

### Committed

The authoritative selector has atomically changed to the candidate slot. The previous record may still exist.

A crash in this phase recovers the next state selected as authoritative.

### Cleaned

The obsolete previous record has been erased or made reusable. The model returns to `Clean`.

## Complete internal invariant

Before an ordinary persistence operation reads, selects, or clears a record,
the model validates the complete phase/selector/record relationship:

- every referenced slot index is in range;
- the authoritative selector identifies a complete record;
- `Clean` has no untracked candidate record and forbids a pending command
  outcome;
- `Prepared` identifies a complete inactive record distinct from the active
  slot, requires and binds its pending command outcome, and agrees with that
  record's `commit_id`;
- `Committed` agrees with the actual selector, identifies a distinct complete
  previous record, forbids a pending command outcome, and agrees with the
  selected record's `commit_id`; and
- every candidate or selected next identifier is exactly the checked successor
  of the previous authoritative identifier.

Invariant errors are stable and leave the complete model unchanged.
Integrity-aware recovery uses the authority-preserving rules in the Integrity
Recovery Model rather than requiring every corrupted or incomplete record to
satisfy the ordinary invariant. It still validates structural phase, slot,
required-record, outcome, and trustworthy identifier metadata before clearing
either slot, so malformed metadata cannot erase the only recoverable record.
`Prepared` recovery requires the phase identifier to be the checked successor
of the valid previous record even when the candidate is corrupted or missing;
fields from a corrupted record are never trusted. `Committed` recovery
requires the previous record to remain structurally present even when its
verdict is `Corrupted`.

## Rejected commands

- A rejection that leaves lifecycle state unchanged creates no candidate record.
- A rejection that intentionally moves the lifecycle model into `FAULT` is a durable state change and must be prepared and committed like any other transition.
- Every rejection result identifies both the rejection class and whether a durable candidate was staged.

## Commit identifiers

- A prepared record receives `active.commit_id + 1`.
- Identifiers never wrap or reset silently.
- A crash before selector commit may consume no externally authoritative identifier.
- A crash after selector commit makes the next identifier authoritative.
- Commit identifiers order complete persistent snapshots; they do not replace device generation, transition count, measurement epoch, or receipt sequence.

## Outcome release

Preparation may execute a command against a candidate clone, but it does not
release the command outcome or any receipt claims to the caller. The outcome is
released only by the abstract selector-commit operation that makes the complete
candidate authoritative. This prevents callers from treating a prepared record
as durable and prevents a prepare/crash/retry sequence from releasing duplicate
receipt sequences.

`Prepared`, `Committed`, and the persistence audit operation names describe
logical authority inside this executable model only. They do not assert that a
flash, EEPROM, filesystem, fuse, or other physical medium completed a write.

## Interruption audit

Every preparation, commit, cleanup, rejected command, and simulated crash returns a non-secret persistence audit event containing:

- operation kind;
- prior and resulting lifecycle state;
- relevant commit identifier; and
- whether recovery selected the previous or next record.

This does not yet satisfy the complete lifecycle authorization-audit schema.

## Current exclusions

- torn writes within a record;
- computation or authentication of selector and record integrity verdicts;
- wear, retention, power timing, and storage geometry;
- concurrency and multi-writer access;
- real serialization or checksums;
- physical rollback resistance; and
- platform-specific atomic-write guarantees.

These are explicit later increments. The two-slot model first establishes transaction semantics and test injection points.

## Increment 3 persistence acceptance gate

- crashes in `Prepared` recover the exact previous snapshot;
- crashes in `Committed` recover the exact next snapshot;
- cleanup preserves the selected next snapshot;
- unchanged rejected commands produce no candidate record;
- fault-producing rejected commands can be durably committed;
- commit identifiers advance without reuse or wrap;
- persistence operations reject invalid phase ordering;
- malformed phase, selector, record, and commit-identifier relationships return
  stable errors without panicking or partially mutating the model;
- command outcomes are withheld until selector commit;
- the crate has no third-party runtime dependencies; and
- formatting, Clippy with denied warnings, tests, and `git diff --check` pass.
