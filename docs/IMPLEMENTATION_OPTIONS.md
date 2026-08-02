# Implementation and Encoding Options

Status: Phase 0 comparison. Versions and crate maturity must be rechecked when implementation begins.

## Executable-model language

| Option | Strengths | Risks | Decision |
|---|---|---|---|
| Rust | algebraic data types fit explicit states and outcomes; memory safety without garbage collection; strong unit, integration, property-test, fuzzing, and embedded ecosystem; plausible path to `no_std` experiments | learning/toolchain cost; unsafe transitive dependencies require audit; Rust does not itself prove protocol correctness | **Selected for the executable model and reference verifier** |
| Go | simple tooling, readable concurrency, mature fuzzing, easy verifier deployment | garbage collection and runtime make it a weaker fit for future constrained targets; state modeling is less expressive | retain as an interoperability-test candidate, not the canonical model |
| Python | fastest exploratory scripting and vector inspection | dynamic types and runtime behavior weaken its value as the canonical security state model | useful only for independent test-vector tooling |
| Formal specification language first | exhaustive or symbolic analysis may find design errors early | high setup cost and risk of modeling a design that has not stabilized | defer; add a small formal model after Rust semantics and invariants stabilize |

### Rust decision boundary

Rust is selected for host-side research artifacts, not automatically for ROM, firmware, RTL, or every future component. The first crate uses the standard library. A `no_std` split occurs only after model behavior stabilizes and constrained-target requirements are known.

## Evidence encoding

| Option | Strengths | Risks | Current position |
|---|---|---|---|
| EAT claims profile represented as a CBOR Web Token, with COSE evaluated separately | aligns with IETF RATS/EAT vocabulary; standard claims/profile machinery; interoperability path | EAT is a framework, CBOR is the encoding, and neither defines a Baochip profile or canonicalization policy | serious candidate; no preference assigned |
| Purpose-built deterministic CBOR described by CDDL, with COSE evaluated separately | compact; explicit schema; good constrained-device ecosystem | CDDL describes data shape but does not itself enforce canonical decoding or security semantics | serious candidate; no preference assigned |
| Purpose-built deterministic binary grammar | complete control of accepted forms and streaming behavior | high risk of private-protocol ambiguity, parser defects, and maintenance burden | serious candidate; no preference assigned |
| Protobuf or another IDL | strong tooling and schema evolution | canonical signing semantics and constrained verification require additional rules; poor direct alignment with EAT | not preferred for the signed evidence core |
| JSON/JWT | familiar debugging and broad libraries | larger and easier to parse inconsistently; canonical JSON and integer/byte representation complicate deterministic signing | possible presentation form, not preferred core encoding |

## Rust CBOR/COSE candidates

- [`minicbor`](https://docs.rs/minicbor) is designed for small CBOR encoding/decoding and supports `no_std` configurations. Its exact canonical-decoding behavior must be tested rather than assumed.
- [`ciborium`](https://github.com/enarx/ciborium) offers CBOR utilities and underpins the `coset` COSE types.
- [`coset`](https://docs.rs/coset/latest/coset/) provides Rust COSE structures. Cryptographic use, protected headers, canonical payload handling, and rejection behavior still require Baochip-specific review.
- [`cose_minicbor`](https://docs.rs/cose_minicbor/latest/cose_minicbor/) is oriented toward borrowed, minimal-allocation, `no_std` COSE decoding but is decode-first and must not be assumed to supply a full signing/verifying profile.

No crate is approved merely by appearing in this list. The encoding spike must pin versions, inventory dependencies, run malformed-input and differential tests, and document unsupported behavior.

## Encoding spike acceptance tests

Each candidate must demonstrate:

1. one canonical byte representation for every accepted core receipt;
2. rejection of duplicate keys and noncanonical alternatives when required by the profile;
3. explicit protected-header coverage;
4. stable handling of unknown critical and noncritical extensions;
5. bounded allocation and input size behavior;
6. negative tests for truncation, nesting abuse, integer edge cases, and trailing data;
7. independently generated compatible vectors; and
8. separation between parsing, cryptographic validation, and appraisal.

## Current decisions

- Rust: accepted for the executable model and reference verifier.
- Receipt semantics: accepted at the abstract claim level.
- Evidence encoding and security envelope: no candidate selected.
- Cryptographic algorithms: open.
- FPGA board and RTL language: open.
