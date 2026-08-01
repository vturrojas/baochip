# Project Charter

## Mission

Baochip investigates whether a small, inspectable hardware state machine can produce useful, independently verifiable evidence about device state and selected computation without expanding that evidence into claims it cannot support.

## Research hypothesis

A minimal root of trust that protects device identity, ordered measurements, monotonic state, and canonical receipt signing can improve evidence provenance while remaining simpler to inspect and reproduce than a general-purpose trusted execution environment.

This is a hypothesis to test, not a project result.

## Intended output

The first complete research artifact is not a chip. It is a versioned, implementation-independent specification with:

- a bounded threat model;
- defined lifecycle and failure states;
- canonical evidence semantics;
- positive and negative test vectors;
- a reference verifier; and
- an executable state-machine model.

RTL and FPGA experiments follow only if the model establishes a defensible primitive set.

## Relationship to Surety

Surety asks what evidence justifies reliance on an AI-assisted decision system. Baochip explores one possible source of hardware-backed evidence. Baochip evidence may inform a Surety assurance argument, but possession of a valid Baochip receipt does not establish that software, a model, a policy, or an output is correct.

## Governing principles

1. **Evidence before confidence.** Every phase exits through recorded, reproducible evidence.
2. **Narrow claims.** A measurement is evidence of an observed value, not proof that the measured artifact is safe.
3. **Verifier independence.** Verification must not require a proprietary hosted Baochip service.
4. **Explicit lifecycle.** Provisioning, update, recovery, revocation, and decommissioning are part of the security design.
5. **Software model before RTL.** State semantics must be testable without hardware.
6. **No novelty claim without comparison.** Prior art must be documented before differentiating Baochip.

## Phase 0 exit gate

Phase 0 is complete only when:

- the prior-art matrix covers TPM 2.0, DICE, OpenTitan, Caliptra, and IETF RATS/EAT;
- requirements are uniquely identified and traceable to threats;
- unresolved architecture decisions are recorded rather than silently assumed;
- the initial evidence boundary can be explained without implementation language; and
- no document claims fabricated hardware, validated security, or production readiness.
