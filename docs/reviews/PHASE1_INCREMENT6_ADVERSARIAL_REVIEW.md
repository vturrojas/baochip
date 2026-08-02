# Phase 1 Increment 6 Adversarial Review

## Baseline and review branch

- Reviewed baseline: `6d3357b770341b346bf63eb0025b71f956a6453c`
- Previous reviewed baseline: `4dfc83a`
- Review branch: `junior/phase1-increment6-adversarial-review`
- Final review commit: this report's local commit, titled
  `Harden Baochip Increment 6 semantic conformance gates [skip ci]`; its SHA is
  reported separately because a commit cannot contain its own identifier.
- Baseline state: clean `main`; local `main` and `origin/main` matched the
  reviewed baseline before branch creation.
- Baseline tests: 34 lifecycle, 62 persistence, and 33 semantic-fixture and
  conformance tests; 129 workspace tests total with zero failures.

## Scope and methodology

The review attacked the complete `4dfc83a..6d3357b` Increment 6 diff and its
interaction with lifecycle, rejection, persistence, recovery, authority,
receipt-release, and semantic-fixture behavior. The primary reviewer inspected
the source, specifications, Cargo metadata, claims, and test construction;
performed mutation and substitution attacks; reproduced confirmed defects with
failing tests; implemented corrections centrally; and reran the full workspace
gate.

Three read-only reviewers worked independently and were prohibited from
editing, committing, pushing, merging, or coordinating conclusions:

| Assignment | Independent findings |
|---|---|
| Correctness and model conformance | public gate omitted operand/global-identifier enforcement; complete `ReceiptRelease` case unused; recovered and rejected model projections absent |
| Security and adversarial state | shared protected context was not cross-object bound; corpus construction could panic; complete three-object release case unused |
| Specification, traceability, and claims | negative corpus was less frozen than documented; Increment 6 exit status was stale; no unsupported implementation claim found |

The primary pass independently found that a receipt and matching
non-`Operational` state could pass the complete release API. Findings were
reconciled against executable evidence rather than accepted solely from a
reviewer report.

## Findings by severity

### Critical

None.

### High

1. **Shared protected context was not bound across release objects.** A valid
   receipt with a substituted profile, schema version, integrity suite, or
   extension set could be released against otherwise matching authority and
   state objects. Only subject and commit/payload fields were compared.
2. **Non-Operational receipt release was accepted.** A standalone receipt and
   persistent state could both be changed to `Recovery` and still satisfy the
   three-object release relationship, although the executable lifecycle model
   issues receipts only from `Operational`.

### Medium

1. **The exported conformance gate under-enforced its contract.** It did not
   pin negative identifiers, check identifier collisions across both corpora,
   or independently validate cross-object operands before relationship
   evaluation.
2. **Negative-corpus construction could panic on drift.** Public construction
   used `expect` and `panic` for missing or reshaped positive operands, so the
   supposedly fail-closed conformance API could abort instead of returning a
   typed error.
3. **The complete release variant was not in the frozen corpus.** The boxed
   `NegativeCase::ReceiptRelease` representation compiled cleanly and had
   transparent clone/equality/match behavior, but no fixture constructed it;
   the final selector-to-state slot relationship was outside the conformance
   corpus.
4. **Recovery and rejected-outcome adapter evidence was incomplete.** The
   initial adapter covered nominal clean, prepared, committed, and cleanup
   states but did not project prepared-crash recovery, committed-crash
   recovery, or the persistence model's durable rejected-to-`FAULT` path.
5. **Negative-corpus documentation overstated payload freezing.** Identifier
   and error coverage can be pinned without inventing a byte representation,
   but heap-backed typed case payloads are not independently fingerprinted.
6. **Lifecycle checkpoint wording exceeded adapter coverage.** Enum mappings
   are exhaustive, but executable snapshot comparisons cover selected Blank,
   initial Provisioning, and Operational receipt-release checkpoints rather
   than every reachable lifecycle snapshot.

### Low

1. **Exit status was stale.** The Increment 6 baseline was already merged to
   `main`, while the exit review still described that baseline merge as
   pending.

## Corrections and regression tests

- Required equality of profile, schema version, integrity suite, subject, and
  complete extension set across receipt, authority metadata, and persistent
  state while preserving distinct object classes.
- Required the selected persistent state to be `Operational` before receipt
  release.
- Pinned the ordered negative identifier manifest, retained exact one-to-one
  `ValidationError` coverage, checked positive/negative identifier collisions,
  and moved independent operand validation into the exported conformance gate.
- Made `negative_fixtures` fallible with typed `NegativeCorpusError` variants;
  conformance now propagates construction drift instead of panicking.
- Routed the pinned state-context negative through the complete boxed
  `ReceiptRelease` operation with three independently valid operands.
- Added real persistence-model projections for recovered previous authority,
  recovered next authority, and a fault-producing rejected prepare. The
  lifecycle `test-support` feature is enabled only in fixture-crate development
  tests and remains off in ordinary builds.
- Corrected canonical record, evidence, semantic-corpus, model-test, crate
  README, and exit-review wording to match the implemented evidence.

The RED evidence was explicit. After the first test-only patch, the crate did
not compile because negative-manifest and operand gates did not exist. After
adding only those gates, 35 tests passed and three failed: shared-context
substitution returned `Ok(())`, non-Operational release returned `Ok(())`, and
the corpus contained zero complete `ReceiptRelease` cases. The minimal
behavioral corrections made all focused tests pass.

## Adversarial cases exercised

- exact 17-entry positive and negative identifier manifests;
- exact one-to-one coverage of all 17 stable validation errors;
- pairwise-unique negative cases and globally unique identifiers;
- independently valid receipt, authority, and state operands;
- profile, schema, suite, extension, subject, generation, commit, selector,
  phase, slot, lifecycle, counter, epoch, sequence, and version substitutions;
- complete three-object release with authority-first error precedence;
- non-Operational and merely prepared receipt-release attempts;
- prepared and committed crash recovery authority;
- fault-producing rejected prepare with the prior `Operational` state still
  authoritative until selector commit;
- checked commit successors and counter-exhaustion behavior; and
- public corpus-construction drift without panic.

## Test totals

| Crate | Before | After |
|---|---:|---:|
| `baochip-model` | 34 | 34 |
| `baochip-persistence-model` | 62 | 62 |
| `baochip-semantic-fixtures` | 33 | 40 |
| Workspace | 129 | 136 |

The seven-test increase consists of focused release-security, conformance-drift,
complete-case, recovery, and rejected-outcome regressions. Lifecycle and
persistence production behavior did not change.

## Validation evidence

Final local validation completed successfully:

- `cargo fmt`: exit 0;
- `cargo fmt --check`: exit 0;
- `cargo check --workspace`: exit 0;
- `cargo check --workspace --all-features`: exit 0;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0;
- `cargo test --workspace --all-targets`: 34 + 62 + 40 = 136 passed,
  zero failed;
- `cargo tree --workspace --edges normal`: only the three local workspace
  crates and their local edges;
- `git diff --check` and `git diff --cached --check`: exit 0;
- changed Markdown links and trailing whitespace: clean;
- secret-like material, `unsafe` blocks, unfinished-work markers, unsupported claims,
  generated artifacts, and accidental external dependencies: none introduced;
  and
- Apache-2.0 workspace and crate licensing: unchanged.

## Remaining limitations

- The adapters exercise selected public surfaces; they are not a formal or
  field-complete equivalence proof for private model state.
- Typed case payloads remain reviewed semantic Rust values. They are not
  canonical bytes or independently fingerprinted serialized vectors.
- Semantic equality and a logical commit identifier do not authenticate an
  object or prove collision resistance.
- No parser, canonical representation, independent decoder, malformed byte
  corpus, encoding choice, cryptographic suite, integrity mechanism, physical
  storage behavior, durability evidence, constrained-resource measurement,
  interoperability result, RTL, FPGA implementation, badge architecture, or
  hardware component exists.
- Independent selector and phase-metadata appraisal remains unresolved.
- Merge of this adversarial correction commit remains pending explicit owner
  authorization.

This review claims no encoding, cryptographic, durability, interoperability,
RTL, FPGA, badge, or hardware result.
