# Evidence Semantics

Status: Phase 0 semantic contract. Field names and concepts are provisional. Encoding, numeric labels, canonicalization rules, and cryptographic algorithms are intentionally undecided.

The complete receipt projection and its separation from persistent-state and
authority-metadata objects are defined in
[Canonical Record Model](CANONICAL_RECORD_MODEL.md). That document remains
semantic: no canonical byte representation or integrity suite is selected.

## Evidence boundary

A Baochip receipt is **Evidence** produced by an Attester in the sense of the IETF RATS architecture. It is not an Endorsement, Reference Value, Appraisal Policy, Attestation Result, or relying-party decision.

A cryptographically valid receipt establishes only that the identified evidence-signing key authenticated the canonical receipt bytes. Additional appraisal is required to decide whether the key, device, measurements, policy, freshness, and operating context are acceptable.

## Proposed receipt claims

| Claim | Required | Semantics |
|---|---:|---|
| `profile` | yes | Stable identifier for the complete receipt profile |
| `schema_version` | yes | Version of the claim schema and interpretation rules |
| `integrity_suite` | yes | Identifier for future signature, digest, and related algorithm choices |
| `key_id` | yes | Identifier used to locate the evidence verification key and associated endorsements |
| `key_generation_context` | yes | Provisioning lineage or evidence-key generation needed to prevent cross-generation substitution |
| `authority_commit_id` | yes | Logical persistent-snapshot identifier whose selector commit released the receipt; not a cryptographic digest |
| `lifecycle_state` | yes | Authenticated lifecycle state at receipt commitment |
| `device_generation` | yes | Rollback-relevant identity or provisioning generation |
| `transition_counter` | yes | Lifecycle-transition order within the declared device generation |
| `measurement_epoch` | yes | Measurement-session scope for the committed transcript |
| `receipt_sequence` | conditional | Receipt order within the profile-declared identity and generation scope |
| `active_version` | yes | Active implementation version at receipt commitment |
| `challenge` | conditional | Verifier-supplied freshness value for challenge-response use |
| `measurement_root` | yes | Domain-separated commitment to the ordered measurement transcript |
| `measurement_context` | yes | Identifier defining what the measurement transcript represents |
| `policy_id` | yes | Identifier of the policy under which evidence was collected and released |
| `policy_version` | yes | Version or immutable digest of that policy |
| `input_commitment` | optional | Domain-separated commitment to selected input bytes or structured input |
| `output_commitment` | optional | Domain-separated commitment to selected output bytes or structured output |
| `extensions` | optional | Profile-governed typed extension entries carrying identifier, criticality, and value |

## Required bindings

The future authenticated receipt envelope must bind all present claims, their
types, their ordering or canonical map representation, and their interpretation
under `profile` and `schema_version`. This is a semantic requirement, not a
claim that an envelope or canonical bytes exist.

Receipt release is valid only after the selected snapshot becomes
authoritative. The receipt subject and `authority_commit_id` must match that
`Committed` authority context, and every claim shared with the authoritative
persistent state must agree. A bare commit identifier does not authenticate the
record and cannot substitute for the future integrity mechanism.

Digest inputs use explicit domain separation. A measurement event, policy digest, input commitment, and output commitment cannot share an undifferentiated hash domain.

## Measurement transcript

The receipt commits to an ordered transcript rather than asserting that every event is independently known-good. The future transcript specification must define:

- event type and version;
- measured byte representation;
- event order;
- component or execution-stage identity;
- extension behavior;
- omitted-event behavior; and
- verifier access to the transcript or supporting disclosure.

Acceptance of `measurement_root` means the commitment is internally consistent with disclosed events. Comparison with endorsed or configured Reference Values is a separate verifier operation.

## Freshness

Challenge freshness and monotonic state answer different questions:

- `challenge` helps bind evidence to a verifier request and resist reuse in another exchange;
- `receipt_sequence` helps a verifier reason about ordering or rollback within a documented scope; and
- neither proves that the measured state remained unchanged after receipt issuance.

The profile must define when each is required, its scope, minimum challenge quality, and verifier retention behavior.

## Input and output commitments

Input and output commitments prove only that the receipt was bound to specified byte or structured-data commitments. They do not establish truthful acquisition, semantic correctness, model quality, causal use, or output correctness.

Every commitment definition must specify canonicalization, media or schema identity, confidentiality implications, and whether selective disclosure is supported.

## Extension behavior

- Unknown noncritical extensions may be retained and ignored according to the profile.
- Unknown critical extensions cause rejection.
- Each extension entry is the source of truth for its own criticality. Any
  derived critical-extension index must match the entries exactly.
- An accepted unknown noncritical extension remains inside the protected
  object and must survive decode/re-encode; ignoring its semantics does not
  permit dropping or rewriting it.
- Duplicate claim keys, noncanonical encodings, type confusion, and unsupported profile versions cause rejection.
- Extensions cannot redefine the semantics of mandatory core claims.

## Verification layers

An implementation reports these layers separately:

1. parsing and canonical-form validation;
2. cryptographic envelope validation;
3. key and endorsement validation;
4. freshness and replay evaluation;
5. lifecycle and rollback evaluation;
6. measurement and Reference Value appraisal;
7. policy appraisal; and
8. relying-party decision.

Success at one layer does not imply success at a later layer.
