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

- Expand prior-art review from orientation to specific mechanism-level comparisons during the relevant Phase 1 design spikes.
- Review all normative words (`shall`, `must`, `prohibited`) for consistency before publishing a versioned specification.

The revocation/recommissioning rule, authority actors, counter scopes, requirement-to-test plan, and trust-input ownership are now defined.

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

**Go for Phase 1 Increment 1 scaffolding.** Repository structure and the dependency-free Rust lifecycle model may now be created. Receipt serialization, real signing, and hardware interfaces remain blocked until their separate design spikes produce supporting evidence.
