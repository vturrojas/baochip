# Roadmap

This roadmap is ordered by evidence, not calendar dates.

## Phase 0 — framing

- Publish the threat model and non-goals
- Define actors, assets, trust boundaries, and lifecycle states
- Compare existing TPM, TEE, secure-element, and open-root-of-trust approaches
- Record unresolved questions as issues

Exit criterion: reviewers can identify exactly what Baochip would and would not protect.

## Phase 1 — protocol specification

- Specify device identity, measurement, counter, receipt, and verification formats
- Define provisioning, update, recovery, revocation, and decommissioning flows
- Publish test vectors and negative cases

Exit criterion: an independent implementation can parse and reject malformed evidence.

## Phase 2 — executable model

- Build a deterministic software model
- Add property tests and adversarial state-transition tests
- Implement a reference verifier

Exit criterion: the model and verifier reproduce the published vectors in CI.

## Phase 3 — RTL experiment

- Select the smallest defensible primitive set
- Implement reference RTL for simulation
- Measure area, latency, state requirements, and fault behavior

Exit criterion: reproducible simulation artifacts and documented limitations.

## Phase 4 — FPGA evaluation

- Map the design to a named development board
- Publish build instructions and measurement methodology
- Commission external review before stronger claims

Exit criterion: independently reproducible evidence on documented hardware.

No release should be tagged until Phase 1 has a versioned specification artifact.
