# Threat Model

## Objective

Baochip aims to make selected device state and execution measurements harder to forge, erase, or roll back without detection. It does not prove that measured software is correct.

## Assets

- Device identity and signing keys
- Boot and workload measurements
- Policy identifiers and version state
- Monotonic state used to detect replay or rollback
- Execution receipts and verifier trust anchors

## Adversaries considered

- A compromised host operating system
- A malicious or buggy application
- A network attacker who can replay, delay, reorder, or modify messages
- An operator attempting unauthorized rollback or evidence substitution
- An attacker with temporary logical access to the host

## Initially out of scope

- Invasive semiconductor attacks
- Nation-state supply-chain compromise
- Side-channel resistance claims
- Availability against physical destruction or power denial
- Correctness of arbitrary software observed by the device

## Trust boundaries

The proposed trusted boundary is limited to the Baochip state machine, its protected key material, authenticated update logic, and the verifier's configured trust anchors. Host software, networks, build systems, and policy authors remain independently fallible.

## Required security properties

- Receipts are unforgeable without the device key
- Measurements are bound to an identified protocol version and policy
- Replayed or rolled-back state is detectable within documented assumptions
- Malformed, ambiguous, or unsupported evidence fails closed
- Provisioning, recovery, revocation, and decommissioning are auditable

## Open questions

- Which properties require tamper resistance rather than tamper evidence?
- How should privacy-preserving or pseudonymous device identity work?
- What is the minimum reliable monotonic-state primitive?
- Which update and recovery authority model is least dangerous?
- How should verifier policy represent freshness and partial evidence?
