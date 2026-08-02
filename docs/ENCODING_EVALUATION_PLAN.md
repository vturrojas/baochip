# Encoding Evaluation Plan

Status: Phase 1 Increment 4 decision gate. No encoding candidate is selected by
this document.

## Question

Which representation can encode the canonical Baochip semantic projections
deterministically, fail closed under adversarial input, remain independently
implementable, and fit a future constrained evidence source without allowing
the encoding library to define security semantics?

## Candidates

The first comparison retains three serious candidates:

1. An EAT claims profile represented as a CWT using deterministic CBOR, with a
   separately evaluated COSE envelope for future protection.
2. Purpose-defined deterministic CBOR described by CDDL, with a separately
   evaluated COSE envelope only after the protected payload is stable.
3. A purpose-built deterministic binary format with an explicit grammar and
   independent parser.

JSON, generic Protocol Buffers, native Rust serialization, and implementation
memory layout may be retained as comparison controls, but they are not leading
candidates without evidence that they satisfy canonicality and constrained
verification requirements.

EAT is a claims framework, CBOR is an encoding, CDDL describes data shape, and
COSE provides security-envelope structures. None supplies a Baochip profile,
canonicalization policy, semantic domain separation, or acceptance policy by
itself. The custom binary candidate likewise receives no presumption from
small size or implementation control.

## Evaluation dimensions

| Dimension | Weight | Evidence required |
|---|---:|---|
| Semantic fidelity | 20 | Every canonical projection field and distinction round-trips without coercion |
| Deterministic canonical form | 15 | One byte representation per semantic value; noncanonical alternatives rejected |
| Fail-closed parsing | 15 | Mutation corpus, overflow cases, duplicates, truncation, unknown-critical tests |
| Integrity binding | 10 | Protected headers and payload cover domain, profile, version, suite, and extensions |
| Extension and downgrade safety | 10 | Version negotiation and critical-extension negative vectors |
| Independent implementability | 10 | Two implementations agree on vectors without shared encoder code |
| Constrained cost | 10 | Measured encoded size, peak stack and heap, parser/canonicalizer code size, and latency on a named target |
| Ecosystem interoperability | 10 | Standards alignment, maintained tooling, offline verification, and reviewability |

No candidate receives a numeric score until its evidence artifact exists.

## Required prototype artifact

Each candidate prototype must use the same frozen semantic fixtures and
produce:

- an adapter from the versioned `baochip-semantic-fixtures` corpus, with any
  unsupported semantic value reported rather than silently omitted;
- evidence that every candidate-neutral negative semantic case is preserved as
  a rejection distinction before candidate-specific malformed bytes are added;
- preservation of the fixture corpus's semantic cross-object bindings,
  including receipt release against committed authority metadata;
- canonical encodings for boundary values and every optional-field state;
- decode and re-encode stability results;
- explicit rejection codes for malformed inputs;
- mutations covering truncation, duplicate fields, reordered fields,
  alternate integer widths, overflow, invalid enums, invalid booleans,
  invalid text, normalization differences, absent-versus-default confusion,
  unknown critical fields, retained unknown noncritical fields, extension
  criticality mismatch, unsupported-version downgrade attempts, trailing data,
  resource-limit violations, and domain
  substitution across object classes, devices, key generations, profiles,
  schema versions, suites, and extension sets;
- protected-input bytes separated from any integrity value;
- encoded size, peak stack and heap, code-size, and latency measurements on a
  named toolchain and target;
- parser and verifier dependency inventory;
- reproduction commands and tool versions; and
- a claim ledger stating exactly what the prototype demonstrates.

## Differential requirement

Before selection, at least two independently implemented decoders must agree
on all positive vectors and stable rejection classes. They must not share the
same generated parser, canonicalizer, schema-generated decoding core, or
unchecked reference library. Merely wrapping the same implementation in a
second language does not satisfy independence.

One implementation may be the Rust reference path. The second implementation
language and toolchain remain open.

## Decision gate

An encoding may be selected only when:

- the canonical semantic projection is frozen for the evaluated version;
- at least two candidates have comparable prototype evidence;
- positive and negative vectors are versioned in the repository;
- ambiguity and downgrade findings are resolved or explicitly accepted;
- constrained costs are measured rather than estimated;
- dependency and maintenance risks are recorded;
- the decision log identifies rejected alternatives and tradeoffs; and
- independent review finds no unsupported cryptographic or hardware claim.

If no candidate passes, Baochip keeps the encoding decision open.

The Rust fixture crate is an input oracle for comparison only. Passing its
positive validators, negative semantic corpus, and model-conformance checks
does not satisfy parser, canonicalization, differential, cryptographic,
interoperability, or constrained-cost evidence requirements.

## Explicit non-results

Standards alignment alone does not prove canonicality, security, suitability,
or correctness. Small encoded size alone does not justify a custom format.
Successful round trips alone do not demonstrate rejection safety. Library
availability alone does not satisfy independent implementability.
