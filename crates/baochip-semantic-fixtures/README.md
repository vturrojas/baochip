# Baochip semantic fixtures

This dependency-free crate defines candidate-neutral semantic objects and a
small positive fixture corpus for Baochip encoding experiments.

It does not serialize values, assign protocol field numbers, select an
encoding or cryptographic suite, compute integrity, parse untrusted bytes, or
claim compatibility with EAT, CBOR, CDDL, COSE, or a custom format.

Future candidate encoders must consume equivalent semantic values and publish
their resulting bytes, rejection vectors, measurements, and reproduction
instructions separately.
