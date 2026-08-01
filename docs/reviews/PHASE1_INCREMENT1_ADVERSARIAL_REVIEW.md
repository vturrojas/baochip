# Phase 1 Increment 1 Adversarial Engineering Review

Status: independent review of commit `1a008e7` plus the fixes on
`junior/phase1-increment1-adversarial-review`.

## Scope

This review treated the Phase 0 specifications as canonical and the Rust model
as the artifact under review. It covered every lifecycle state and command,
authorization policy, rejected-command mutation, revocation and recommission,
update and recovery rollback, receipt eligibility and sequence scope, counter
exhaustion, decommission terminality, and staged-state cleanup.

The review did not select or implement receipt encoding, cryptography,
persistence emulation, property-testing dependencies, FPGA hardware, or RTL.
Those exclusions limit what this increment can prove.

Baseline evidence:

- Reviewed commit: `1a008e7` (`Implement initial Baochip lifecycle model [skip ci]`).
- Baseline Rust tests: 9.
- Baseline core dependencies: none.
- Baseline model: eight lifecycle states and twelve command variants.

## Specification-to-code discrepancies

The following canonical requirements remain only partial or unimplemented:

1. `spec/EXECUTABLE_MODEL.md` requires explicit reset and interruption
   injection points plus `Prepared`, `Committed`, and `Cleaned` persistence
   phases. The model still mutates one in-memory state object and cannot test
   power-loss recovery. This was explicitly excluded from this review's
   implementation scope.
2. `spec/AUTHORITY_MODEL.md` requires accepted transition records to bind the
   transition type/version, authorizing role identifiers, policy identifier and
   version, affected commitments, and success or stable rejection. `AuditEvent`
   currently records only prior/resulting state, committed/staged device
   generation, and transition counter. Rejected commands return
   `Err(Rejection)` without an audit event.
3. `spec/EXECUTABLE_MODEL.md` calls for a protected identity handle, policy and
   firmware commitments, an ordered measurement accumulator, and staged
   persistence. This increment represents only an `identity_active` flag,
   firmware versions, counters, and limited staging.
4. `spec/AUTHORITY_MODEL.md` requires a recommissioned identity to be
   cryptographically unrelated, newly endorsed, and distinguishable. The model
   now preserves revocation provenance and advances generation, but it cannot
   prove cryptographic unlinkability, key destruction, or endorsement issuance.
5. Recovery completion advances rollback state and its audit shows a transition
   from `RECOVERY`, but receipts contain no dedicated recovery generation or
   identity-change marker. Equal-version recovery is therefore not fully
   distinguishable to a verifier using receipt claims alone.
6. Update acceptance now consumes explicit abstract authentication,
   compatibility, and integrity results. These are deterministic model inputs,
   not implementations of signature verification or compatibility policy.
7. The default provisioning policy supports physical presence but not the
   profile-defined manufacturing-ceremony alternative.
8. Six stable rejection classes remain typed but unreachable in this
   increment: `InvalidVersion`, `ReplayDetected`, `MalformedInput`,
   `UnsupportedProfile`, `UnsupportedCriticalExtension`, `PersistenceFailure`,
   Their later behavior must not be inferred from type presence.
9. Receipt claims remain a deliberately small semantic subset and omit profile,
   schema, suite, key identifier, measurement root/context, policy identifier,
   and commitment fields from `spec/EVIDENCE_SEMANTICS.md`.

## Defects found and fixes made

### High severity

1. **Counter exhaustion did not fail closed.** Receipt-sequence,
   measurement-epoch, and transition-counter exhaustion returned an error while
   leaving the device in its prior operational state.
   - Fix: exhaustion now enters `FAULT`, erases staged state and disables the
     active identity without wrapping counters.
   - Exception preserved: separately authorized decommission remains available
     with an exhausted transition counter.
2. **Aborting recommission bypassed the recommission ceremony.**
   `REVOKED -> PROVISIONING -> AbortProvisioning` returned to `BLANK`, after
   which ordinary provisioning required fewer authorities.
   - Fix: provisioning origin is staged explicitly; initial-provisioning abort
     returns to `BLANK`, while recommission abort returns to `REVOKED`.
3. **A never-provisioned revoked device could enter recommission.** Revoking
   `BLANK` was permitted by the canonical any-nonterminal rule, but the resulting
   state could incorrectly invoke a ceremony that assumes destroyed prior
   identity material.
   - Fix: recommission requires a prior nonzero device generation and otherwise
     returns `InvalidTransition` without mutation.
4. **Update activation did not represent required validation evidence, and
   update rejection mixed trusted failure with caller cancellation.**
   - Fix: `AcceptUpdate` now requires typed abstract authentication,
     compatibility, and integrity outcomes. `RejectUpdate` now uses a typed
     validation-failure or authorized-cancellation cause; cancellation requires
     both update and owner authorities.

### Medium severity

5. **Accepted staging and terminal transitions did not consistently advance
   rollback-relevant state.** Beginning provisioning, beginning recommission,
   and decommissioning changed persistent lifecycle state without recording an
   available transition-counter advance.
   - Fix: both staging transitions advance the counter; decommission advances it
     when possible and saturates only for the specified exhaustion exception.
6. **Integrity and internal invariant failures did not enter `FAULT`.** Missing
   provisioning/update staging and an inactive operational identity returned
   errors while leaving the prior lifecycle intact.
   - Fix: these paths now erase staging and identity state and enter `FAULT`.
7. **Revocation disagreed with the canonical source-state rule and retained
   staged data.** `BLANK` was excluded despite the any-nonterminal requirement,
   and provisioning staging survived revocation.
   - Fix: authorized revocation is accepted from all nonterminal states except
     an already revoked state, and it erases update/provisioning staging.
8. **Provisioning interruption had no explicit abort command.**
   - Fix: `AbortProvisioning` now records rollback-relevant state, erases staging,
     and returns to the safe origin state.
9. **Provisioning and recommission audit records did not bind the staged next
   generation.**
   - Fix: successful audit output now includes `staged_device_generation` while
     provisioning or recommissioning is pending.
10. **Malformed staged state could bypass fail-closed invariants.** Provisioning
    commit accepted an equal/lower generation, update rejection accepted a
    missing candidate, and provisioning abort could retain an injected stale
    update candidate.
    - Fix: nonadvancing generations and missing candidates now enter `FAULT`
      with `InternalInvariantViolation`; abort erases all staged fields.

## Test and requirement coverage

The dependency-free suite now contains 34 Rust tests, up from 9 (+25). One
table-driven test executes all 104 combinations of eight lifecycle states and
thirteen command variants with canonical authorized inputs. It asserts the
exact accepted destination or stable state-gating rejection, successful audit
state binding, and full-state equality for ordinary rejected commands.

Additional tables and focused tests cover:

- every required authority bit and second-condition policy;
- lower, equal, and higher recovery versions;
- lower/equal update rollback and typed update validation;
- every implemented transition-counter exhaustion path;
- receipt and measurement counter exhaustion without wrap;
- device-generation exhaustion and decommission availability;
- receipt-sequence uniqueness within a generation and tuple uniqueness across
  recommission;
- revocation erasure of staged update and recommission state;
- recommission abort provenance and blank-device recommission denial;
- integrity/invariant failure transition to `FAULT`;
- revoked receipt denial and terminal decommissioning.

Eight rejection classes now have direct behavioral coverage:
`InvalidState`, `Unauthorized`, `InvalidTransition`, `RollbackDetected`,
`CounterExhausted`, `IntegrityFailure`, `Decommissioned`, and
`InternalInvariantViolation`.

This test expansion materially covers the lifecycle portion of
`BCR-LIF-001` through `BCR-LIF-004` and parts of `BCR-FRE-002` and
`BCR-FRE-003`. It does not satisfy receipt cryptography/encoding, measurement
accumulator, verifier, replay, persistence-interruption, or bounded model
exploration test families. No E2E application surface exists in this
dependency-free library increment; the state/command matrix is the broadest
integration-style behavioral test available here.

No line-coverage tool is installed in the review environment, so a numeric
statement of 80% line coverage would be unsupported. Behavioral coverage is
reported by matrix cells, rejection classes, and focused requirement cases
instead.

## Remaining risks and blockers

- Phase 1's full exit criterion remains blocked on persistence interruption,
  measurement accumulation, receipt mutation/vector work, challenge replay,
  reference verification, and bounded command-sequence exploration.
- Rejected lifecycle attempts are not represented by an audit event, and
  successful audit events omit several canonical authorization commitments.
- Recommission identity separation, destroyed-secret evidence, and new
  endorsements remain abstract assertions outside this model.
- Recovery lacks a receipt-bound recovery generation/identity-change marker.
- Abstract update-validation values must not be described as cryptographic
  validation.
- The model contains no manufacturing-ceremony policy alternative.
- Typed but unreachable rejection classes remain untested until their related
  protocol/profile surfaces exist.

## Validation results

Fresh validation on 2026-08-01 in Ubuntu WSL completed as follows:

| Command | Result |
|---|---|
| `cargo fmt` | passed, exit 0 |
| `cargo fmt --check` | passed, exit 0 |
| `cargo clippy --all-targets --all-features -- -D warnings` | passed, exit 0, no warnings |
| `cargo test --all-targets` | passed, 34 tests, 0 failed |
| `git diff --check` | passed, exit 0 |
| `cargo tree -p baochip-model --edges normal` | only `baochip-model`; no runtime dependencies |
| unsafe-code scan | only `#![forbid(unsafe_code)]` found |
| license check | workspace declares `Apache-2.0` |
| changed-content secret-pattern scan | no matches |

No supported Rust line-coverage tool was installed, so no numeric coverage
percentage is claimed. The behavioral evidence is the complete 104-cell matrix
and the focused requirement tests described above.

The final independent correctness reviewer reported no remaining actionable
Critical or Important findings. The final security reviewer reported no new
actionable code-level security defect beyond the disclosed broader
audit/evidence boundary.
