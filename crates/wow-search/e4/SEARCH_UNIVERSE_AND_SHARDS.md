# E4-A search universes and immutable shards

**Status:** normative.

## Universe classes

Initial E4-A shard classes:

```text
PrimaryUserProject
BlizzardUiSource
ReferencePlatformContract
```

Dependency source already represented inside a project publication stays in that project's universe under its exact source-class identity. Runtime, historical lineage, external candidate, and Codebase Memory universes are deferred.

## One shard, one exact owner generation

### User project shard

Binds:

```text
ProjectStore/Epoch/Generation/PublicationSet
ProjectGeneration/ProjectSnapshot/ProjectView
GraphGeneration/GraphSnapshot/GraphView
AnalyzerSnapshot identity carried by publication
source/detail/capability/coverage/conflict manifests
```

### Blizzard UI shard

Binds the equivalent separate E3-A platform-source publication and `SkeletonInputView` profile where indexed fields use it.

### Reference shard

Binds:

```text
ProfileId
ReferenceGenerationId
ReferenceViewId
ReferenceStore identity
entity/fact/correction/alias/coverage/conflict manifests
```

No implementation-source shard field becomes Reference authority.

## SearchUniverseSet binding

A query binds exact shard IDs, not current stores. Compatibility validates:

- product/client flavor/build/Interface/ProfileId;
- source/reference generation expectations;
- entity kind/field/query/ranking schema compatibility;
- graph registry and detail-handle compatibility;
- privacy/license/consumer scope;
- required lane/index capabilities;
- coverage/conflict state.

Multiple incompatible profiles may be queried only as explicitly separate universes under a cross-profile comparison request defined later. E4-A normal queries reject the mix.

## Why separate shards

- owner generations remain isolated;
- a Reference Pack update does not mutate a project shard;
- a user project update does not rebuild Blizzard UI/reference shards;
- old retained documents cannot affect new FTS corpus statistics;
- raw FTS ranks stay local;
- privacy/license policy can differ by universe;
- shards can be rebuilt/discarded without changing owner truth;
- service can acquire exact combinations explicitly.

## No combined global index

E4-A does not create one mutable database containing whichever project/reference/source versions are “current.” Such an index would hide generation switches, allow stale documents to affect scores, and complicate owner/privacy boundaries.

A higher service layer may cache one exact `SearchUniverseSet` binding, but the shards remain separate.

## Shard compatibility and absence

A missing optional Blizzard UI shard can yield a query with an explicit omitted universe only when the request/lane profile permits it. It cannot silently fall back to a prior or different-build shard.

Reference shard is mandatory for reference/API search operations and for project queries requesting ReferenceView enrichment. A project-only exact entity search may explicitly omit reference only under a dedicated profile.

## Shard lifecycle

```text
planned
-> building staging artifact
-> published inactive
-> independently validated
-> sealed read-only
-> retained/referenced
-> GC eligible
```

Failed or cancelled artifacts never become sealed/current. Search core has no current pointer; owner/service catalogs map exact generations to exact validated shards.

## Retention

Retain shards by exact:

- active service acquisition;
- search continuation;
- evaluation/golden corpus;
- debug/integrity investigation;
- explicit policy.

GC must prove no universe set, continuation, validation/evaluation report, or active reader references the shard. `wow-store` owns physical retention/GC.

## Final-state determinism

Two builds from identical exact owner records/profile produce the same logical documents, partition membership, shard ID, golden result manifests, and query results independent of build history. Physical SQLite bytes are classified separately.

## Cross-universe candidate merge

Candidates remain tagged with exact universe/shard. Same canonical string/entity kind across user project, Blizzard UI, and ReferenceView is not one entity. Exact graph/reference cross-links may be included in explanations; they do not collapse candidates.
