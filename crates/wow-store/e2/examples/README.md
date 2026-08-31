# E2-D ProjectStore fixtures

- `physical-profile.json` — selected file-per-generation SQLite/runtime/filesystem profile.
- `store-generation.json` — staging, sealed, open-validated, and headed manifest closure.
- `publication-cases.json` — successful, cancel, fault, checksum, and CAS scenarios.
- `read-lease-cases.json` — stable old readers, exact historical reads, and lease/GC interaction.
- `recovery-gc-cases.json` — inventory classification, sealed-inactive adoption, retention roots, and object sweep.
- `CHECKSUMS.json` — prerequisite/profile/vector/member freeze gate.

Implementation-dependent IDs and SHA-256 values remain null only while `implementation_state = not-started`.
