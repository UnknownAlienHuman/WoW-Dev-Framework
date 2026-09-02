# `wow-reference` contract router

**Status:** E0-B fixture, E1-B persistent ReferenceView, the E4-B transition-evidence seam, and the E6-B external-locator mapping seam are implementation-ready documentation; Rust implementation has not started.

`wow-reference` owns exact ReferenceProfiles, source ingestion/evaluation, raw metadata, normalized API/type/event/restriction facts, corrections, coverage/negative authority, immutable ReferenceView records, explicit cross-profile transition evidence, and exact mapping of bounded external locators into retained ReferenceViews. It does not own project-source continuity, external-provider semantics, candidate selection, search ranking, accepted lineage graph publication, migration application, runtime state, context orchestration, or service orchestration.

## Canonical routes

### E0-B — fixture ReferenceView

Read the root package beginning with [`FIXTURE_PROFILE.md`](FIXTURE_PROFILE.md), [`LOOKUP_AND_COVERAGE.md`](LOOKUP_AND_COVERAGE.md), [`OPERATIONS.md`](OPERATIONS.md), and [`CONTRACT.json`](CONTRACT.json).

### E1-B — persistent reference build

Read [`e1/README.md`](e1/README.md). It defines exact source snapshots, restricted APIDocumentation evaluation, raw/normalized facts, digest-bound corrections, ReferenceStore plans, coverage/negative authority, and exact read-only ReferenceView.

### E4-B — explicit transition producer

Read [`E4_B_TRANSITION_EVIDENCE.md`](E4_B_TRANSITION_EVIDENCE.md). It defines exact before/after ReferenceView producer partitions for stable identity, explicit transition/rename/move keys, explicit deprecation/replacement, introduced/removed availability, signature/type/restriction changes, and reviewed correction transitions.

Reference transition authority remains scoped to exact compared profiles. Search/source name similarity cannot manufacture aliases, replacements, removal, or platform facts.

### E6-B — external locator mapping owner

Read [`E6_B_EXTERNAL_LOCATOR_MAPPING.md`](E6_B_EXTERNAL_LOCATOR_MAPPING.md). It maps owner-neutral bounded provider locator fields into one exact retained ReferenceView.

Supported identity classes can include API namespace/name, callable/event/CVar/type/enum/restriction stable keys, and exact pinned source artifact/path/span fields. Provider labels, scores, summaries, snippets, and inferred relations never create aliases, replacements, deprecations, or platform facts.

Mapping preserves `ExactMapped`, `MultipleMappings`, `NoMappingWithOwnerAuthority`, `NoMappingPartial`, `Conflict`, `NotEvaluated`, and `Failed`. Clean no-mapping requires complete relevant reference coverage and negative authority.

## Direct dependency boundary

```text
wow-core
wow-store when persistent storage is active
```

`wow-reference` never depends on `wow-annotations`, `wow-project`, `wow-graph`, `wow-search`, `wow-service`, `wow-context`, `wow-cbm`, or applications. E4-C submits transition records to `wow-graph`; E6-B consumes owner-neutral mapping requests through a narrow port.

## Authority boundary

- ReferenceView owns platform-contract facts under exact profiles.
- Blizzard implementation source is a distinct authority class.
- Project/source continuity and project locator mapping are owned by `wow-project`.
- External provider results remain `semantic_candidate + Candidate`.
- Exact locator mapping proves only identity to one reference record.
- Search signals remain Candidate lineage evidence.
- Runtime spell secrecy, taint, combat or hotfix behavior requires exact runtime evidence.
- Deprecated does not imply a replacement target.
- Replacement does not imply same lineage or automatic edit compatibility.

## Current implementation state

```text
documentation frontier: E6-B reference mapping seam
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```

The E6-B seam cannot activate before the E1-B Reference implementation, exact mapping profiles/fixtures, owner coverage/negative-authority records, E6-A, and E6-B service gates are implemented and frozen.