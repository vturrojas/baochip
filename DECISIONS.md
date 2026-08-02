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
- **Candidates:** An EAT claims profile represented as a CWT using deterministic CBOR; purpose-defined deterministic CBOR described by CDDL; and a purpose-built deterministic binary format. Any future COSE envelope is evaluated separately from payload canonicalization.
- **Evidence needed:** canonical-encoding tests, malformed-input behavior, extension semantics, dependency audit, constrained-resource measurements, and independently generated vectors.

## BC-0013 — Revocation is permanent for an identity

- **Status:** Accepted
- **Decision:** A revoked evidence identity can never return to operation. A physical device may be recommissioned only through a new provisioning ceremony, advanced device generation, destroyed old secrets, and a cryptographically unrelated identity.
- **Reason:** This preserves revocation meaning while allowing controlled hardware reuse without pretending continuity of trust.

## BC-0014 — Separate lifecycle authorities

- **Status:** Accepted
- **Decision:** Provisioning, ownership, update, recovery, revocation, decommission, endorsement, reference-value, and verifier-owner roles are semantically distinct. High-impact recovery and decommission transitions require physical presence or an independent second authority by default.
- **Reason:** A universal administrator would collapse the lifecycle trust boundary and make recovery an undocumented bypass.

## BC-0015 — Separately scoped monotonic values

- **Status:** Accepted
- **Decision:** Model device generation, lifecycle transition count, measurement epoch, and receipt sequence as distinct scopes. Wrap and silent reset are prohibited.
- **Reason:** These values answer different freshness and rollback questions; one ambiguous counter cannot safely substitute for all of them.

## BC-0016 — Verifier Owner controls trust inputs

- **Status:** Accepted
- **Decision:** The Verifier Owner selects trust anchors, accepted Endorsers, Reference Value Providers, and appraisal policy. Baochip does not require a project-operated verification or trust-anchor service.
- **Reason:** Independent appraisal and explicit trust configuration are core research goals.

## BC-0017 — Integrity verdict before integrity mechanism

- **Status:** Accepted for the executable model
- **Decision:** Model record and selector recovery using an abstract `Valid` or `Corrupted` verdict before selecting serialization, checksums, digests, MACs, signatures, or physical storage.
- **Reason:** Recovery authority semantics can be falsified independently of an implementation mechanism, while explicit naming prevents the model from becoming an unsupported cryptographic or durability claim.

## BC-0018 — Recovery preserves authority rather than availability

- **Status:** Accepted
- **Decision:** A prepared candidate is never promoted after loss of its previous authority, and a corrupted committed selection is never rolled back to an obsolete record. Ambiguous or impossible recovery fails closed without mutation.
- **Reason:** Continuing from a convenient record would allow corruption to redefine the commit boundary, defeat revocation or rollback policy, and turn availability pressure into an authority bypass.

## BC-0019 — Canonical semantic projection precedes canonical bytes

- **Status:** Accepted
- **Decision:** Define the complete typed semantic projection for persistent state, authority metadata, receipts, and future trust inputs before selecting field labels, byte ordering, serialization, or a canonical encoder.
- **Reason:** Otherwise Rust layout, omitted defaults, library behavior, or the first convenient encoding can silently redefine the security contract.

## BC-0020 — Protected object classes are domain-separated

- **Status:** Accepted at the semantic layer
- **Decision:** Persistent state, authority metadata, execution receipts, lifecycle audits, endorsements, and reference values are distinct protected object classes. A future integrity mechanism must bind the object class, profile, schema version, suite, subject scope, complete extension set, and complete class-specific payload.
- **Reason:** Equal-looking fields from different contexts must not be substitutable or verifiable under the wrong security meaning.

## BC-0021 — Encoding selection requires comparative evidence

- **Status:** Accepted decision rule; encoding remains open
- **Decision:** Do not select EAT/CBOR/COSE or a custom deterministic format until at least two candidates encode the same frozen semantic fixtures, publish negative vectors, undergo differential decoding, and report measured constrained costs.
- **Reason:** Standards alignment, small output, or library availability alone cannot establish canonicality, rejection safety, independent implementability, or suitability.
