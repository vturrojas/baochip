# Canonical Record Model

Status: Phase 1 Increment 4 design specification. This document defines the
semantic values that future encodings and integrity mechanisms must preserve
and bind. It does not define bytes, field numbers, a wire format, a hash,
signature, MAC, checksum, key hierarchy, or physical storage layout.

## Objective

Two independent implementations must be able to determine whether they are
protecting the same Baochip state without relying on Rust memory layout,
implementation-specific enum discriminants, map ordering, omitted defaults,
or undocumented host context.

Canonicality has two distinct layers:

1. **Canonical semantic projection:** the complete typed Baochip values and
   their meaning, defined here.
2. **Canonical byte representation:** one deterministic encoding of that
   projection, selected only after comparative experiments.

Agreement at the semantic layer is required before byte-level canonicalization
can be evaluated.

## Protected object classes

Every protected object has an explicit object class. Values from one class
must not verify as another even when their remaining fields happen to match.

| Semantic domain | Purpose |
|---|---|
| `baochip.persistent-state` | Complete durable lifecycle snapshot |
| `baochip.authority-metadata` | Selector and transaction authority metadata |
| `baochip.execution-receipt` | Evidence claims released after durable commit |
| `baochip.endorsement` | Future manufacturer or provisioner assertions |
| `baochip.reference-values` | Future verifier-owner appraisal inputs |

These are semantic identifiers. Their byte encoding and cryptographic domain
separation remain unselected.

## Common protected context

Every future integrity input must bind:

- object class;
- Baochip profile identifier;
- schema version;
- critical-extension set;
- integrity-suite identifier;
- identity or key-generation context when the suite requires one; and
- the complete class-specific payload.

An integrity value never protects itself. All metadata that changes how a
verifier interprets or authenticates the payload must be inside the protected
input. Unprotected transport hints may exist later but cannot override any
protected semantic value.

## Persistent-state projection

A complete persistent-state record contains:

- `slot_id`;
- logical `commit_id`;
- lifecycle state;
- `device_generation`;
- `transition_counter`;
- `measurement_epoch`;
- `receipt_sequence`;
- `active_version`;
- optional `pending_version`;
- optional `provisioning_generation`;
- optional provisioning origin;
- identity-active state; and
- any future field declared security-relevant by the lifecycle specification.

No current `StateMachine` field may be omitted merely because it is private,
zero, false, empty, derivable in normal operation, or considered an
implementation detail. A future field addition requires an explicit schema
version and compatibility decision.

## Authority-metadata projection

Authority metadata contains the information needed to distinguish `Clean`,
`Prepared`, and `Committed` recovery authority:

- transaction phase;
- raw selected slot;
- selector representation and integrity-suite context;
- prepared candidate slot, when applicable;
- previous and selected-next slots, when applicable;
- phase commit identifier, when applicable; and
- the schema and object context common to every protected object.

The eventual physical design may encode this information differently or use
redundant selectors. Any replacement must preserve the same authority
distinctions and recovery invariants.

An integrity verdict is a local appraisal result, not trusted protected data.
It MUST NOT be serialized as a claim that makes itself valid. The eventual
verification mechanism computes a verdict from the protected representation
and external trust inputs.

## Execution-receipt projection

The receipt projection incorporates the normative claims in
`EVIDENCE_SEMANTICS.md`, including:

- profile and schema version;
- device or evidence-key identity reference;
- device generation;
- lifecycle state;
- implementation and policy commitments;
- measurement commitment and scope;
- transition counter;
- measurement epoch;
- receipt sequence;
- active version;
- challenge or freshness input;
- result or output commitment when present;
- declared extension set; and
- signing or integrity-suite context.

Receipt protection cannot be reused as persistent-state protection. The
semantic object class is always bound.

## Type rules

- Protected counters and versions are unsigned 64-bit semantic integers.
- Integer byte order is an encoding decision; numeric meaning is not.
- Optional values distinguish absence from zero, false, an empty string, and
  an empty byte sequence.
- Enum variants use specification-defined names and future stable codes, never
  Rust discriminants or display strings.
- Boolean values have exactly two semantic values and no truthy aliases.
- Byte strings and text strings are distinct types.
- Text normalization is forbidden unless a specific field rule explicitly
  requires and tests it.
- Duplicate fields, duplicate map keys, multiple encodings of the same value,
  numeric overflow, and lossy coercion must fail closed.
- Unknown critical fields fail closed. Unknown noncritical fields require a
  versioned extension rule before they may be ignored.

## Canonical projection invariants

1. Equal semantic objects produce one selected canonical byte representation.
2. Different protected semantic objects never intentionally share an
   integrity input.
3. Object class, profile, version, suite, and critical extensions are bound.
4. Every security-relevant state field is represented exactly once.
5. Absence and default values cannot be confused.
6. A decoder rejects noncanonical alternatives when canonical form is
   required for integrity verification.
7. Decode followed by canonical encode is stable.
8. Integrity verification occurs over the selected canonical protected input,
   not over a parser's lossy in-memory reconstruction.

## Change control

Any change to the protected projection requires:

- a schema-version decision;
- compatibility and downgrade analysis;
- updated positive and negative vectors;
- traceability to a requirement or threat;
- independent decoder behavior review; and
- a statement about whether old verifiers must reject, understand, or safely
  ignore the change.

## Deferred decisions

- deterministic CBOR, an EAT profile, or a purpose-built binary format;
- field labels and numeric assignments;
- cryptographic suite and algorithm agility;
- key identifiers and certificate or endorsement representation;
- selector redundancy and physical atomicity;
- streaming versus buffered verification; and
- constrained `no_std` representation.
