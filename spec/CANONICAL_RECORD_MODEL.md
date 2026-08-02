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
| `baochip.lifecycle-audit` | Future protected authorization and transition audit records |
| `baochip.endorsement` | Future manufacturer or provisioner assertions |
| `baochip.reference-values` | Future verifier-owner appraisal inputs |

These are semantic identifiers. Their byte encoding and cryptographic domain
separation remain unselected.

## Common protected context

Every future integrity input must bind:

- object class;
- Baochip profile identifier;
- schema version;
- the complete extension set, including each extension identifier, criticality,
  type, and value;
- integrity-suite identifier;
- the subject scope needed to prevent cross-device, cross-lineage, or
  cross-key-generation substitution; and
- the complete class-specific payload.

An integrity value never protects itself. All metadata that changes how a
verifier interprets or authenticates the payload must be inside the protected
input. Unprotected transport hints may exist later but cannot override any
protected semantic value.

Trust anchors, accepted endorsers, reference values, verifier-owner policy,
and any key material used to appraise an integrity input are external trust
inputs. They are not authenticated by a field inside the object being
appraised. An integrity value is stored or transported separately from the
protected-input bytes and is never included in the bytes it covers.

## Persistent-state projection

A complete persistent-state record contains the following projection. The
`slot_id` is logical record context in the two-slot model rather than a field
inside the current Rust `Record`; it is still bound so a record cannot be
substituted between slots.

| Semantic field | Current model source | Required meaning |
|---|---|---|
| `slot_id` | logical slot containing `Record` | slot identity under the authority-metadata profile |
| `commit_id` | `Record.commit_id` | logical identifier of the complete snapshot |
| `lifecycle_state` | `StateMachine.lifecycle` | one of the eight lifecycle states |
| `device_generation` | `StateMachine.device_generation` | physical trust-root lineage generation |
| `transition_counter` | `StateMachine.transition_counter` | lifecycle-transition order within the generation |
| `measurement_epoch` | `StateMachine.measurement_epoch` | measurement-session scope |
| `receipt_sequence` | `StateMachine.receipt_sequence` | receipt order within its declared identity and generation scope |
| `active_version` | `StateMachine.active_version` | authoritative implementation version |
| `pending_version` | `StateMachine.pending_version` | absent or the staged update version; absence is not zero |
| `provisioning_generation` | `StateMachine.provisioning_generation` | absent or the staged next generation; absence is not zero |
| `provisioning_origin` | `StateMachine.provisioning_origin` | absent, `Initial`, or `Recommission` |
| `identity_active` | `StateMachine.identity_active` | exact Boolean identity-eligibility state |

Command authorizations, update-validation inputs, executions, and audit events
are not fields of `StateMachine` and are not silently added to this snapshot.
Where another protected object retains one of them, that object's projection
must bind it explicitly.

No current `StateMachine` field may be omitted merely because it is private,
zero, false, empty, derivable in normal operation, or considered an
implementation detail. A future field addition requires an explicit schema
version and compatibility decision.

## Authority-metadata projection

Authority metadata contains the information needed to distinguish `Clean`,
`Prepared`, and `Committed` recovery authority:

| Semantic field | Phase rule |
|---|---|
| transaction phase | exactly `Clean`, `Prepared`, or `Committed` |
| raw selected slot | present for every phase; it is authoritative only after its selector appraisal succeeds |
| record-presence bitmap | identifies which of the two logical slots contain complete records |
| prepared candidate slot | present only in `Prepared` and distinct from the previous-authority slot |
| previous slot | present only in `Committed` and distinct from the selected-next slot |
| selected-next slot | present only in `Committed` and equal to the selected slot after selector commit |
| phase `commit_id` | absent in `Clean`; present in `Prepared` and `Committed` |
| `prepared_outcome` | present only in `Prepared`; absent in `Clean` and `Committed` |

The protected prepared outcome binds a stable `Applied` or `Rejected`
discriminator and the complete staged `Execution` or `Rejection` value. It is
withheld from callers until selector commit and discarded on recovery to the
previous record. A future schema may replace the complete staged value with a
binding reference only if release reconstructs exactly the same outcome and
cannot confuse outcomes across transactions.

The eventual physical design may encode this information differently or use
redundant selectors. Any replacement must preserve the same authority
distinctions and recovery invariants.

An integrity verdict is a local appraisal result, not trusted protected data.
It MUST NOT be serialized as a claim that makes itself valid. The eventual
verification mechanism computes a verdict from the protected representation
and external trust inputs.

Selector-corruption recovery also requires a non-circular trust split. The
selector representation and the phase metadata used when that selector is
untrusted must be independently protected and independently appraisable; one
integrity result over an indivisible authority object cannot establish
"selector corrupted, phase metadata trustworthy." Phase metadata cannot be
authenticated by the selector whose authority it is used to replace. The
encoding, redundancy, and physical realization of this split remain deferred.

## Execution-receipt projection

The future receipt projection incorporates every normative claim in
`EVIDENCE_SEMANTICS.md`: `profile`, `schema_version`, `integrity_suite`,
`key_id`, key-generation or provisioning-lineage context, `lifecycle_state`,
`device_generation`, `transition_counter`, `measurement_epoch`, conditional
`receipt_sequence`, `active_version`, conditional `challenge`,
`measurement_root`, `measurement_context`, `policy_id`, `policy_version`,
optional `input_commitment`, optional `output_commitment`, and the complete
extension set.

The current Rust `ReceiptClaims` demonstrates only `lifecycle_state`,
`device_generation`, `transition_counter`, `measurement_epoch`,
`receipt_sequence`, `active_version`, and optional `challenge`. It does not yet
demonstrate profiles, schemas, suites, keys or endorsements, measurement or
policy commitments, input/output commitments, extensions, canonical bytes, or
cryptographic protection. Those missing executable artifacts do not disappear
from the future semantic projection.

Receipt protection cannot be reused as persistent-state protection. The
semantic object class is always bound.

## Lifecycle-audit boundary

The current `AuditEvent` and `PersistenceAudit` values are non-secret research
observations, not protected protocol records. If a future profile protects or
exports them, `baochip.lifecycle-audit` is a separate object class and must
include the current prior/resulting lifecycle, generation, staged generation,
transition counter, persistence operation, and commit identifier fields, plus
the transition type and version, authorizing role identifiers, authorization
policy identifier and version, affected commitments, and stable result or
rejection class required by `AUTHORITY_MODEL.md`. Increment 4 does not claim
that this future audit schema or its protection has been implemented.

## Type rules

- Protected counters and versions are unsigned 64-bit semantic integers in
  the inclusive range `0..=2^64-1`; negative, out-of-range, wrapped, truncated,
  or saturated representations fail closed.
- Integer byte order is an encoding decision; numeric meaning is not.
- Optional values distinguish absence from zero, false, an empty string, and
  an empty byte sequence.
- Enum variants use specification-defined identities and, before byte encoding
  begins, assigned stable codes; Rust discriminants, display strings, aliases,
  and case folding are never protocol definitions.
- Boolean values have exactly two semantic values and no truthy aliases.
- Byte strings and text strings are distinct types.
- Text normalization is forbidden unless a specific field rule explicitly
  requires and tests it.
- Duplicate fields, duplicate map keys, multiple encodings of the same value,
  numeric overflow, and lossy coercion must fail closed.
- Extension criticality has one source of truth: the criticality carried by
  each extension entry. A separate derived critical-extension index, if an
  encoding uses one, must match exactly or decoding fails.
- Unknown critical fields fail closed. Unknown noncritical fields require a
  versioned rule for type-safe retention and canonical re-encoding before they
  may be ignored semantically; dropping them from a protected object is not an
  allowed round trip.
- Profile, schema-version, suite, and extension negotiation has no implicit
  default or downgrade. An implementation accepts only an explicitly supported
  combination; it does not reinterpret an unsupported newer or older object as
  a supported version.
- Extensions cannot redefine mandatory core fields, their types, or their
  absence rules.

## Canonical projection invariants

1. Equal semantic objects produce one selected canonical byte representation.
2. Different protected semantic objects never intentionally share an
   integrity input.
3. Object class, profile, version, suite, subject scope, and the complete
   extension set are bound.
4. Every security-relevant state field is represented exactly once.
5. Absence and default values cannot be confused.
6. A decoder rejects noncanonical alternatives when canonical form is
   required for integrity verification.
7. Decode followed by canonical encode is stable.
8. Integrity verification occurs over the selected canonical protected input,
   not over a parser's lossy in-memory reconstruction.

## Executable fixture corpus

The initial executable representation of these semantic values is maintained
in `crates/baochip-semantic-fixtures` and specified by
[Semantic Fixture Model](SEMANTIC_FIXTURE_MODEL.md). Its Rust types and example
identifiers are fixture-authoring scaffolding, not selected protocol types,
numeric assignments, or canonical bytes.

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
