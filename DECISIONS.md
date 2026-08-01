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

## BC-0009 — Explicit lifecycle before command design

- **Status:** Accepted
- **Decision:** Baochip defines lifecycle states, invariants, and authorized transitions before defining device commands or host APIs.
- **Reason:** Provisioning, reset, update, recovery, revocation, and decommission behavior determine the security boundary and cannot be safely bolted on later.

## BC-0010 — Evidence semantics before wire format

- **Status:** Accepted
- **Decision:** Core receipt claims and verification layers are defined independently of CBOR, JSON, EAT, COSE, or any custom encoding.
- **Reason:** Semantic agreement is required to compare encoding candidates without allowing a convenient library to dictate the security model.

## BC-0011 — Rust for the executable model and reference verifier

- **Status:** Accepted
- **Decision:** Use stable Rust with the standard library for the first executable model and reference verifier. Do not require `no_std` until behavior and constrained-target requirements stabilize.
- **Reason:** Rust provides explicit state modeling, memory safety, reproducible tooling, and a credible path toward constrained experiments without forcing hardware-specific design into Phase 1.

## BC-0012 — Encoding family remains open

- **Status:** Open
- **Leading candidate:** A defined EAT profile using CBOR/CWT and COSE, compared against a purpose-built deterministic CBOR baseline.
- **Evidence needed:** canonical-encoding tests, malformed-input behavior, extension semantics, dependency audit, constrained-resource measurements, and independently generated vectors.
