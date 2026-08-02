# Phase 1 Increment 4 Adversarial Review

Baseline: `59cdcdfe3e03d90ddf8c23928d7ebd0aa97be305`

Review branch: `junior/phase1-increment4-adversarial-review`

## Scope

This independent review attacked the Phase 1 Increment 4 semantic inventory,
authority-metadata boundary, receipt projection, canonical type rules, domain
separation, encoding-evaluation gate, traceability, and claims discipline. It
compared the documents field-by-field with the lifecycle and persistence Rust
models and with the lifecycle, authority, counter, evidence, persistence,
integrity-recovery, executable-model, threat, and test-plan specifications.

The review did not select or implement an encoding, cryptographic suite,
serialization library, physical storage design, RTL, FPGA behavior, or hardware
security mechanism. Rust and Cargo metadata were read-only comparison sources.

## Methodology and independent assignments

The primary reviewer read every file named in the review brief, inventoried the
current Rust fields, inspected repository-wide claim language and Markdown
links, reconciled two independent read-only reviews, and made the final edits.

- **Reviewer A — semantic completeness:** independently compared lifecycle and
  persistence state, selector and phase authority, prepared outcomes, recovery
  invariants, audit provenance, and exit criteria. Reviewer A did not edit.
- **Reviewer B — security and encoding neutrality:** independently attacked
  self-authentication, domain substitution, receipt coverage, extension and
  downgrade behavior, canonical types, claims discipline, and candidate
  evaluation neutrality. Reviewer B did not edit.

Both reviewers independently found the missing prepared outcome. Reviewer A
also identified the selector/phase trust-split realizability gap; Reviewer B
independently identified receipt and extension-set ambiguity.

| Independent return | Finding evidence recorded before correction |
|---|---|
| Reviewer A | High: missing `prepared_outcome`; High: selector/phase trust split absent; Medium: lifecycle-audit boundary absent |
| Reviewer B | High: missing `prepared_outcome`; High: receipt projection incomplete; Medium: extension criticality ambiguous; Medium: receipt-sequence conditionality inconsistent |

These were separate bounded read-only assignments with no shared edits or
reviewer-to-reviewer coordination. The primary reviewer reconciled the returned
findings; this report does not represent them as executable proof.

## Findings by severity

### Critical

None. Increment 4 contains no implementation of a parser, encoding, integrity
mechanism, storage technology, or hardware boundary that could support a
Critical implementation finding.

### High

1. **The authority-metadata projection omitted `prepared_outcome`.** The Rust
   `DurableModel`, persistence invariant, commit release rule, and integrity
   recovery all treat outcome presence and absence as phase-critical state.
   Omitting it allowed two authority objects with different releasable outcomes
   to appear semantically equal.
2. **Selector-corruption recovery had no non-circular semantic trust split.**
   The executable model can mark the selector corrupted while continuing to
   trust `Prepared` or `Committed` phase metadata, but the canonical document
   placed selector and phase data in one undifferentiated authority object. A
   single verdict over that object cannot express “selector untrusted, phase
   trustworthy,” and the selector cannot authenticate the recovery metadata
   used to replace it.

### Medium

1. **Receipt coverage was incomplete and overstated current evidence.** The
   canonical list omitted `input_commitment`, used vague identity and policy
   language, did not expose key-generation/provisioning lineage, and did not
   distinguish future normative claims from the seven fields in the current
   Rust `ReceiptClaims`.
2. **Extension criticality had two possible sources of truth.** A common
   critical-extension set and receipt extensions could disagree, with no
   required rejection rule or retention rule for accepted unknown noncritical
   values.
3. **Cross-context separation was conditional and canonical type boundaries
   were incomplete.** Device, lineage, key generation, full extension content,
   unsigned bounds, enum-code stability, and lossless unknown-extension
   round trips were not all mandatory.
4. **The lifecycle authorization-audit boundary was absent.** Existing audit
   structs are not persistent-state or receipt fields, while the Authority
   Model requires a larger future authorization record. Without a separate
   object class, a later implementation could reuse another protection domain
   or overstate what current audits demonstrate.

### Low

1. **Encoding terminology and presentation could bias evaluation.** EAT, CBOR,
   CDDL, and COSE were grouped as though they performed the same function, and
   the decision log and implementation-options document called one family the
   leading candidate before prototype evidence existed.
2. **The Increment 4 status language was premature.** The README said the
   semantic boundary was frozen and the exit record claimed complete
   projections before field-by-field review evidence existed.

## Corrections

- Replaced the persistent-state list with an exact table covering both record
  context fields and all ten current `StateMachine` fields.
- Added record presence, phase-specific slot identities, phase commit ID, and
  complete withheld `prepared_outcome` semantics to authority metadata.
- Required independently protected and appraisable selector and phase
  substructures; explicitly prohibited selector self-recovery and serialized
  `IntegrityVerdict` claims.
- Identified trust anchors, accepted endorsers, reference values,
  verifier-owner policy, and verification keys as external trust inputs.
- Made subject scope and complete extension entries mandatory protected
  context for every applicable object.
- Aligned receipt field names and optionality with Evidence Semantics, added
  key-generation context and input commitment, and listed the exact subset
  demonstrated by the Rust model.
- Defined one extension-criticality source of truth and lossless retention for
  accepted unknown noncritical extensions.
- Tightened unsigned integer, enum, text, duplicate, coercion, extension, and
  round-trip rules and expanded required negative vectors.
- Clarified EAT as a claims framework, CBOR as an encoding, CDDL as a data-shape
  language, and COSE as security-envelope structures; kept three serious
  candidates without a preferred family.
- Added a separate future lifecycle-audit object boundary without claiming the
  current audits satisfy it.
- Updated decision, traceability, persistence, recovery, README, implementation
  options, and exit-review language consistently.

## Persistent-state completeness

| Current value | Projection result |
|---|---|
| logical slot identity | covered as protected record context |
| `Record.commit_id` | covered |
| `StateMachine.lifecycle` | covered |
| `device_generation` | covered |
| `transition_counter` | covered |
| `measurement_epoch` | covered |
| `receipt_sequence` | covered |
| `active_version` | covered |
| `pending_version` | covered with absence distinct from zero |
| `provisioning_generation` | covered with absence distinct from zero |
| `provisioning_origin` | covered as absent, `Initial`, or `Recommission` |
| `identity_active` | covered as an exact Boolean |

No current field is omitted because it is private, zero, false, absent,
derivable, or considered an implementation detail. Command authorizations,
validation inputs, outcomes, and audits are not falsely presented as
`StateMachine` fields; their separate boundaries are explicit.

## Authority-metadata results

`Clean`, `Prepared`, and `Committed` remain distinct. The projection now covers
the raw selected slot, record presence, candidate/previous/selected-next slots,
phase commit IDs, and exact prepared-outcome presence rules. `IntegrityVerdict`
remains an external local appraisal result, not protected serialized data.

The current model's raw selector payload is ignored under a corrupted selector
only because phase metadata is assumed trustworthy. The corrected specification
requires selector and phase data to be independently appraisable before this
rule can be realized. Ambiguous duplicate records still fail closed, and
successful recovery audit lifecycle and commit IDs remain sourced only from
trustworthy selected records.

## Circular-trust analysis

Protected-input bytes and integrity-value bytes are now explicitly distinct.
No object may authenticate itself by carrying `Valid`, a digest, a signature,
or a trust anchor inside the bytes whose trust is being decided. Appraisal uses
external trust inputs. A selector verdict cannot transitively establish the
phase metadata needed to recover from selector corruption.

The Rust model still lacks a distinct phase-metadata verdict. That is preserved
as an explicit downstream implementation blocker rather than being hidden by a
documentation claim or silently fixed outside Increment 4.

## Domain-separation analysis

The protected context binds object class, profile, schema, suite, applicable
device/lineage/key-generation subject scope, the full extension set, and the
complete payload. Separate classes cover persistent state, authority metadata,
execution receipts, lifecycle audits, endorsements, and reference values.
Negative vectors must attack substitution across every listed context. These
are semantic requirements only; no cryptographic domain-separation mechanism
exists yet.

## Receipt-projection analysis

Evidence remains distinct from Endorsements, Reference Values, Appraisal
Policy, Attestation Results, and relying-party decisions. Challenge freshness,
transition ordering, measurement epochs, and receipt sequencing have separate
scope. The receipt class cannot be reused for state protection. The document
now states exactly what the current Rust receipt demonstrates and labels the
remaining profile, measurement, policy, key, extension, input/output, encoding,
and protection claims as future requirements.

## Encoding-evaluation analysis

No candidate was selected. Three serious alternatives remain: an EAT/CWT
profile using deterministic CBOR, purpose-defined deterministic CBOR described
by CDDL, and a purpose-built deterministic binary grammar. COSE is evaluated
separately as a possible future envelope. All candidates must use identical
semantic fixtures, publish positive and negative vectors, use two genuinely
independent decoders without a shared generated parser or canonicalizer, and
measure named-target size, memory, code-size, and latency. Scoring remains
prohibited until evidence artifacts exist, and dependency and maintenance risk
remain evaluation inputs.

## Claims audit

Repository-wide searches examined language about canonical bytes,
cryptographic protection, tamper resistance, atomic storage, physical
durability, hardware and FPGA results, constrained performance, EAT/COSE
compliance, and interoperability. Reviewed occurrences were proposals,
requirements, planned evidence, explicit exclusions, or limitations. The one
premature “frozen” status statement and “leading candidate” language were
corrected. No produced byte-level, cryptographic, durability, or hardware
result is claimed.

## Validation evidence

| Command or inspection | Observed result |
|---|---|
| `cargo fmt --check` | PASS, exit 0 |
| `cargo check --workspace --all-targets` | PASS, exit 0 |
| `cargo check --workspace --all-targets --all-features` | PASS, exit 0 |
| `cargo clippy --workspace --all-targets --all-features -- -D warnings` | PASS, exit 0, no warnings |
| `cargo test --workspace --all-targets` | PASS: 34 lifecycle and 62 persistence tests; 96 total, 0 failed |
| `cargo tree --workspace --edges normal` | PASS: only the two local workspace crates; no third-party normal dependency |
| `git diff --check` | PASS, exit 0 |
| `git diff --cached --check` | PASS, exit 0 |
| local Markdown-link inspection | PASS: every changed local target exists |
| trailing-whitespace scan | PASS: no finding |
| secret-like material scan | PASS: no finding |
| unsafe-Rust scan | PASS: no unsafe block; crate-level forbids remain unchanged |
| Cargo/dependency inspection | PASS: no manifest or lockfile change |
| generated-artifact and intended-scope inspection | PASS: only intended Markdown files |

The test totals are unchanged from baseline. This documentation-only review
adds no tests and changes no Rust or Cargo artifact.

## Remaining limitations

- No canonical byte representation, field labels, numeric enum assignments,
  parser, encoder, or vector corpus exists.
- No integrity or cryptographic suite, key hierarchy, selector layout, or
  physical storage mechanism is selected.
- No constrained-resource, interoperability, physical durability, tamper,
  side-channel, RTL, FPGA, or hardware result exists.
- The full future receipt and lifecycle-audit projections are not executable.
- Unknown noncritical extension value syntax remains an encoding decision even
  though lossless retention is now a semantic requirement.

## Remaining blockers

There is no blocker to committing this corrected documentation review. One
downstream implementation blocker remains: selector-corruption recovery cannot
be realized until selector and phase metadata have independently appraisable,
non-circular protection. Encoding selection also remains blocked on comparable
prototype evidence, versioned vectors, independent decoders, measured costs,
and a recorded tradeoff decision.

**This review selected and implemented no encoding, cryptographic suite,
integrity mechanism, physical storage technology, RTL, FPGA behavior, or
hardware security result. It is not evidence of canonical bytes,
cryptographic integrity, physical durability, or hardware security.**
