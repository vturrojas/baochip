# Baochip

Baochip is a design-phase research project exploring a small, inspectable hardware root of trust for verifiable computation and evidence provenance.

The project asks a narrow question: what is the smallest useful hardware/software boundary that can produce evidence about *what ran, under which policy, and over which inputs* without pretending that hardware alone makes a larger system trustworthy?

## Status

Baochip is currently a research scaffold. There is no fabricated chip, production implementation, security certification, or validated benchmark result. Specifications, interfaces, and experiments will change as the threat model becomes concrete.

## Initial scope

- Measured boot and signed device identity
- Monotonic counters and rollback evidence
- Signed execution receipts bound to code, policy, and input commitments
- A minimal host protocol and reference verifier
- Explicit failure modes, trust assumptions, and recovery behavior
- Reproducible simulation before any FPGA or silicon work

## Explicit non-goals

- Claiming that a secure element makes an entire host trustworthy
- Hiding unverifiable policy behind a proprietary service
- Designing a general-purpose CPU
- Treating remote attestation as proof that arbitrary output is correct
- Making production-readiness claims before independent evaluation

## Planned repository shape

```text
docs/          architecture, protocol, threat model, research notes
spec/          versioned machine- and human-readable specifications
rtl/           future reference RTL experiments
sim/           future simulation harnesses and test vectors
verifier/      future host-side receipt verifier
```

## Executable model

Phase 1 begins with a dependency-free Rust lifecycle model in [`crates/baochip-model`](crates/baochip-model). It models state transitions, authorization, protected counter scopes, update/recovery behavior, revocation/recommissioning, decommissioning, measurement epochs, and abstract receipt eligibility. It does not yet implement serialization, cryptography, persistence hardware, or RTL.

Atomic state-change experiments are isolated in [`crates/baochip-persistence-model`](crates/baochip-persistence-model), which models complete-record preparation, selector commit, cleanup, and interruption recovery. See the [Persistence Model](spec/PERSISTENCE_MODEL.md). It is an abstract transaction model, not a durability claim about any storage technology.

Phase 1 Increment 3 adds policy for abstract corrupted-record and corrupted-selector verdicts, plus typed test-only fault injection for otherwise unreachable fail-closed paths. See the [Integrity Recovery Model](spec/INTEGRITY_RECOVERY_MODEL.md). No checksum, digest, signature, serialization, or physical fault detector has been selected or implemented.

Phase 1 Increment 4 defines the [Canonical Record Model](spec/CANONICAL_RECORD_MODEL.md) and an [Encoding Evaluation Plan](docs/ENCODING_EVALUATION_PLAN.md). It freezes the semantic comparison boundary for future prototypes without selecting canonical bytes, an encoding family, or a cryptographic suite.

Local validation:

```text
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets
```

## Relationship to Surety

[Surety](https://github.com/vturrojas/surety) studies evidence and assurance for software systems. Baochip is a possible hardware-backed evidence source within that broader research direction; it is not a prerequisite for Surety and does not validate Surety's claims.

## Research discipline

Contributions should separate proposals, implementations, experiments, and results. Results must identify the exact artifact, configuration, threat model, and reproduction procedure used.

Start with the [Project Charter](PROJECT_CHARTER.md), then review the [Research Program](docs/RESEARCH_PROGRAM.md), [Prior-Art Matrix](docs/PRIOR_ART_MATRIX.md), [Threat Model](docs/THREAT_MODEL.md), [Threat Traceability](docs/TRACEABILITY_MATRIX.md), [Security Requirements](spec/SECURITY_REQUIREMENTS.md), [Lifecycle State Machine](spec/LIFECYCLE_STATE_MACHINE.md), [Authority Model](spec/AUTHORITY_MODEL.md), [Counter Model](spec/COUNTER_MODEL.md), [Evidence Semantics](spec/EVIDENCE_SEMANTICS.md), [Canonical Record Model](spec/CANONICAL_RECORD_MODEL.md), [Trust Model](docs/TRUST_MODEL.md), [Executable Model Specification](spec/EXECUTABLE_MODEL.md), [Persistence Model](spec/PERSISTENCE_MODEL.md), [Integrity Recovery Model](spec/INTEGRITY_RECOVERY_MODEL.md), [Encoding Evaluation Plan](docs/ENCODING_EVALUATION_PLAN.md), [Model Test Plan](docs/MODEL_TEST_PLAN.md), [Implementation Options](docs/IMPLEMENTATION_OPTIONS.md), [Architecture](docs/ARCHITECTURE.md), [Decision Log](DECISIONS.md), [Phase 0 Exit Review](docs/PHASE0_EXIT_REVIEW.md), [Increment 4 Exit Review](docs/PHASE1_INCREMENT4_EXIT_REVIEW.md), and [Roadmap](ROADMAP.md).

## License

Apache License 2.0. Hardware-description artifacts may later adopt an explicitly documented hardware license if the project reaches that stage.
