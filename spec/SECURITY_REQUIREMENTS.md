# Security Requirements

Status: Phase 0 working draft. These requirements constrain future specifications and experiments; they do not assert that an implementation exists or satisfies them.

## Evidence integrity

- **BCR-EVI-001:** A verifier shall detect any modification to a signed receipt after issuance.
- **BCR-EVI-002:** A receipt shall bind every security-relevant field to one authenticated canonical encoding.
- **BCR-EVI-003:** A receipt shall identify its schema, protocol version, and cryptographic suite without relying on out-of-band defaults.
- **BCR-EVI-004:** Unknown critical fields, ambiguous encodings, and unsupported versions shall fail closed.
- **BCR-EVI-005:** Evidence shall remain distinguishable from endorsements, reference values, appraisal policy, and attestation results.

## Identity and key protection

- **BCR-ID-001:** The device evidence-signing private key shall not be readable through the host interface.
- **BCR-ID-002:** Evidence shall identify the trust anchor and key-generation or provisioning method needed for verification.
- **BCR-ID-003:** The specification shall support key revocation and device decommissioning.
- **BCR-ID-004:** Device identification shall document linkability and privacy consequences.

## Measurement and policy binding

- **BCR-MEA-001:** Measurement extension shall be order-sensitive and domain-separated.
- **BCR-MEA-002:** A receipt shall bind measurements to a named policy identifier and version.
- **BCR-MEA-003:** The specification shall distinguish measured state, asserted metadata, and verifier-supplied claims.
- **BCR-MEA-004:** Optional input and output commitments shall identify their digest domain and interpretation.
- **BCR-MEA-005:** The verifier shall not infer software correctness solely from an accepted measurement.

## Freshness, replay, and rollback

- **BCR-FRE-001:** Challenge-based receipts shall bind a verifier-supplied nonce with documented minimum entropy.
- **BCR-FRE-002:** State-bearing receipts shall bind monotonic state when rollback detection is required.
- **BCR-FRE-003:** Reset, interrupted write, counter exhaustion, and persistence failure behavior shall be specified.
- **BCR-FRE-004:** A verifier shall distinguish freshness evidence from monotonic-state evidence.

## Lifecycle and update

- **BCR-LIF-001:** Provisioning, operational, update, recovery, revoked, and decommissioned states shall have explicit transitions.
- **BCR-LIF-002:** Unauthorized lifecycle transitions shall be rejected and recorded where persistence permits.
- **BCR-LIF-003:** Firmware or state-machine updates shall be authenticated and rollback constrained.
- **BCR-LIF-004:** Recovery authority and its ability to weaken evidence shall be explicit.

## Verifier behavior

- **BCR-VER-001:** A standalone open reference verifier shall be possible without network access to a Baochip-operated service.
- **BCR-VER-002:** Verification shall separate syntactic validity, cryptographic validity, freshness, reference-value appraisal, and relying-party policy.
- **BCR-VER-003:** Rejection shall produce a stable machine-readable reason without leaking protected state.
- **BCR-VER-004:** Positive and negative test vectors shall cover every normative receipt field and failure class.

## Assurance and evaluation

- **BCR-ASR-001:** Every claimed property shall map to a threat, requirement, implementation mechanism, and test or argument.
- **BCR-ASR-002:** Software-model results, RTL simulation, FPGA measurements, formal checks, and external review shall be reported as distinct evidence types.
- **BCR-ASR-003:** The project shall publish limitations and failed experiments alongside successful results when they affect interpretation.
- **BCR-ASR-004:** No production, certification, side-channel-resistance, or physical-tamper claim shall be made without specific supporting evaluation.
