# Trust Anchor, Endorsement, and Reference-Value Model

Status: Phase 0 semantic model.

## Trust inputs

A Baochip verifier may consume four independent classes of input:

1. **Trust anchors** — public keys or digests configured by the Verifier Owner.
2. **Endorsements** — signed statements about a device key, generation, implementation, or capability.
3. **Reference Values** — expected or acceptable measurements and the metadata needed to interpret them.
4. **Appraisal Policy** — rules defining how Evidence, Endorsements, and Reference Values are evaluated.

None of these inputs is silently fetched from a mandatory Baochip service.

## Ownership

- The `VerifierOwner` decides which trust anchors, Endorsers, Reference Value Providers, and appraisal policies are accepted.
- An `Endorser` controls its signed endorsements but cannot force a Verifier Owner to trust them.
- A `ReferenceValueProvider` controls its published values and validity metadata but does not control the verifier's decision.
- Baochip project artifacts may provide schemas, test anchors, and examples; they are not automatically production trust anchors.

## Endorsement minimum semantics

An endorsement profile must bind:

- endorser identity and key identifier;
- subject evidence key or device-generation identifier;
- asserted capability or implementation identity;
- validity or revocation information;
- profile/schema version;
- algorithm identifiers; and
- any constraints required to interpret the assertion.

An endorsement does not assert that all future evidence from its subject is acceptable.

## Reference Value minimum semantics

A Reference Value record must identify:

- provider and record version;
- measurement context and event interpretation;
- component, firmware, policy, or workload identity;
- acceptable digest or comparison rule;
- validity interval or supersession relationship where applicable;
- device/profile applicability; and
- revocation or withdrawal status.

“Known good” is avoided unless the provider states what property and evaluation support that judgment.

## Distribution modes

The reference verifier must support local files or caller-supplied inputs. Future profiles may additionally support authenticated registries, transparency logs, or enterprise policy services, but network retrieval remains outside core receipt validation.

## Failure behavior

- Missing required trust input produces an indeterminate or rejected appraisal, never implicit trust.
- Expired, revoked, ambiguous, or mismatched endorsements are not accepted.
- Unknown Reference Value history is reported separately from a known mismatch.
- Conflicting trusted sources produce a visible policy conflict.
- Cryptographically valid Evidence can still produce an unacceptable or indeterminate appraisal result.
