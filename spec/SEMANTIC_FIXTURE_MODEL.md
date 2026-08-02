# Semantic Fixture Model

Status: Phase 1 Increment 6 executable-fixture specification. This document
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

## Negative corpus

Increment 6 publishes a typed negative corpus beside the positive objects.
Each negative fixture has a stable repository identifier, a stated semantic
distinction, an evaluation operation, and one exact expected `ValidationError`.
The frozen corpus includes exactly one case for every current validation error
class. Cross-object cases deliberately use individually valid receipt,
authority, and state objects so the rejection tests the relationship rather
than an unrelated malformed operand.

The corpus is candidate-neutral: these cases contain invalid typed meanings,
not malformed CBOR, custom binary, or other candidate bytes. Candidate parsers
must later derive their own malformed vectors and stable parser-level rejection
codes without treating Rust construction as a wire-format oracle.

## Automated conformance and drift checks

`validate_corpus_conformance` pins the ordered positive-fixture manifest,
validates every positive object, checks unique fixture metadata, validates each
negative case against its exact error, and requires complete one-to-one
coverage of the stable validation-error domain.

Test-only adapters exhaustively map lifecycle-state and rejection enums between
`baochip-model` and this crate. They compare reachable lifecycle checkpoints,
map a model-issued receipt into the complete semantic release relationship, and
project `DurableModel` clean/prepared/committed phases into valid authority
metadata. The model crates are development dependencies only. No mutable state
loader or runtime coupling is introduced.

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

The conformance adapters detect enum-domain changes and drift in the public
lifecycle, receipt, counter, commit, and persistence-phase surfaces they
exercise. They do not expose every private model field and do not prove that an
unchanged public projection captures every future internal field. Model changes
still require explicit fixture and specification review; the automated gate
makes silent drift harder, not impossible.

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
