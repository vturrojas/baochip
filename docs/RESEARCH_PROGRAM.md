# Research Program

## Core question

Can a small, inspectable hardware state machine provide useful, independently verifiable provenance evidence while keeping its trust claims narrow and testable?

## Workstreams

### Prior-art review

Map Baochip's proposed properties against TPM 2.0, DICE, measured boot, confidential-computing attestations, secure elements, and open hardware roots of trust. Avoid novelty claims until this review is published.

### Protocol design

Define canonical evidence, lifecycle transitions, failure behavior, freshness, privacy, and algorithm agility. Publish positive and negative test vectors.

### Executable model

Build a deterministic model before RTL. Use state-machine and property testing to explore reset, interruption, replay, downgrade, and partial-write behavior.

### Hardware experiment

Only after the model stabilizes, implement the smallest useful subset in RTL and report resource and latency costs with reproducible tooling.

### Evaluation

Evaluate security properties separately from performance. Clearly distinguish analytical arguments, automated checks, simulation results, FPGA measurements, and external review.

## Publication standard

Every reported result should include a commit identifier, toolchain version, configuration, test vectors or workload, expected failure behavior, and reproduction command.
