# Decision Log

Decisions are provisional until their stated evidence and consequences are recorded. Superseded decisions remain in this file.

## BC-0001 — Evidence source, not correctness oracle

- **Status:** Accepted
- **Decision:** Baochip will produce evidence about selected state and events. It will not claim to prove that measured software, AI models, policies, inputs, or outputs are correct.
- **Reason:** Signed measurements and correctness are different propositions. Keeping them separate is essential to Baochip's relationship with Surety.

## BC-0002 — Specification and executable model before RTL

- **Status:** Accepted
- **Decision:** A versioned protocol specification, test vectors, reference verifier, and deterministic state-machine model precede RTL work.
- **Reason:** Hardware makes semantic mistakes expensive. The evidence contract must be falsifiable before implementation technology constrains it.

## BC-0003 — Align terminology with IETF RATS

- **Status:** Accepted
- **Decision:** Baochip will use the RATS distinctions among Evidence, Endorsements, Reference Values, Appraisal Policy, Attestation Results, Verifier, and Relying Party.
- **Reason:** Existing terminology prevents Baochip from collapsing evidence production and trust decisions into one vague “attestation” operation.

## BC-0004 — No mandatory hosted verification service

- **Status:** Accepted
- **Decision:** The evidence format and reference verifier shall support offline or locally administered verification.
- **Reason:** A proprietary online dependency would weaken inspectability, reproducibility, and independent research use.

## BC-0005 — Initial implementation language and encoding

- **Status:** Open
- **Candidates:** Rust or another memory-safe modeling language; CBOR/CDDL with COSE; an EAT profile; a purpose-built canonical format.
- **Evidence needed:** requirements traceability, parser complexity, canonicalization behavior, constrained-device cost, existing tooling, interoperability, and negative-test coverage.

## BC-0006 — FPGA target and hardware boundary

- **Status:** Open
- **Candidates:** no board during Phases 0–2; later select an FPGA board only after estimating logic, memory, nonvolatile-state, entropy, and host-interface requirements.
- **Evidence needed:** executable-model resource assumptions and a documented experiment objective.

## BC-0007 — Cryptographic suite

- **Status:** Open
- **Decision rule:** Do not choose algorithms by fashion. Compare security level, implementation footprint, side-channel assumptions, key lifecycle, verifier availability, and algorithm-agility cost.

## BC-0008 — License

- **Status:** Accepted for current artifacts
- **Decision:** Documentation and software are Apache-2.0. A future hardware-specific license may be evaluated before accepting material RTL contributions.
