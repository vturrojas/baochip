# Phase 1 Increment 5 Adversarial Review

## Baseline and branch

- Reviewed baseline: `fa8df682fcb7515450b7664f5e565c232729c578`
- Review branch: `junior/phase1-increment5-adversarial-review`
- Baseline tree: clean `main`; local `main` and `origin/main` matched the
  reviewed commit before branch creation.
- Baseline tests: 34 lifecycle, 62 persistence, and 15 semantic-fixture tests;
  111 workspace tests total with zero failures.
- Baseline normal dependencies: the three local workspace crates only.

## Scope and exclusions

The review attacked the candidate-neutral semantic fixture crate, its
canonical specifications, corpus claims, evaluation gate, traceability, and
workspace metadata. It compared the fixture projections with the lifecycle
and persistence executable models and attempted malformed, substitution,
boundary, optionality, authority, and cross-object cases.

The review did not select or implement an encoding, canonical bytes,
cryptographic suite, integrity algorithm, storage design, host interface,
FPGA/MCU target, RTL, or badge architecture. It makes no cryptographic,
physical-durability, hardware-security, production-readiness, interoperability,
or formal-verification claim.

## Method and independent review passes

Four read-only reviewers worked independently from the same baseline and were
prohibited from editing or sharing conclusions:

| Pass | Assignment | Independent return |
|---|---|---|
| Correctness and invariants | lifecycle identity, optionality, authority phases, record/phase IDs, receipt/state binding, integer boundaries | missing receipt lineage; impossible `FAULT` identity; receipt-bearing non-operational execution; unvalidated phase commit ID |
| Security and fail-closed behavior | domain/context substitution, malformed authority, incomplete/transplanted receipts, claims, secrets, unsafe, dependencies, license | missing lineage and incomplete staged-receipt checks; impossible identity eligibility; no claims, secret, dependency, unsafe, or license defect |
| Specification/code/traceability | field-by-field spec and executable-model comparison, checklist evidence, terminology, links | impossible positive `FAULT` fixture and unpinned documented corpus coverage; retained selector/phase appraisal blocker |
| Test quality | tautologies, shared assumptions, single-field mutations, missing negative classes, coverage claims | persistent-state and error-path coverage gaps; corpus claims removable without test failure; tautological extension-type test |

The primary reviewer reproduced the semantic defects locally. Four focused RED
tests produced 15 passes and four expected failures because the validators
returned `Ok(())` for missing receipt lineage, impossible identity eligibility,
a receipt-bearing update execution, and a zero prepared commit identifier.
Corrections were then implemented and the focused crate returned green.

## Findings by severity

### Critical

None.

### High

1. **Required receipt lineage was optional in practice.** The future complete
   receipt validator accepted no key-generation or provisioning-generation
   lineage, contrary to the required anti-substitution claim.
2. **Receipt release lacked complete authoritative binding.** A receipt had no
   authoritative snapshot identifier or semantic checks against committed
   authority metadata and the selected persistent state. Cross-transaction or
   cross-state transplantation could not be detected by the fixture layer.
3. **Prepared executions accepted impossible receipts.** An applied update
   execution could carry a receipt so long as three audit fields matched, even
   though the current executable model issues receipts only for an
   `Operational` to `Operational` execution.

### Medium

1. **Authority commit identifiers were ignored.** Record presence was a
   Boolean bitmap, so `Prepared` and `Committed` phase identifiers could not be
   checked against previous, candidate, or selected records, including
   successor overflow.
2. **Lifecycle identity eligibility was not validated.** The corpus accepted
   impossible identity-active states, and its positive `FAULT` fixture
   contradicted the executable model by retaining an active identity.
3. **Documented corpus coverage was weakly evidenced.** Positive-object
   uniqueness, authority outcome coverage, receipt optionality, required-value
   errors, and important single-field mutations were not pinned. One extension
   test compared enum constructors rather than a validated fixture mutation.

### Low

None retained after correction.

## Corrections

- Added a required receipt-lineage choice supporting either key generation or
  provisioning generation and checked it against the protected subject.
- Added `authority_commit_id` to the receipt projection and semantic checks
  against matching `Committed` authority metadata.
- Added a receipt-to-authoritative-state check covering subject, commit,
  lifecycle, generation, transition counter, measurement epoch, conditional
  receipt sequence, and active version.
- Replaced the record-presence bitmap with per-slot optional commit identifiers;
  absence denotes an empty slot without a second contradictory representation.
- Required prepared and committed phase IDs to equal the candidate/selected
  record and the checked successor of the previous record.
- Required current staged receipts to describe an `Operational` to
  `Operational` execution with no staged generation and plausible nonzero
  executable-model counter/version context.
- Enforced executable lifecycle/identity eligibility, exact successor staging
  generation, and initial-versus-recommission origin constraints.
- Corrected the positive `FAULT` fixture to disable identity eligibility and
  corrected the present-zero receipt fixture to use a provisioned device
  generation.
- Added focused adversarial tests for record-ID mismatch and overflow,
  authority phase/subject transplant, authoritative-state mismatch, required
  receipt values, persistent single-field mutations, security-context
  distinctions, semantic-object uniqueness, and documented corpus coverage.
- Removed the tautological enum-only extension test; a validated fixture
  mutation now demonstrates text/byte type separation.
- Updated canonical receipt, authority, evidence, persistence, evaluation,
  test-plan, traceability, decision, README, and exit-review language.

## Adversarial cases exercised

- all eight lifecycle identities and impossible identity-active combinations;
- absent versus present-zero key generation, both required receipt-lineage
  modes, and receipt optionals;
- exact extension type, criticality, ordering, and duplicate identifiers;
- object class, profile, schema, suite, subject, device generation, key
  generation, lifecycle, and extension substitutions;
- empty required identifiers, subjects, receipt values, and commitments;
- `Clean`, prepared-applied, prepared-rejected, and `Committed` authority;
- missing, unexpected, conflicting, out-of-range, mismatched, zero, maximum,
  and overflowing slot/record/phase identifier relationships;
- non-operational staged receipts and receipt/audit disagreement;
- receipt release under the wrong phase, subject, commit, lifecycle, counter,
  epoch, sequence, or version; and
- positive fixture label and complete semantic-object uniqueness.

All implemented malformed cases return typed `ValidationError` values. The
validators are immutable inspections: they borrow inputs and do not partially
mutate the fixture on failure.

## Test totals

| Crate | Before | After |
|---|---:|---:|
| `baochip-model` | 34 | 34 |
| `baochip-persistence-model` | 62 | 62 |
| `baochip-semantic-fixtures` | 15 | 26 |
| Workspace | 111 | 122 |

The eleven-test increase consists of focused regression and corpus-quality tests;
no lifecycle or persistence behavior changed.

## Validation evidence

The final review validation requires and records:

- `cargo fmt`: completed successfully;
- `cargo fmt --check`: exit 0;
- `cargo check --workspace`: exit 0;
- `cargo check --workspace --all-features`: exit 0;
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: exit 0;
- `cargo test --workspace --all-targets`: 34 + 62 + 26 = 122 passed,
  zero failed;
- `cargo tree --workspace --edges normal`: only the three local workspace
  crates and the local persistence-to-lifecycle edge;
- `git diff --check`: exit 0;
- `git diff --cached --check`: exit 0 before commit;
- changed-file scans: no secret-like material, unsafe Rust, dependency or lock
  change, generated artifact, or unsupported security claim; and
- Apache-2.0 workspace and crate licensing remains unchanged.

## Remaining limitations and blockers

- The fixture crate is manually traced rather than generated from
  `StateMachine` and `DurableModel`; model-field drift still requires review.
- Semantic validation and cross-object equality do not authenticate data and
  do not prove parser rejection, canonicalization, collision resistance of a
  future algorithm, or interoperability.
- A logical commit identifier is not a hash or cryptographic commitment.
- Candidate-specific encoded vectors, negative byte vectors, independent
  decoders, resource measurements, and an encoding decision remain absent.
- Independent selector and phase-metadata appraisal remains a downstream
  blocker for realizing corruption recovery without circular trust.
- No cryptographic suite, physical storage behavior, durability mechanism,
  RTL, FPGA/MCU implementation, or hardware result exists.

This review does not provide physical durability or cryptographic integrity
evidence and does not select or implement an encoding, cryptographic suite, or
hardware design.
