# E3-B normative fixture shapes

All IDs, digests, canonical bytes, tokenizer results, benchmark results, and SHA-256 values remain null only while implementation state is `not-started`.

## Active fixtures

- `context-universe-set.json` — exact user-project, optional Blizzard UI, and ReferenceView binding.
- `context-request.json` — exact roots plus reviewed intent, expansion, budget, tokenizer, privacy, and renderer profiles.
- `project-map-l0-l1.json` — deterministic Project Map and L0/L1 identity/evidence closure.
- `context-semantic-pack.json` — selected typed items, evidence, omissions, coverage, conflicts, and budgets.
- `rendered-context-artifact.json` — canonical JSON/Markdown artifact identity and byte/token accounting.
- `budget-source-continuation-cases.json` — mandatory-budget, pruning, source boundary, cancellation, and continuation cases.
- `omission-coverage-cases.json` — omission, partial, conflict, negative-authority, and no-new-evidence cases.
- `cache-determinism-cases.json` — exact cache-key and 1/2/N rebuild mutations.
- `control-effect-cases.json` — closed published-fact control/effect projection cases.
- `metrics-evaluation-cases.json` — noncanonical metrics and frozen utility/recall evaluation cases.
- `CHECKSUMS.json` — prerequisite/profile/corpus/vector/member freeze gate.

Tests verify committed fixture bytes. They never rewrite the canonical examples automatically.
