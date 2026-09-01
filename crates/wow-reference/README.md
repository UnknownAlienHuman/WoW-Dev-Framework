# `wow-reference` contract router

**Status:** E0-B fixture, E1-B persistent ReferenceView and the E4-B transition-evidence seam are implementation-ready documentation; Rust implementation has not started.

`wow-reference` owns exact ReferenceProfiles, source ingestion/evaluation, raw metadata, normalized API/type/event/restriction facts, corrections, coverage/negative authority, immutable ReferenceView records, and explicit cross-profile transition evidence. It does not own project-source continuity, search ranking, accepted lineage graph publication, migration application, runtime state, or service orchestration.

## Canonical routes

### E0-B — fixture ReferenceView

Read the root package beginning with [`FIXTURE_PROFILE.md`](FIXTURE_PROFILE.md), [`LOOKUP_AND_COVERAGE.md`](LOOKUP_AND_COVERAGE.md), [`OPERATIONS.md`](OPERATIONS.md), and [`CONTRACT.json`](CONTRACT.json).

### E1-B — persistent reference build

Read [`e1/README.md`](e1/README.md). It defines exact source snapshots, restricted APIDocumentation evaluation, raw/normalized facts, digest-bound corrections, ReferenceStore plans, coverage/negative authority, and exact read-only ReferenceView.

### E4-B — explicit transition producer

Read [`E4_B_TRANSITION_EVIDENCE.md`](E4_B_TRANSITION_EVIDENCE.md). It defines exact before/after ReferenceView producer partitions for:

```text
stable reference identity
explicit transition or rename/move key
explicit deprecation/replacement
introduced/removed availability
signature/type/restriction changes
reviewed correction transitions
```

Reference transition authority remains scoped to exact compared profiles. Search/source name similarity cannot manufacture aliases, replacements, removal or platform facts.

## Direct dependency boundary

```text
wow-core
wow-store when persistent storage is active
```

`wow-reference` never depends on `wow-annotations`, `wow-project`, `wow-graph`, `wow-search`, `wow-service`, `wow-context`, `wow-cbm`, or applications. E4-C orchestration submits typed transition records to `wow-graph` E4-B.

## Authority boundary

- ReferenceView owns platform-contract facts under exact profiles.
- Blizzard implementation source is a distinct authority class.
- Project/source continuity is owned by `wow-project`.
- Search signals remain Candidate lineage evidence.
- Runtime spell secrecy, taint, combat or hotfix behavior requires exact runtime evidence.
- Deprecated does not imply a replacement target.
- Replacement does not imply same lineage or automatic edit compatibility.

## Current implementation state

```text
documentation frontier: E4-B transition seam
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

The E4-B seam cannot activate before the E1-B Reference implementation, exact paired profiles, correction/coverage records, E2 graph and E4-B registry/fixture gates are implemented and frozen.
