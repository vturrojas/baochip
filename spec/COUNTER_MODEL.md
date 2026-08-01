# Counter and Generation Model

Status: Phase 0 semantic specification. Storage width and physical monotonic primitive remain implementation decisions.

## Independent scopes

| Value | Scope | Purpose | Reset behavior |
|---|---|---|---|
| `device_generation` | physical trust-root lineage | distinguishes reprovisioned or identity-replaced device generations | never decreases; normal reset has no effect |
| `transition_counter` | device generation | orders committed lifecycle, policy, identity, recovery, and update transitions | never decreases within a generation |
| `measurement_epoch` | operational boot or explicitly started measurement session | separates measurement transcripts across resets/sessions | advances before a new accepted epoch |
| `receipt_sequence` | key identity and device generation | supports ordered receipt/replay policy when enabled | never reused within its scope |

These values are semantically distinct. A profile may store or derive them together only if it preserves each scope and exhaustion behavior.

## Commit rules

- A counter value becomes externally meaningful only through an atomic committed transition or receipt.
- Failed or interrupted operations may consume values; they must never cause reuse.
- Verifiers treat gaps as possible failed, interrupted, withheld, or concurrent operations—not automatically as compromise.
- A reset does not return any protected counter to a prior value.
- Counter comparison includes the counter scope and device generation; bare integers are insufficient.

## Receipt allocation

Receipt issuance reserves a sequence value before releasing authenticated evidence. If authentication or delivery fails after reservation, the value remains consumed. The verifier does not require contiguous sequences unless a profile defines a stronger audit channel.

Challenge-only profiles may omit `receipt_sequence`, but must not claim rollback ordering from challenge freshness alone.

## Exhaustion

Counter wrap is prohibited.

- `receipt_sequence` exhaustion disables receipt issuance and enters `FAULT` unless an authenticated identity-generation transition can occur without reusing the exhausted scope.
- `measurement_epoch` exhaustion prevents starting a new epoch and enters `FAULT`.
- `transition_counter` exhaustion prevents further mutable lifecycle transitions except a separately authorized decommission path whose safety does not depend on incrementing the exhausted counter.
- `device_generation` exhaustion prohibits recommissioning; decommission remains the only terminal path.

No exhaustion path silently resets, truncates, saturates while continuing operation, or substitutes wall-clock time.

## Verifier state

A stateful verifier may retain the greatest accepted tuple for a configured identity and scope. A stateless verifier can validate cryptographic structure and a challenge but cannot claim historical rollback detection without another trusted state source.

Verifier rollback decisions must distinguish:

- lower generation;
- same generation with lower transition counter;
- reused receipt sequence;
- older measurement epoch; and
- unknown verifier history.

Unknown history is not equivalent to a fresh device.
