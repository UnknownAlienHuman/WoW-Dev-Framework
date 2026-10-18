# Current recognizer test inputs

`tests/fixture_ids.rs` reads these six JSON files for the current pack fixture-ID
checks. They do not define product acceptance or a future rule framework.

- `core-pack.json` — pack/profile/rule-schema shape and representative rules.
- `fact-bundle.json` — synthetic Emmy/TOC/XML/project fact input with explicit capability/coverage state.
- `match-cases.json` — positive, near-negative, partial, dynamic, signal, hook, library, and state cases.
- `ambiguity-cases.json` — competing producer/target/parent/path cases and expected Possible outcomes.
- `mutation-cases.json` — repository/path/name/literal/edge/coverage/producer-version mutations.
- `graph-output.json` — proposed assertions, output partition, graph validation, and replacement shapes.

Remove unused fixture-ID checks and their exclusive data with the corresponding
pack/rule removal in #195/#103. Preserve useful behavior in the existing typed
source/graph tests; do not turn this inventory into a mandatory corpus campaign.
