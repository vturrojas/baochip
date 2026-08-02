# Phase 1 Increment 4 Exit Review

Status: Proposed design-tranche exit record.

## Scope completed

- Defined protected semantic object classes.
- Enumerated the complete persistent-state and authority-metadata projections.
- Connected execution receipts to the canonical projection boundary.
- Defined type, optionality, versioning, extension, and domain-separation
  requirements.
- Established a comparative encoding evaluation plan.
- Preserved the open encoding and cryptographic-suite decisions.

## Claims this tranche supports

- Baochip has an explicit inventory of semantic values a future integrity
  mechanism must bind.
- Candidate encodings can be compared against one shared projection and
  evidence gate.
- Rust layout, enum discriminants, omitted defaults, and parser convenience are
  prohibited as accidental protocol definitions.

## Claims this tranche does not support

- No canonical bytes exist yet.
- No parser, encoder, vector corpus, integrity algorithm, signature, MAC,
  checksum, or key hierarchy has been implemented.
- No EAT, CBOR, CDDL, COSE, or custom binary profile has been selected.
- No constrained-resource, interoperability, cryptographic, durability, FPGA,
  or hardware result has been measured.

## Exit checklist

- [ ] Canonical record specification is reviewed against every current model
      field and evidence claim.
- [ ] Encoding plan retains at least two serious candidates.
- [ ] Decision log keeps encoding and cryptographic choices open.
- [ ] Traceability maps canonicality work to substitution, rollback,
      interruption, and malformed-input threats.
- [ ] Documentation contains no byte-level or security result that has not been
      produced.
- [ ] Independent adversarial review is complete.

The tranche exits only after every item above is satisfied and the review
commit is merged.
