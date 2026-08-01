# Executable Model Specification

Status: Phase 0 implementation contract. This document defines the first software model's observable behavior without defining its final API or receipt encoding.

## Purpose

The executable model exists to falsify lifecycle, persistence, measurement, and receipt assumptions before RTL work. It is not an emulator of a chosen chip and is not evidence of physical security.

## Model boundary

The model contains:

- committed persistent lifecycle state;
- staged transition state;
- a protected logical device identity handle;
- policy and firmware version commitments;
- monotonic generation and sequence state;
- an ordered measurement accumulator;
- deterministic command processing;
- explicit reset and interruption injection points; and
- structured outcomes and rejection reasons.

The model does not initially contain:

- real cryptographic implementations;
- a CBOR, EAT, COSE, or other wire encoding;
- physical entropy, timing, voltage, or side-channel behavior;
- FPGA peripherals or a host transport;
- claims of tamper resistance; or
- arbitrary application or AI execution.

## Determinism

Every state transition is a pure function of:

```text
(committed_state, staged_state, command, injected_event) -> (next_state, outcome, audit_event)
```

Random values, signatures, persistent-write completion, and reset timing are supplied through explicit test-controlled interfaces. Identical inputs must produce identical outputs.

## Core types

The first implementation should represent:

- `LifecycleState`: the eight states in `LIFECYCLE_STATE_MACHINE.md`;
- `Command`: typed provisioning, measurement, receipt, update, recovery, revocation, decommission, reset, and fault-injection operations;
- `PersistentState`: committed identity generation, lifecycle, versions, counters, and measurement state;
- `StagedTransition`: authenticated-but-uncommitted transition data;
- `Outcome`: success or a stable rejection/fault reason;
- `AuditEvent`: a non-secret record of the attempted transition; and
- `ReceiptClaims`: semantic claims from `EVIDENCE_SEMANTICS.md`, independent of serialization.

Invalid lifecycle states should be unrepresentable where practical. State-dependent command authorization remains explicit and testable rather than hidden in constructors.

## Persistence model

Every security-relevant persistent transition has three abstract phases:

1. `Prepared`: candidate state is complete but not authoritative;
2. `Committed`: the authoritative selector advances atomically; and
3. `Cleaned`: obsolete staging data may be erased.

Tests may interrupt execution before and after every phase. Recovery must yield the prior committed state, the next committed state, or `FAULT`. Mixed state is never accepted as operational.

## Measurement model

The Phase 1 model uses an abstract collision-resistant accumulator interface rather than selecting a digest algorithm. Each event supplies:

- domain identifier;
- event type and version;
- ordered payload commitment; and
- optional component/stage identifier.

The model records enough transcript information to verify order sensitivity, domain separation, reset behavior, and disclosure consistency.

## Receipt model

Receipt creation is permitted only when the lifecycle policy allows it. A test signer returns deterministic authentication bytes for a canonical abstract claim sequence. This permits binding and mutation tests without claiming production cryptography.

Receipt creation must be atomic with respect to any sequence value it consumes. A reset cannot issue two accepted receipts with the same supposedly unique sequence under one scope.

## Stable rejection classes

At minimum:

- `InvalidState`
- `Unauthorized`
- `InvalidTransition`
- `InvalidVersion`
- `RollbackDetected`
- `ReplayDetected`
- `MalformedInput`
- `UnsupportedProfile`
- `UnsupportedCriticalExtension`
- `PersistenceFailure`
- `CounterExhausted`
- `IntegrityFailure`
- `Decommissioned`
- `InternalInvariantViolation`

Rejections contain no protected key material or secret-derived diagnostic value.

## Required test families

1. Every allowed lifecycle transition succeeds from exactly its documented source states.
2. Every undocumented state/command pair fails closed.
3. Reset or interruption at every persistence injection point preserves atomicity.
4. Update and recovery cannot reduce protected version state without an explicit policy path.
5. Measurement permutations and cross-domain substitutions change the commitment.
6. Receipt mutation, claim omission, duplication, and profile substitution are rejected by the reference verifier model.
7. Challenge replay and monotonic-state rollback are tested separately.
8. Revoked identities never issue operational receipts.
9. Decommissioning is terminal.
10. Model exploration preserves every lifecycle invariant over bounded command sequences.

## Phase 1 exit criterion

The executable model is complete enough to advance when all state/command pairs are covered, interruption tests exercise every persistent commit point, negative receipt vectors are published, and requirements traceability identifies the exact tests supporting each implemented property.
