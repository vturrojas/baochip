# Persistence and Interruption Model

Status: Phase 1 Increment 2 behavioral specification. This is an abstract atomic-storage model, not a claim about flash, EEPROM, fuses, filesystems, or FPGA hardware.

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

Each record contains one complete `StateMachine` snapshot and its `commit_id`. Integrity metadata is abstracted until the corruption model is added.

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

## Interruption audit

Every preparation, commit, cleanup, rejected command, and simulated crash returns a non-secret persistence audit event containing:

- operation kind;
- prior and resulting lifecycle state;
- relevant commit identifier; and
- whether recovery selected the previous or next record.

This does not yet satisfy the complete lifecycle authorization-audit schema.

## Initial exclusions

- torn writes within a record;
- corrupted selector or record integrity metadata;
- wear, retention, power timing, and storage geometry;
- concurrency and multi-writer access;
- real serialization or checksums;
- physical rollback resistance; and
- platform-specific atomic-write guarantees.

These are explicit later increments. The two-slot model first establishes transaction semantics and test injection points.

## Increment 2 acceptance gate

- crashes in `Prepared` recover the exact previous snapshot;
- crashes in `Committed` recover the exact next snapshot;
- cleanup preserves the selected next snapshot;
- unchanged rejected commands produce no candidate record;
- fault-producing rejected commands can be durably committed;
- commit identifiers advance without reuse or wrap;
- persistence operations reject invalid phase ordering;
- the crate has no third-party runtime dependencies; and
- formatting, Clippy with denied warnings, tests, and `git diff --check` pass.
