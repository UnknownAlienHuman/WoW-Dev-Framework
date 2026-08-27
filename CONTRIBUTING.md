# Contributing

WoW Dev Framework is in architecture/bootstrap. Contributions should reduce uncertainty around an accepted milestone, not expand the product surface without evidence.

## Before starting

Read `AGENTS.md`, the documentation index, the relevant architecture contract, and the current roadmap gate. Search existing decisions before proposing a new component.

A useful contribution normally does one of the following:

- implements one independently testable responsibility in the active milestone;
- turns a research claim into a pinned fixture or reproducible compatibility probe;
- closes a documented coverage gap;
- improves determinism, provenance, safety, or diagnostic precision;
- clarifies a contract without changing its meaning;
- proposes an ADR with explicit alternatives and consequences.

## Change classes

### Normative contract change

Changes to architecture, public schemas, evidence semantics, storage formats, profile isolation, or accepted ADRs must include:

- the problem and concrete failure mode;
- affected invariants and documents;
- alternatives considered;
- migration and compatibility impact;
- an acceptance test or measurable gate;
- an ADR update.

### Implementation change

Implementation changes must identify the owning crate/application boundary, avoid speculative abstractions, and include tests that prove the intended path executed.

### Research update

Research belongs in `docs/RESEARCH_BASELINE.md` only when it pins an input required by this repository. Living WoW patch/security notes belong in the separate knowledge base. Research conclusions are not production contracts until promoted into a decision, schema, fixture, or test.

### Candidate idea

Ideas belong in `docs/IDEAS.md` and must be marked non-normative. Include a falsifiable experiment, expected benefit, cost, and rejection criterion.

## Pull requests

A pull request should contain:

- a concise problem statement;
- scope and non-goals;
- affected contracts or ADRs;
- exact profile/build assumptions, when relevant;
- evidence or fixtures;
- validation commands and results;
- compatibility and migration notes;
- unresolved coverage gaps.

Do not combine an architecture rewrite, dependency update, schema migration, and unrelated cleanup in one pull request.

## Validation

The implementation is not present yet. As executable components appear, the default local validation sequence is expected to become:

```text
cargo fmt --all --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
cargo test --workspace --all-features
cargo deny check
wow-eval <relevant task set>
```

Until those commands exist, validate documentation routing, links, schemas, fixture checksums, and deterministic generators manually. Report unavailable commands as `skipped`.

## Test requirements

- Golden outputs must be deterministically sorted and stable across repeated runs.
- Negative fixtures must be capable of failing when the target behavior is broken.
- Differential tests must preserve disagreements rather than overwrite one oracle with another.
- Performance claims require a corpus, hardware/runtime description, repeated samples, and raw measurements.
- Runtime WoW claims require the exact client build, restriction state, scenario, and raw probe output.

See `docs/TEST_STRATEGY.md`.

## Compatibility

Public contracts are versioned. Breaking changes require a schema/version bump, migration notes, compatibility fixtures, and a release-note entry once releases begin.

A project generation may use only one active Reference Pack profile. Tests that intentionally compare profiles must keep their outputs and evidence partitions separate.

## License and third-party material

Contributions are accepted under the MIT license. Before copying third-party code or data, record its license and provenance. Prefer small independently authored fixtures and stable source handles over vendoring external repositories.
