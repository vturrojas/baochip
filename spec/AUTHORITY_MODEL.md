# Authority Model

Status: Phase 0 semantic specification. Authorization mechanisms and key formats remain implementation decisions.

## Principle

No single vague “administrator” role controls the complete lifecycle. A Baochip profile assigns distinct authorities and may require multiple independent approvals for high-impact transitions.

## Roles

| Role | Responsibility | Prohibited shortcuts |
|---|---|---|
| `RootAuthority` | establishes the device family or manufacturing trust root and authorizes initial provisioning rules | cannot silently override an owner's active policy |
| `OwnerAuthority` | controls operational policy, approved measurements/reference-value sources, and normal updates after ownership transfer | cannot recover or reuse revoked key material |
| `UpdateAuthority` | authorizes a specific firmware or state-machine update under version and compatibility policy | cannot change ownership or erase audit-relevant generation state by implication |
| `RecoveryAuthority` | authorizes recovery images and recovery-state transitions | cannot make recovered state indistinguishable from uninterrupted operation |
| `RevocationAuthority` | revokes a device identity, authority, firmware lineage, or endorsement | cannot reverse a completed revocation of the same identity |
| `DecommissionAuthority` | authorizes terminal secret destruction and retirement | cannot restore a decommissioned device state |
| `PhysicalPresence` | supplies a locally observed condition for transitions that require access to the device | is a condition, not a cryptographic identity or universal override |
| `VerifierOwner` | configures appraisal policy and accepted trust anchors for a verifier | cannot change the evidence originally produced by the device |
| `ReferenceValueProvider` | publishes expected or acceptable measurement values and metadata | cannot sign device Evidence merely by serving reference values |
| `Endorser` | vouches for identified device capabilities, keys, or provenance | does not decide a relying party's trust policy |

One entity may hold multiple roles, but the evidence and policy must show which role authorized each action.

## Default authorization policy

| Transition | Minimum authorization |
|---|---|
| `BLANK` → `PROVISIONING` | `RootAuthority` plus `PhysicalPresence` or a profile-defined manufacturing ceremony |
| `PROVISIONING` → `OPERATIONAL` | `RootAuthority` and prospective `OwnerAuthority`; atomic commitment required |
| `OPERATIONAL` → `UPDATE_PENDING` | `UpdateAuthority` within `OwnerAuthority` version policy |
| `UPDATE_PENDING` → `OPERATIONAL` | authenticated candidate plus compatibility, integrity, and rollback checks |
| `OPERATIONAL` → `RECOVERY` | `RecoveryAuthority` plus `PhysicalPresence` or an independent second authority |
| `PROVISIONING`, `OPERATIONAL`, `UPDATE_PENDING`, `RECOVERY`, or `FAULT` → `REVOKED` | `RevocationAuthority`; emergency profiles may allow a signed remote revocation; `BLANK`, `REVOKED`, and `DECOMMISSIONED` are excluded source states |
| `REVOKED` → `PROVISIONING` | recommission ceremony defined below; never a reversal of the revoked identity |
| any nonterminal state → `DECOMMISSIONED` | `DecommissionAuthority` plus `PhysicalPresence` or an independent second authority |

Profiles may strengthen these rules but cannot merge semantically distinct authority claims without declaring the reduced separation.

Because `BLANK` has no operational identity, `RevocationAuthority` does not
revoke a blank device. Supply-chain and hardware blacklisting are represented
through endorsements, reference values, or `VerifierOwner` policy. An
authorized blank physical device may still be retired through the terminal
decommission transition.

## Recommissioning after revocation

A revoked physical device may be recommissioned only when the profile supports it and all of the following occur:

1. the revoked evidence-signing key and protected operational secrets are destroyed;
2. the device generation advances and cannot be rolled back;
3. a new, cryptographically unrelated evidence-signing identity is established;
4. new endorsements explicitly identify the new generation;
5. the recommission event remains distinguishable from uninterrupted operation;
6. `RootAuthority`, the prospective `OwnerAuthority`, and physical presence or a profile-defined independent authority approve; and
7. verifiers are not required to link the new identity publicly to the old identity.

This is a new provisioning event, not “unrevocation.” If any condition cannot be established, the device remains `REVOKED` or proceeds to `DECOMMISSIONED`.

## Authorization evidence

Every accepted lifecycle transition records non-secret commitments to:

- transition type and version;
- prior and resulting lifecycle state;
- authorizing role identifiers;
- authorization policy identifier and version;
- device generation and transition counter;
- affected firmware, identity, or policy commitments; and
- success or stable rejection class.

The record proves only that the modeled authorization inputs were accepted. Implementation-specific key custody and ceremony assurance require separate evidence.
