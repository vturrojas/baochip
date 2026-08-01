# Prior-Art Matrix

This is an initial orientation, not a complete literature review or novelty analysis. Specifications are authoritative; summaries below are Baochip's working interpretation.

| System or standard | Primary role | Relevant mechanisms | What Baochip should learn | What Baochip must not imply |
|---|---|---|---|---|
| [TPM 2.0](https://trustedcomputinggroup.org/resource/tpm-library-specification/) | General platform root of trust and cryptographic command architecture | protected objects, PCR-style measurement accumulation, authorization policies, attestation keys, algorithm agility | mature semantics for measurements, protected keys, authorization, and platform lifecycle | that a smaller command set is automatically novel or more secure |
| [TCG DICE](https://trustedcomputinggroup.org/resource/dice-attestation-architecture/) | Layered device identity and attestation | identity derivation across layers, endorsements, evidence certificates, layered trust | minimize immutable state; make each layer's identity and evidence relationship explicit | that derived identity eliminates provisioning, privacy, or recovery problems |
| [OpenTitan](https://opentitan.org/documentation/index.html) | Open silicon root of trust and secure microcontroller | lifecycle controller, key manager, measured/secure boot, OTP, entropy, alerting, update flows | hardware/software co-design, lifecycle as a first-class security boundary, transparent implementation | that Baochip should reproduce a full secure microcontroller |
| [OpenTitan Platform Integrity Module](https://opentitan.org/earlgrey_1.0.0/book/doc/use_cases/platform_integrity_module/index.html) | External platform boot measurement and policy enforcement | boot-flash interposition, reset/heartbeat monitoring, A/B update policy, attestation | distinguish observation, enforcement, and recovery; define when the host is held in reset | that measurement alone supplies runtime integrity |
| [Caliptra](https://www.chipsalliance.org/news/chips-alliance-welcomes-the-caliptra-open-source-root-of-trust-project/) | Open root of trust for measurement in SoCs | device identity, measured boot, attestation, open RTL/firmware/verification | use a transparent verification strategy and compare against an established open RoT | that Baochip is a datacenter SoC RoT competitor at its current stage |
| [IETF RATS architecture](https://www.rfc-editor.org/rfc/rfc9334.html) | Protocol-independent attestation roles and artifacts | Attester, Verifier, Relying Party, Evidence, Endorsements, Reference Values, Appraisal Policy, Attestation Results | use precise evidence terminology and keep evidence appraisal separate from relying-party decisions | that a signed device statement is itself an attestation result or trust decision |
| [Entity Attestation Token](https://www.rfc-editor.org/rfc/rfc9711.html) | Standard claims framework for attestation tokens | CWT/JWT forms, claims, profiles, nonce freshness, nested/submodule evidence | evaluate existing claims and profile machinery before inventing a receipt format | that EAT conformance follows merely from using CBOR or signing a map |

## Initial differentiation question

Baochip should not begin from “build another TPM.” Its plausible research wedge is a deliberately small evidence-producing state machine optimized for:

- inspectable state transitions;
- receipts bound to a versioned policy and selected input/output commitments;
- deterministic executable modeling and adversarial test vectors;
- verifier operation without a Baochip service; and
- explicit separation between device evidence and conclusions about computation.

Whether that wedge is genuinely distinct remains an open research question.

## Required next comparisons

- TCG DICE layering and certificate profiles in detail
- Caliptra specification, RTL boundaries, firmware, and verification strategy
- OpenTitan lifecycle and key-manager state transitions
- TPM PCR, quote, NV counter, policy, and key-hierarchy semantics
- RATS endorsements, reference values, appraisal policy, and composite-device modeling
- EAT profile rules, privacy considerations, nonce requirements, and detached claims
