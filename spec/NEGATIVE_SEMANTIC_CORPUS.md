# Negative Semantic Corpus

Status: Phase 1 Increment 6 candidate-neutral rejection specification. This
document defines invalid typed meanings and expected semantic errors. It does
not define malformed protocol bytes, parser behavior, canonical encoding, or
cryptographic verification.

## Contract

Every case contains:

- a stable repository fixture identifier;
- a concise explanation of the violated semantic distinction;
- either one invalid semantic object or a relationship among individually
  valid receipt, authority, and persistent-state objects; and
- one exact expected `ValidationError`.

The initial corpus contains exactly one pinned case for each current stable
validation error. `validate_corpus_conformance` rejects missing, duplicated,
unexpected, or differently classified cases.

## Covered rejection domains

| Domain | Stable semantic errors |
|---|---|
| Required identity and values | `EmptyIdentifier`, `EmptySubject`, `EmptyRequiredValue` |
| Object and slot domain | `WrongObjectClass`, `InvalidSlot` |
| Authority record topology | `MissingRecord`, `UnexpectedRecord`, `SlotConflict`, `SelectorMismatch` |
| Extension set | `DuplicateExtension`, `UnorderedExtensions` |
| Commit relationship | `CommitIdMismatch` |
| Cross-object release | `AuthorityPhaseMismatch`, `AuthorityContextMismatch`, `StateContextMismatch` |
| State and execution consistency | `InconsistentState`, `InconsistentExecution` |

An encoding candidate must preserve these distinctions, but it will have
additional byte-level rejection classes for truncation, noncanonical integers,
duplicate fields, invalid grammar, resource limits, and related parser input.
Those future candidate vectors must not be added to this corpus as though they
were neutral semantic truth.

## Acceptance rules

1. Every positive fixture validates successfully.
2. Every negative fixture fails with its exact pinned semantic error.
3. Fixture identifiers and cases are unique.
4. Cross-object negative operands validate independently before their
   relationship is evaluated.
5. Every stable `ValidationError` appears exactly once in the frozen negative
   manifest.
6. Adding or removing a validation error, positive fixture, or negative case
   requires an explicit conformance-manifest and specification update.

## Non-results

Passing this corpus does not show that a parser rejects malformed bytes, that
an encoder is canonical, that two implementations interoperate, or that an
integrity mechanism authenticates any object. It provides a versioned semantic
rejection contract for later candidate experiments.
