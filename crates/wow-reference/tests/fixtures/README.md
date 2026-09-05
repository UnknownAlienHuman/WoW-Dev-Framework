# Synthetic v1 import compatibility

These are fixed synthetic wire-contract examples, not outputs from current
Blizzard data. The native Rust integration test invokes the compiled API,
topology and bundle CLIs; it verifies lookup, digest tamper rejection,
source-generation isolation, partial coverage and no-clobber publication.

Do not regenerate these expected files during tests. New source-to-annotation
work uses the Ketho native loader and its separate source-bound regression tests.
