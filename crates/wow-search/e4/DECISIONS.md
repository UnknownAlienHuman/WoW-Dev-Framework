# E4-A search decisions

**Status:** normative.

## SEARCH-001 — Owner facts and query matches are separate

Owner entity/source/reference/graph records remain domain facts. Search signals only explain why a query retrieved an exact entity.

## SEARCH-002 — One immutable shard per exact owner generation

User project, Blizzard UI source, and ReferenceView are indexed separately. A shard never mixes current and retained generations.

## SEARCH-003 — No combined mutable global corpus

Cross-universe search federates exact shards. Raw corpus statistics and FTS scores never cross shard boundaries.

## SEARCH-004 — Search core accepts exact shards only

Symbolic `current` or `latest` resolution belongs to service orchestration. E4-A inputs contain exact shard IDs.

## SEARCH-005 — Shard publication is immutable

Build in staging, run logical/index/integrity/golden validation, seal, and publish read-only. Published shards are never edited in place.

## SEARCH-006 — Documents are typed bounded projections

A SearchDocument is not a source copy or arbitrary property bag. Every field has a schema, origin, authority, privacy/license, normalization, indexing, and size policy.

## SEARCH-007 — Raw query syntax is forbidden

Callers provide closed structured fields and bounded literal text. The engine compiles a safe AST and parameterized owner operations. Raw SQL, FTS `MATCH`, regex programs, callbacks, expressions, and plugins are rejected.

## SEARCH-008 — Exact identity, canonical name, and alias are distinct

Each lane has separate match semantics, evidence, coverage, and ranking band.

## SEARCH-009 — Exact identifier matching is case-sensitive

WoW Lua/API identifiers are case-sensitive. Unicode normalization may be frozen, but case-folded matches are approximate signals and cannot become exact canonical or alias matches.

## SEARCH-010 — Alias requires explicit owner evidence

Fuzzy spelling, documentation occurrence, popularity, or previous search behavior never creates an alias.

## SEARCH-011 — Approximate identifier similarity is candidate-only

Trigram and edit-distance results never become aliases, lineage, replacement, negative authority, or intent proof.

## SEARCH-012 — Shape queries are structured

Kind, receiver, signature, type, restriction, load, and universal-role shape comes from explicit query fields or exact selected seeds, not opaque-text model inference.

## SEARCH-013 — Graph expansion starts from exact seeds

No graph-wide ranking scan. Expansion uses a reviewed relation whitelist, direction, depth, confidence, fanout, and path budget.

## SEARCH-014 — Paths remain paths

Reason paths are search signals and never persisted or rendered as direct relations.

## SEARCH-015 — Exact authority bands precede approximate scores

Exact ID, qualified canonical name, canonical short name, explicit alias, and exact member relations cannot be outranked by repeated fuzzy/text signals unless the request explicitly excludes the exact lane class.

## SEARCH-016 — Canonical ranking uses integer/ordinal features

No platform-dependent floating aggregate enters semantic result identity.

## SEARCH-017 — Raw FTS scores remain shard-local

Within one frozen shard/profile, raw rank can establish a local ordinal. Cross-shard fusion uses canonical lane ranks and typed features, never raw BM25 comparison.

## SEARCH-018 — High rank is not intended-entity proof

Every result remains a query-relative candidate. An application or user must explicitly select exact roots before context generation.

## SEARCH-019 — No lineage, replacement, migration, or impact in E4-A

E4-A may display an exact owner deprecation fact bound to one generation, but it does not infer cross-generation identity or a replacement target. Those belong to E4-B.

## SEARCH-020 — Miss authority is exact and scoped

Approximate no-hit is only `NoCandidatesUnderExecutedLanes`. Exact authoritative absence requires complete relevant owner and index coverage plus owner-supported negative authority where applicable.

## SEARCH-021 — Result manifests precede pagination

The engine freezes an immutable ordered result-set manifest before returning page one. Continuation pages are views of that exact manifest.

## SEARCH-022 — Continuation is exact and cumulative

A cursor binds exact shards, request, profiles, result manifest, ordering, cumulative budgets, and prior omissions. It cannot refresh generations or reset budgets.

## SEARCH-023 — Explanations are complete arithmetic traces

Every contribution, authority band, feature, cap, penalty, tie key, skipped lane, failed lane, coverage blocker, and omission is visible.

## SEARCH-024 — Candidate deduplication preserves all signals

The same exact entity retrieved by multiple lanes becomes one candidate with all signals and inclusion reasons. Signals are not votes that upgrade authority.

## SEARCH-025 — Source text is indexed data, not policy

Comments and documentation cannot register aliases, change fields, inject query syntax, configure ranking, or direct agents/tools.

## SEARCH-026 — Built-in/static FTS only for the v1 candidate profile

No loadable SQLite extension, source-controlled tokenizer, auxiliary function, or virtual-table module. Exact compiled capabilities are probed and frozen.

## SEARCH-027 — Physical bytes are not logical identity by default

SQLite page layout, rowid, WAL, optimize state, host, clock, thread order, and staging path do not enter SearchShard identity.

## SEARCH-028 — Snippets are nonauthority presentation data

FTS snippets/highlights are optional bounded display records. Exact source evidence is retrieved later through owner source handles.

## SEARCH-029 — SearchStore is a rebuildable sidecar

Search indexes do not mutate or become truth tables inside ProjectStore or ReferenceStore. They bind exact owner generations and can be discarded/rebuilt.

## SEARCH-030 — No context, service, model, or external candidate dependency

Search returns exact candidates and detail handles. Service performs explicit selection and context handoff in E4-C. Embeddings/CBM remain later optional candidate lanes.

## SEARCH-031 — Final-state determinism controls identity

Different build/update histories reaching identical exact documents and profiles produce identical logical shard manifests and query results.

## SEARCH-032 — FTS/runtime changes are profile changes

SQLite version/compile options, tokenizer, detail/content/prefix/columnsize choices, rank function, normalization, query compiler, or column weights create a new profile/shard identity.

## SEARCH-033 — Search indexes only owner-public fields

E4-A does not index `wow-context` artifacts, analyzer internals, recognizer engine state, raw storage rows, or private source bodies through an undeclared dependency.

## SEARCH-034 — Mandatory explanation and uncertainty cannot be pruned

If result identity, exact entity handles, match class, coverage/conflict state, ranking trace, omissions, and continuation cannot fit, the request fails rather than returning an opaque ranking.

## SEARCH-035 — Evaluation cannot relax authority gates

Recall or user-preference gains cannot justify false exact/alias/lineage/replacement claims. False-authority count must remain zero in the accepted corpus.
