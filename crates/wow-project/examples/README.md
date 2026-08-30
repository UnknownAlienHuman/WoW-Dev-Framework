# `wow-project` E0-D normative examples

These files define the closed project-generation and publication contract for the future E0-D implementation.

## Files

- [`project-bundle.json`](project-bundle.json) — project configuration, source origin, explicit first-party file inventory, analyzer binding, capabilities, and budgets.
- [`update-cases.json`](update-cases.json) — valid, stale, conflicting, no-op, failure, and deterministic update cases.
- [`publication-cases.json`](publication-cases.json) — initial/degraded/failure/mismatch/last-known-good publication outcomes.
- [`CHECKSUMS.json`](CHECKSUMS.json) — exact prerequisite identity and byte-freeze gate.

## Current state

The repository still contains no E0 Rust implementation. Therefore these values remain null:

```text
selected ReferenceGenerationId
accepted wow-emmy upstream pin/probe/config digest
Main file SHA-256/byte lengths shared with wow-emmy fixture
baseline/update ProjectGenerationId values
AnalyzerSnapshotId values
ProjectSnapshot digests
member/bundle SHA-256 values
```

Nulls are valid only while `crates/MANIFEST.json` reports `wow-project.implementation_state = not-started`.

Before the first `wow-project` Rust commit, the implementation agent must:

1. use the implemented E0-A canonicalization/hash contract;
2. import the accepted E0-C upstream pin/probe/config identity;
3. freeze the selected E0-B fixture profile/reference generation;
4. freeze the E0-C Main source bytes/digests/lengths;
5. verify the project file declarations reference the exact same bytes;
6. derive baseline and update `ProjectGenerationId` vectors;
7. derive expected analyzer/project snapshot IDs/digests;
8. write all member and bundle SHA-256 values;
9. update `CONTRACT.json` and manifest implementation state;
10. execute all applicable `TEST_MATRIX.md` cases.

Tests verify these values and never rewrite them automatically.

## Source ownership

`project-bundle.json` references the Main source fixture declarations under `crates/wow-emmy/examples/workspace-fixture.json`. It does not duplicate the Lua source text.

Before implementation, one canonical byte owner is frozen and both crates verify the same digest. Divergent analyzer/project fixture bytes are a hard failure.

## Semantic boundary

The project examples establish only:

```text
project configuration and file state
project generation identity
project source registry
analyzer snapshot binding
project capability/coverage
atomic publication and update behavior
```

They do not establish:

```text
WoW API existence
Secret/restriction status
WoW diagnostic findings
TOC/XML/load graph
runtime behavior
```

## Change protocol

Any semantic example change must update:

- owning contract documents;
- `CONTRACT.json`;
- affected update/publication cases;
- `TEST_MATRIX.md`;
- freeze identities/checksums after implementation begins.

Do not alter expected publication outcomes to accommodate an implementation that mixes generations or publishes partial state.
