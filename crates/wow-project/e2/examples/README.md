# E2-C project indexing normative fixtures

- `source-snapshot.json` — closed source roots, universes, files, package and profile pins.
- `toc-cases.json` — variants, directives, file order, dependencies, LOD/bootstrap, SavedVariables, unknown/malformed cases.
- `xml-cases.json` — templates, objects, parents, inheritance, includes, external/inline scripts, unknown/security cases.
- `load-model.json` — package graph, phases, direct order, reachability, conflicts, and static-not-runtime assertions.
- `analyzer-recognizer-handoff.json` — physical/virtual Lua units, AnalyzerSnapshot binding, fact adapters, recognizer outputs, graph proposal validation.
- `invalidation-cases.json` — Lua/TOC/XML/profile/rule/registry changes, exact invalidation, conservative widening, reuse, and stale-removal vectors.
- `project-index-candidate.json` — complete/partial/failure/no-change candidate and E2-D publication-bundle boundary.
- `CHECKSUMS.json` — prerequisite/profile/real-addon/vector/member/bundle freeze gate.

Implementation-dependent IDs, exact source bytes, pinned real-addon revision, expected outputs, and SHA-256 values remain null only while `implementation_state` is `not-started`. Tests verify committed fixtures and never rewrite them automatically.
