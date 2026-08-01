# Phase 0 Exit Review

Status: In progress.

## Completed evidence

- Project mission, hypothesis, non-goals, and Surety relationship are explicit.
- Threat model distinguishes logical attackers from out-of-scope invasive and side-channel attacks.
- Initial prior-art matrix covers TPM 2.0, DICE, OpenTitan, Caliptra, RATS, and EAT.
- Thirty security requirements are uniquely identified.
- Twelve threat classes map to requirements and planned evidence.
- Lifecycle states, invariants, transitions, reset behavior, and open questions are specified.
- Receipt claims and layered verification semantics are defined independently of encoding.
- Executable model behavior and required test families are specified.
- Rust is selected for the host-side model and reference verifier through a documented comparison.

## Remaining Phase 0 gates

- Expand prior-art review from orientation to specific mechanism-level comparisons.
- Resolve whether revocation permits reprovisioning under a cryptographically unrelated identity.
- Define authorization actors for provisioning, update, recovery, revocation, and decommission.
- Define counter scopes and exhaustion policy at the semantic level.
- Produce a requirement-to-test-plan mapping for the first model increment.
- Record assumptions about endorsements, trust-anchor distribution, and reference-value ownership.
- Review all normative words (`shall`, `must`, `prohibited`) for consistency.

## Explicitly deferred

- Wire-format selection
- Cryptographic-suite selection
- Production key provisioning
- `no_std` refactor
- Formal verification language
- RTL architecture and language
- FPGA board
- Side-channel and invasive-attack claims

## Go/no-go

**Conditional go for Phase 1 scaffolding.** Repository structure and a Rust workspace may be created, but receipt serialization, real signing, and hardware interfaces remain blocked until their Phase 0 decisions are supported.
