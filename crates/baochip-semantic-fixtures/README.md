# Baochip semantic fixtures

This runtime-dependency-free crate defines candidate-neutral semantic objects,
a frozen positive fixture corpus, typed negative semantic cases, and test-only
lifecycle/persistence conformance checks for Baochip encoding experiments.

It does not serialize values, assign protocol field numbers, select an
encoding or cryptographic suite, compute integrity, parse untrusted bytes, or
claim compatibility with EAT, CBOR, CDDL, COSE, or a custom format.

Future candidate encoders must consume equivalent semantic values and publish
their resulting bytes, rejection vectors, measurements, and reproduction
instructions separately.

`validate_corpus_conformance` pins the positive manifest and requires exactly
one negative fixture for every stable semantic validation error. The model
crates are development dependencies only; candidate encoders do not acquire a
runtime model dependency.
