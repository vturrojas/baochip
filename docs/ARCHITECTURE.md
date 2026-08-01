# Architecture Sketch

This document is a proposal, not an implemented architecture.

## Components

1. **Immutable root** — establishes the first measurement and verifies the mutable firmware image.
2. **Protected identity** — holds or derives a device signing key without exposing it to the host.
3. **Measurement registers** — commit to ordered boot or workload events.
4. **Monotonic state** — supports rollback and replay detection under documented persistence assumptions.
5. **Receipt engine** — signs a canonical statement binding measurements, policy, counter state, challenge, and protocol version.
6. **Host transport** — an untrusted interface that moves commands and evidence.
7. **Reference verifier** — validates syntax, trust chain, freshness, policy, and counter expectations.

## Receipt envelope

A future canonical receipt should include at least:

- protocol and algorithm versions
- device or pseudonymous key identifier
- verifier-provided challenge
- ordered measurement commitment
- policy identifier and version
- monotonic-state value
- optional input/output commitments
- signature over the complete canonical encoding

The encoding and cryptographic suite are deliberately undecided until the protocol requirements and implementation constraints are compared.

## Design principles

- Minimize trusted mutable code
- Make downgrade and ambiguity failures explicit
- Bind every signed claim to a versioned schema
- Keep verification possible without a hosted Baochip service
- Prefer reproducible simulation and test vectors over aspirational diagrams
