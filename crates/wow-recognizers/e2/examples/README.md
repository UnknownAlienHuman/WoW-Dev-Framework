# E2-B recognizer normative fixtures

- `core-pack.json` — pack/profile/rule-schema shape and representative rules.
- `fact-bundle.json` — synthetic Emmy/TOC/XML/project fact input with explicit capability/coverage state.
- `match-cases.json` — positive, near-negative, partial, dynamic, signal, hook, library, and state cases.
- `ambiguity-cases.json` — competing producer/target/parent/path cases and expected Possible outcomes.
- `mutation-cases.json` — repository/path/name/literal/edge/coverage/producer-version mutations.
- `graph-output.json` — proposed assertions, output partition, graph validation, and replacement shapes.
- `CHECKSUMS.json` — prerequisite/profile/member/bundle freeze gate.

Implementation-dependent IDs, expected canonical bytes, graph validation outputs, evaluation thresholds, and SHA-256 values may remain null only while `implementation_state` is `not-started`. Tests verify frozen fixtures and never rewrite them automatically.
