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

## Relationship to Surety

[Surety](https://github.com/vturrojas/surety) studies evidence and assurance for software systems. Baochip is a possible hardware-backed evidence source within that broader research direction; it is not a prerequisite for Surety and does not validate Surety's claims.

## Research discipline

Contributions should separate proposals, implementations, experiments, and results. Results must identify the exact artifact, configuration, threat model, and reproduction procedure used.

Start with the [Project Charter](PROJECT_CHARTER.md), then review the [Research Program](docs/RESEARCH_PROGRAM.md), [Prior-Art Matrix](docs/PRIOR_ART_MATRIX.md), [Threat Model](docs/THREAT_MODEL.md), [Threat Traceability](docs/TRACEABILITY_MATRIX.md), [Security Requirements](spec/SECURITY_REQUIREMENTS.md), [Lifecycle State Machine](spec/LIFECYCLE_STATE_MACHINE.md), [Authority Model](spec/AUTHORITY_MODEL.md), [Counter Model](spec/COUNTER_MODEL.md), [Evidence Semantics](spec/EVIDENCE_SEMANTICS.md), [Trust Model](docs/TRUST_MODEL.md), [Executable Model Specification](spec/EXECUTABLE_MODEL.md), [Model Test Plan](docs/MODEL_TEST_PLAN.md), [Implementation Options](docs/IMPLEMENTATION_OPTIONS.md), [Architecture](docs/ARCHITECTURE.md), [Decision Log](DECISIONS.md), [Phase 0 Exit Review](docs/PHASE0_EXIT_REVIEW.md), and [Roadmap](ROADMAP.md).

## License

Apache License 2.0. Hardware-description artifacts may later adopt an explicitly documented hardware license if the project reaches that stage.
