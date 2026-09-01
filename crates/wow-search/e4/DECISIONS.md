# E4-A search decisions

**Status:** normative.

## SEARCH-001 — Search facts and query matches are separate

Owner entity/source/reference/graph records remain domain facts. Search signals describe why a query retrieved them.

## SEARCH-002 — One immutable shard per exact universe generation

User project, Blizzard UI source, and ReferenceView are indexed separately. A shard never mixes current and retained generations.

## SEARCH-003 — No combined global FTS corpus

Cross-universe search federates exact shards. Raw FTS statistics/scores never cross shard boundaries.

## SEARCH-004 — Generation-local FTS corpus

FTS ranking depends only on documents in one exact shard. Retained prior generations cannot affect current result ordering.

## SEARCH-005 — Shard publication is immutable

Build in staging, validate logical documents/indexes/golden queries/integrity, then publish read-only. No in-place update.

## SEARCH-006 — Search core accepts exact shards only

Symbolic current/latest resolution belongs to later service orchestration.

## SEARCH-007 — Document projection is typed and bounded

No arbitrary property bag or full source body. Every indexed field has schema, origin, authority, privacy/license, normalization, tokenizer, and size policy.

## SEARCH-008 — Raw query syntax is forbidden

Callers supply closed structured fields and literal bounded text. Search compiles an internal AST and parameterized FTS operations; raw FTS `MATCH`, SQL, regex, callbacks, or expressions are rejected.

## SEARCH-009 — Exact identity/name/alias remain distinct

Exact ID, canonical identifier, and reviewed explicit alias lanes have separate signals and coverage.

## SEARCH-010 — Alias is explicit evidence

Fuzzy spelling, source occurrence, or popular usage never creates an alias record.

## SEARCH-011 — Approximate identifier lane is candidate-only

Trigram/edit-distance results never become alias, lineage, replacement, or negative authority.

## SEARCH-012 — Shape queries are structured

Receiver/signature/type/restriction/load/role shape comes from explicit query fields or exact selected seeds, not opaque-text model inference.

## SEARCH-013 — Graph expansion starts from seeds

No graph-wide ranking scan. Expansion uses a versioned relation whitelist, direction, depth, confidence, and fanout budget.

## SEARCH-014 — Paths remain paths

Graph reason paths are search signals and never persisted or rendered as direct edges.

## SEARCH-015 — Fusion uses canonical integer/ordinal features

No platform-dependent floating aggregate in semantic result identity.

## SEARCH-016 — FTS ordering is shard-local

Within one frozen shard/SQLite/profile, FTS rank orders text candidates. Fusion uses lane ordinal/reciprocal-rank features; raw BM25 values are supplemental and not compared between shards.

## SEARCH-017 — Authority bands precede approximate totals

Exact ID/canonical/explicit alias classes cannot be outranked by repeated fuzzy/text signals unless an explicit query intent excludes those exact lanes.

## SEARCH-018 — High rank is not intended-entity proof

Every result is a candidate relative to the query. Applications/service require explicit candidate selection before context use.

## SEARCH-019 — No lineage/replacement/impact in E4-A

Even if an explicit current owner record says deprecated, E4-A may display that exact fact but does not infer or rank a replacement relation. Cross-generation reasoning is E4-B.

## SEARCH-020 — Miss authority is exact and scoped

Approximate no-hit is `NoCandidatesUnderExecutedLanes`. Exact authoritative absence requires complete relevant coverage and owner-supported negative state.

## SEARCH-021 — Pagination is snapshot/result-manifest bound

Continuation cannot rerun against newer shards or changed profiles/budgets.

## SEARCH-022 — Explanations are complete arithmetic traces

Every included/excluded lane contribution, band, feature, cap, penalty, tie key, and blocker is visible.

## SEARCH-023 — Candidate duplication preserves all signals

Same exact entity from multiple lanes becomes one candidate with all signals; signals are not votes that upgrade authority.

## SEARCH-024 — Source text is indexed data, not policy

Comments/docs cannot register aliases, change profiles, execute FTS syntax, or direct agents/tools.

## SEARCH-025 — Built-in/static FTS only in v1 candidate profile

No loadable SQLite extensions or source-controlled tokenizers/auxiliary functions. Exact compiled capability is probed and frozen.

## SEARCH-026 — Physical bytes are not logical identity

SQLite page layout, rowid, WAL, optimize/merge state, host, time, and build order do not enter SearchShard identity.

## SEARCH-027 — FTS snippets are nonauthority presentation data

Match snippets/highlights cannot become source evidence; exact source detail uses owner source handles later.

## SEARCH-028 — SearchStore is a sidecar

Search indexes do not mutate or become truth tables inside ProjectStore/ReferenceStore. They bind exact owner generations and may be rebuilt/discarded.

## SEARCH-029 — No model/external candidate lane in E4-A

Embeddings, LLM rerankers, and Codebase Memory remain later optional candidate inputs through service.

## SEARCH-030 — Search never calls context

Search returns candidates/detail handles. Service performs explicit candidate-to-exact-root handoff to `wow-context` in E4-C.

## SEARCH-031 — Final-state determinism matters

Independent build/update histories that reach identical exact documents/profiles produce identical logical shard manifests and query results.

## SEARCH-032 — FTS implementation changes are profile changes

SQLite version/compile options, tokenizer, detail/prefix/content options, column weights, ranking function, query compiler, or normalization changes create a new profile and shard identity.
