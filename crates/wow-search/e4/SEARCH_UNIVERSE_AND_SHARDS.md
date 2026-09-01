# E4-A search universes and immutable shards

**Status:** normative.

## Initial universe classes

```text
PrimaryUserProject
BlizzardUiSource
ReferencePlatformContract
```

Dependency source already represented inside a project publication stays in that project's universe under its exact source-class identity. Runtime, historical lineage, external candidate, and Codebase Memory universes are deferred.

## One shard, one exact owner generation

### User-project shard

Binds the exact:

```text
ProjectStore/Epoch/Generation/PublicationSet
ProjectGeneration/ProjectSnapshot/ProjectView
GraphGeneration/GraphSnapshot/GraphView
AnalyzerSnapshot identity carried by the publication
source/detail/capability/coverage/conflict manifests
```

### Blizzard UI shard

Binds the equivalent separate E3-A platform-source publication, graph view, source/detail catalogs, and exact `SkeletonInputView` profile only when indexed fields require it.

### Reference shard

Binds the exact:

```text
ProfileId
ReferenceGenerationId
ReferenceViewId
ReferenceStore identity
entity/fact/correction/explicit-alias/coverage/conflict manifests
```

Implementation-source fields never become Reference authority.

## SearchUniverseSet binding

A query binds exact validated shard IDs. Compatibility checks:

- product, client flavor, build, Interface, and ReferenceProfile;
- source/reference/project/graph generation compatibility;
- entity-kind, document, field, query, lane, and ranking schema versions;
- graph registry and owner detail-handle compatibility;
- privacy, license, and consumer trust class;
- required lane/index capabilities;
- coverage, conflict, validation, and retention state.

Normal E4-A requests reject incompatible profiles. Cross-profile comparison is a later explicit contract.

## Why shards remain separate

- owner generations stay isolated;
- a Reference Pack update does not mutate a project shard;
- a project update does not rebuild Blizzard UI/reference shards;
- retained documents cannot alter new text corpus statistics;
- raw FTS ranks stay local;
- privacy/license policy can differ by universe;
- shards can be rebuilt/discarded without changing owner truth;
- service can acquire exact combinations explicitly.

## No combined mutable global index

E4-A does not maintain a database containing whichever source/reference/project versions happen to be current. Such an index would hide generation switches, leak stale documents into scores, and weaken ownership/privacy boundaries.

A higher layer may cache an exact `SearchUniverseSet` binding. The bound shards remain independently identified.

## Optional and required shards

- The primary user shard is required for user-project search.
- The Reference shard is required for API/reference search and any request requiring Reference facts or authoritative reference absence.
- Blizzard UI may be omitted only when the request/profile declares it optional.
- Missing optional shards produce explicit omission and lane-availability state.
- No prior, nearest-build, same-name, or current fallback.

## Shard lifecycle

```text
Planned
-> Building
-> PublishedInactive
-> Validating
-> Validated
-> SealedReadOnly
-> Retained
-> GCEligible
```

Failed, cancelled, corrupt, or incomplete shards never become sealed.

Search core has no current pointer. Owner/service catalogs map exact owner generations to exact validated shards.

## Retention

A shard remains protected by:

- active read view;
- bound SearchUniverseSet;
- SearchResultSetManifest or continuation;
- evaluation/golden corpus;
- integrity/debug investigation;
- explicit policy.

GC must prove absence from all roots. `wow-store` owns physical retention and deletion.

## Final-state determinism

Two builds from identical owner records and profiles produce the same logical documents, partition membership, SearchShard ID, validation manifests, and golden query results independent of build history. Physical SQLite bytes are classified separately.

## Cross-universe candidate fusion

Candidates retain exact universe/shard identity. Same strings or kinds across project, Blizzard UI, and ReferenceView remain separate entities. Exact graph/reference links may contribute an explanation signal but never collapse identities.
