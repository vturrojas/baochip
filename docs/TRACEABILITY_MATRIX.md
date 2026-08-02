# Threat and Requirement Traceability

Status: Phase 0 working matrix. “Planned evidence” identifies how a future claim could be supported; it is not evidence that currently exists.

## Threat catalog

| ID | Threat |
|---|---|
| `BCT-001` | An attacker modifies, substitutes, or ambiguously re-encodes a receipt |
| `BCT-002` | A compromised host extracts or misuses device evidence-signing authority |
| `BCT-003` | Evidence is replayed in a different request, time, policy, or device context |
| `BCT-004` | Device state, firmware, policy, or counters are rolled back |
| `BCT-005` | Measurements are reordered, omitted, confused across domains, or misinterpreted |
| `BCT-006` | Unauthorized provisioning, update, recovery, revocation, or decommission occurs |
| `BCT-007` | Reset or power loss creates partial, contradictory, or attacker-selected persistent state |
| `BCT-008` | A verifier collapses cryptographic validity into a claim of software or output correctness |
| `BCT-009` | Identity or evidence exposes unnecessary stable tracking information |
| `BCT-010` | Unsupported algorithms, versions, extensions, or malformed fields are accepted |
| `BCT-011` | Verification depends on an unavailable or untrusted Baochip-operated service |
| `BCT-012` | Security or performance claims exceed the evidence actually produced |

## Traceability matrix

| Threat | Primary requirements | Planned evidence |
|---|---|---|
| `BCT-001` | `BCR-EVI-001`–`004`, `BCR-VER-002`, `BCR-VER-004` | executable semantic fixture corpus, canonical semantic projection, canonical-encoding vectors, mutation corpus, parser differential tests, cross-object/device/generation/profile/version/suite/extension substitution vectors |
| `BCT-002` | `BCR-ID-001`–`003`, `BCR-LIF-001`–`004` | interface noninterference tests, key-lifecycle model, implementation review |
| `BCT-003` | `BCR-FRE-001`, `002`, `004`, `BCR-EVI-002` | nonce replay vectors, verifier state tests, cross-context substitution tests |
| `BCT-004` | `BCR-FRE-002`–`004`, `BCR-LIF-002`–`004` | protected state/authority projection, adversarial state transitions, stale-image tests, counter persistence model |
| `BCT-005` | `BCR-MEA-001`–`004`, `BCR-EVI-003` | transcript permutation tests, domain-separation vectors, disclosure tests |
| `BCT-006` | `BCR-ID-002`, `003`, `BCR-LIF-001`–`004` | lifecycle transition model, authorization negative tests, recovery analysis |
| `BCT-007` | `BCR-FRE-003`, `BCR-LIF-001`, `002`, `BCR-ASR-001` | protected authority-metadata projection including prepared outcomes and record presence, independent selector/phase appraisal design, exhaustive interruption points, atomicity model, typed record/selector corruption injection, reset fuzzing |
| `BCT-008` | `BCR-EVI-005`, `BCR-MEA-003`, `005`, `BCR-VER-002` | verifier-layer tests, documentation review, prohibited-inference cases |
| `BCT-009` | `BCR-ID-002`, `004` | linkability analysis, pseudonymous-profile comparison, privacy review |
| `BCT-010` | `BCR-EVI-003`, `004`, `BCR-VER-003`, `004` | semantic fixture validation, canonical type rules, malformed corpus, version downgrade tests, critical-extension tests, differential decoders |
| `BCT-011` | `BCR-VER-001` | offline reproduction test and published trust-anchor inputs |
| `BCT-012` | `BCR-ASR-001`–`004` | claim-to-evidence ledger, reproducibility records, external review |

## Coverage observations

- Physical invasive attack and side-channel resistance remain outside the initial threat boundary; `BCR-ASR-004` prevents accidental claims about them.
- Availability against destruction or power denial remains out of scope, but safe recovery from interruption is in scope.
- Supply-chain compromise is not solved by a receipt. Endorsement and provisioning assumptions must remain visible to the verifier.
- A compromised host can withhold evidence, inputs, outputs, or communication. Baochip initially targets integrity and provenance properties, not guaranteed availability or truthful sensing.

## Traceability rule

A normative requirement cannot advance into the versioned protocol unless it maps to at least one threat, assumption, or interoperability obligation and has a proposed verification method. A public security claim cannot advance unless the corresponding planned evidence has become a reproducible artifact.
