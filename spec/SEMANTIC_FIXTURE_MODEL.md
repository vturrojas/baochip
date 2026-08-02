# Semantic Fixture Model

Status: Phase 1 Increment 5 executable-fixture specification. This document
defines the role and acceptance boundary of `baochip-semantic-fixtures`. It
does not define a protocol encoding, parser, canonical bytes, cryptographic
suite, or physical representation.

## Purpose

Every encoding candidate must start from equivalent Baochip meanings. The
fixture crate provides one dependency-free Rust representation of those
meanings so a candidate cannot quietly omit difficult fields, collapse absent
and default values, or substitute a different authority model.

The Rust representation is authoring scaffolding. In particular, Rust enum
discriminants, variant order, struct layout, `String`, `Vec`, fixture order,
and example identifier text are not protocol assignments.

## Initial supported semantic objects

The corpus contains:

- complete executable-model persistent-state projections;
- complete current authority metadata for `Clean`, `Prepared`, and
  `Committed`, including per-slot record commit identifiers and prepared
  outcomes; and
- the future execution-receipt projection required by the evidence semantics,
  clearly separated from the smaller receipt subset currently emitted by the
  lifecycle model.

Lifecycle-audit, Endorsement, and Reference Value object classes remain named
domains but do not yet have standalone fixture payloads. A candidate must not
invent those payloads or claim their support.

## Positive corpus coverage

The initial positive fixtures exercise:

- absent versus present-zero key-generation context;
- zero and maximum unsigned values;
- every current lifecycle-state identity;
- absent and present pending update state;
- absent and present provisioning generation and both provisioning origins;
- active and inactive identity state;
- both logical record slots;
- `Clean`, prepared-applied, prepared-rejected, and `Committed` authority;
- record-presence and selected/previous/candidate slot relationships;
- checked-successor relationships among previous, candidate, selected, and
  phase commit identifiers;
- complete staged `Execution` or `Rejection` distinctions;
- receipt release bound to a committed authority subject, commit identifier,
  and every overlapping authoritative persistent-state claim;
- required receipt values and absent/present receipt optionals;
- byte, text, Boolean, and unsigned extension value types; and
- critical and noncritical extensions.

Fixture identifiers are stable repository labels. Their strings are not
authenticated protocol identifiers.

## Semantic validation

The crate rejects fixture objects that contain:

- an empty required identifier or subject;
- an object payload under the wrong object class;
- a slot outside the two-slot model;
- duplicate or non-increasing extension identifiers;
- inconsistent provisioning or pending-update state;
- lifecycle/identity-eligibility combinations that the executable model cannot
  produce;
- subject generation inconsistent with the protected payload;
- a receipt whose required key-generation or provisioning-generation lineage
  contradicts its subject scope;
- missing or unexpected records for the declared authority phase;
- candidate, previous, next, or selector slot conflicts;
- phase/record commit-identifier mismatch or checked-successor overflow;
- a committed selector that does not select the declared next record;
- a prepared execution whose receipt contradicts its operational audit;
- a receipt released under a noncommitted, different-subject, different-
  commit, or contradictory authoritative-state context; or
- an empty required receipt commitment.

These are semantic fixture errors. They are not byte-parser rejection codes
and do not demonstrate that any candidate parser fails closed.

## Required candidate behavior

Each candidate prototype must:

1. identify the exact fixture-corpus commit;
2. consume every in-scope positive object without semantic omission;
3. report unsupported objects or values explicitly;
4. publish candidate-specific canonical bytes separately;
5. derive negative byte vectors from the shared semantic distinctions and the
   candidate grammar;
6. preserve object class, profile, version, suite, subject, extension set, and
   complete payload distinctions; and
7. keep integrity values outside the protected-input bytes.

The same candidate bytes must never be checked into this crate as though they
were neutral semantic truth. Candidate vectors belong in separately named,
versioned evaluation artifacts.

## Drift and limitations

The first corpus is manually traced to the executable model and canonical
record specification. It is not yet generated from `StateMachine` or
`DurableModel`, and it does not prove that future model fields cannot drift.
Any model-field change therefore requires explicit fixture and specification
review. A later adapter increment may construct fixture projections from
public model snapshots without exposing mutable state-loading APIs.

The corpus provides no negative byte vectors, parser, encoder, independent
decoder, benchmark, cryptographic mechanism, selector/phase trust split,
durability evidence, RTL, FPGA result, or hardware claim.

The authority-release check is a semantic cross-object consistency check. A
commit identifier is not a digest or cryptographic commitment, and the check
does not authenticate either object.

Receipt lineage is an explicit required choice between key generation and
provisioning generation. The fixture validator checks the selected mode against
the protected subject instead of treating optional key generation as the only
valid lineage source.
