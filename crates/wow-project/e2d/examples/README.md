# E2-D integrated publication fixtures

- `publication-bundle.json` — exact E2-C candidate, base/head, project plan, graph plan, store plan, objects, validations, and expected manifests.
- `publication-head.json` — shared coherence, ProjectSnapshot, GraphSnapshot, and one coherent head.
- `end-to-end-cases.json` — success, partial, rollback, seal/open failure, CAS conflict, old-reader, inactive adoption, cancellation, and last-known-good cases.
- `CHECKSUMS.json` — prerequisite/profile/vector/member freeze gate.

Null implementation-dependent values are valid only while `implementation_state = not-started`.
