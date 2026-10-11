# E3-A normative fixtures

- `source-profile.json` — exact provider/materializer/client/profile/security/license configuration shape.
- `source-snapshot.json` — closed configured-root inventory and content/provenance manifest shape.
- `package-load-model.json` — synthetic package, TOC variant, XML/script, Lua-unit, and direct-load model.
- `analyzer-graph-handoff.json` — exact analyzer/fact/recognizer/producer-partition/graph-validation closure.
- `update-cases.json` — exact reuse/invalidation/removal/no-change/fingerprint vectors.
- `publication-and-skeleton-input.json` — E2-D publication and bounded context-input read vectors.
- `CHECKSUMS.json` — explicit pending acceptance pins, vectors and checksums.

These are unaccepted specification templates. Null future identities and checksums
remain pending and cannot form an accepted bundle. Rust is partially implemented;
the original preimplementation freeze was missed/NotEvaluated. An acceptance
operation must supply its own exact source/profile and genuine native evidence;
current values never retroactively satisfy that historical gate.
