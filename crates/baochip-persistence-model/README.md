# Baochip persistence model

This dependency-free research crate layers a two-slot atomic persistence model over `baochip-model`.

It models complete-record preparation, atomic selector commit, cleanup, interruption before and after commit, monotonic logical commit identifiers, and audit results for rejected or interrupted operations.

It does not model physical storage, torn record writes, corruption, checksums, concurrency, cryptography, or production durability.
