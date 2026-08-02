# Lifecycle State Machine

Status: Phase 0 behavioral specification. This document defines security-relevant states and transitions without prescribing storage technology, RTL, firmware, or cryptographic algorithms.

## State model

| State | Meaning | Evidence-signing behavior |
|---|---|---|
| `BLANK` | No operational device identity or owner policy is installed | prohibited |
| `PROVISIONING` | An authenticated provisioning session is establishing identity, trust anchors, and initial policy | prohibited except for explicitly typed provisioning evidence |
| `OPERATIONAL` | The device may measure state and issue operational evidence under the active policy | permitted |
| `UPDATE_PENDING` | An authenticated candidate update has been staged but not accepted | existing operational identity may report the pending state; candidate code may not issue operational evidence |
| `RECOVERY` | Normal operation has failed or an authorized recovery transition has begun | prohibited except for explicitly typed recovery evidence |
| `REVOKED` | The current device identity or authority has been revoked | prohibited; the same identity can never return to operation |
| `DECOMMISSIONED` | Protected identity and operational secrets have been irreversibly retired | permanently prohibited |
| `FAULT` | An invariant, integrity check, persistence operation, or unsupported transition failed | prohibited except for a bounded fault report if safely available |

## State invariants

1. `DECOMMISSIONED` is terminal.
2. `REVOKED` cannot return to `OPERATIONAL` using the revoked identity.
3. Only `OPERATIONAL` may issue normal operational evidence.
4. Every accepted transition increments or otherwise advances rollback-relevant state.
5. A reset cannot complete a partially authorized transition.
6. An interrupted persistent write resolves to the previous committed state, the next committed state, or `FAULT`; it never resolves to an undocumented mixture.
7. Candidate firmware cannot become authoritative until authentication, version policy, and integrity checks succeed.
8. Recovery authority cannot silently preserve an identity whose security properties are no longer justified.

## Transition table

| From | Event | Required conditions | To | Required record |
|---|---|---|---|---|
| `BLANK` | begin provisioning | physical or manufacturing authorization established | `PROVISIONING` | provisioning attempt identifier |
| `PROVISIONING` | commit provisioning | identity, trust anchors, policy, and initial monotonic state validate atomically | `OPERATIONAL` | provisioning commitment |
| `PROVISIONING` | abort or validation failure | none | `BLANK` or `FAULT` | failure class without secret material |
| `OPERATIONAL` | stage update | update authority authenticates candidate metadata | `UPDATE_PENDING` | candidate commitment and prior version |
| `UPDATE_PENDING` | accept update | signature, compatibility, integrity, and rollback policy pass | `OPERATIONAL` | old/new version and advanced state |
| `UPDATE_PENDING` | reject update | validation fails or authorized cancellation occurs | `OPERATIONAL` | rejection reason and unchanged active version |
| `OPERATIONAL` | enter recovery | authenticated recovery request or defined fault policy | `RECOVERY` | recovery cause and authority identifier |
| `RECOVERY` | restore operation | recovery image and resulting identity/policy satisfy recovery rules | `OPERATIONAL` | recovery generation and any identity change |
| `PROVISIONING`, `OPERATIONAL`, `UPDATE_PENDING`, `RECOVERY`, or `FAULT` | revoke | a revocable identity or authority lineage exists and revocation authority and policy validate | `REVOKED` | revocation reason and authority |
| `REVOKED` | recommission | revoked secrets destroyed; generation advances; new unrelated identity; root, prospective owner, and physical/independent authority approve | `PROVISIONING` | recommission record bound to new generation |
| `REVOKED` | decommission | decommission authorization and secret-erasure procedure complete | `DECOMMISSIONED` | non-secret destruction confirmation |
| any nonterminal state | decommission | decommission authorization and secret-erasure procedure complete | `DECOMMISSIONED` | non-secret destruction confirmation |
| any nonterminal state | invariant or persistence failure | fail-closed rule applies | `FAULT` | stable fault class if safely recordable |

Revocation applies only to an identity-bearing source state. `BLANK`,
`REVOKED`, and `DECOMMISSIONED` are excluded; an authorized unprovisioned
device may instead move directly from `BLANK` to terminal `DECOMMISSIONED`.
Supply-chain or hardware blacklisting is expressed through endorsements,
reference values, or verifier-owner policy rather than device identity
revocation.

## Reset and power-loss rules

- Volatile session authorization is lost on reset.
- A nonce or challenge accepted for one receipt request is not implicitly reusable after reset.
- Persistent transitions use an atomic commit protocol whose implementation is deferred.
- Boot validates the committed lifecycle state before enabling evidence signing.
- Counter exhaustion, integrity-check failure, and unreadable persistent state enter `FAULT`; they do not wrap or reset silently.

## Commands by state

The eventual command specification shall define an allowlist for every state. Commands not explicitly allowed in the current state fail closed. At minimum:

- identity creation is limited to `PROVISIONING`;
- measurement and operational receipt issuance are limited to `OPERATIONAL`;
- candidate activation is limited to `UPDATE_PENDING`;
- recovery mutation is limited to `RECOVERY`;
- secret destruction is permitted only through an authenticated revocation or decommission path; and
- diagnostic reads never expose protected key material or data sufficient to reconstruct it.

## Unresolved design questions

- What bounded evidence, if any, can safely be issued from `RECOVERY` or `FAULT`
- How persistence atomicity will be modeled before hardware-specific storage is selected

Recommissioning, authority roles, and counter scopes are defined in [Authority Model](AUTHORITY_MODEL.md) and [Counter Model](COUNTER_MODEL.md).
