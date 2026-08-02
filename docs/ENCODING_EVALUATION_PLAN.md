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

1. A defined EAT profile using deterministic CBOR/CWT concepts and COSE for
   future protection.
2. Purpose-defined deterministic CBOR with CDDL, using COSE only after the
   protected payload is stable.
3. A purpose-built deterministic binary format with an explicit grammar and
   independent parser.

JSON, generic Protocol Buffers, native Rust serialization, and implementation
memory layout may be retained as comparison controls, but they are not leading
candidates without evidence that they satisfy canonicality and constrained
verification requirements.

## Evaluation dimensions

| Dimension | Weight | Evidence required |
|---|---:|---|
| Semantic fidelity | 20 | Every canonical projection field and distinction round-trips without coercion |
| Deterministic canonical form | 15 | One byte representation per semantic value; noncanonical alternatives rejected |
| Fail-closed parsing | 15 | Mutation corpus, overflow cases, duplicates, truncation, unknown-critical tests |
| Integrity binding | 10 | Protected headers and payload cover domain, profile, version, suite, and extensions |
| Extension and downgrade safety | 10 | Version negotiation and critical-extension negative vectors |
| Independent implementability | 10 | Two implementations agree on vectors without shared encoder code |
| Constrained cost | 10 | Measured encoded size, parser memory, code size, and verification latency |
| Ecosystem interoperability | 10 | Standards alignment, maintained tooling, offline verification, and reviewability |

No candidate receives a numeric score until its evidence artifact exists.

## Required prototype artifact

Each candidate prototype must use the same frozen semantic fixtures and
produce:

- canonical encodings for boundary values and every optional-field state;
- decode and re-encode stability results;
- explicit rejection codes for malformed inputs;
- mutations covering truncation, duplicate fields, reordered fields,
  alternate integer widths, overflow, invalid enums, invalid booleans,
  absent-versus-default confusion, unknown critical fields, and domain
  substitution;
- protected-input bytes separated from any integrity value;
- encoded size and peak-memory measurements;
- parser and verifier dependency inventory;
- reproduction commands and tool versions; and
- a claim ledger stating exactly what the prototype demonstrates.

## Differential requirement

Before selection, at least two independently implemented decoders must agree
on all positive vectors and stable rejection classes. They must not share the
same generated parser, canonicalizer, or unchecked reference library.

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

## Explicit non-results

Standards alignment alone does not prove canonicality, security, suitability,
or correctness. Small encoded size alone does not justify a custom format.
Successful round trips alone do not demonstrate rejection safety. Library
availability alone does not satisfy independent implementability.
