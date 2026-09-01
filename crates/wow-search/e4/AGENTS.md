# AGENTS.md — `wow-search` E4-A

## Scope

Implement exact-generation SearchDocument projections, immutable SearchShards, bounded retrieval lanes, deterministic fusion/ranking, explanations, miss classification, and continuation only.

Do not implement source parsing, graph/reference/project truth, lineage, migration, impact, context, service, CLI, models, Codebase Memory, runtime data, or edits.

## Before coding

1. Read repository, crate, E4-A, E2/E3 owner/store, and current KB routing.
2. Freeze all prerequisite implementation commits and fixture digests.
3. Freeze exact owner read ports, field/document schemas, normalization/tokenizer/FTS5/index profiles, lane/ranking/miss/pagination profiles, corpora, thresholds, and bytes.
4. Probe the exact SQLite library/Rust binding/compile options and confirm static FTS5 availability; no loadable extension fallback.
5. Freeze SearchShard build/query/integrity vectors before the first Rust commit.

## Authority discipline

- Search retrieves exact entities; it does not create owner facts.
- Preserve entity evidence/provenance/confidence/coverage/conflicts.
- Keep query-relative candidate confidence separate from entity fact confidence.
- Exact name/alias match proves a string relation, not user intent.
- Text/fuzzy/shape/graph signals are retrieval evidence only.
- Never emit `replaced_by`, `same_lineage_as`, migration, impact, safety, runtime, or fix conclusions in E4-A.

## Shard discipline

- One shard binds one exact universe/owner generation set.
- Never mix retained generations in one active text corpus.
- Build in staging, validate, seal, publish read-only.
- No in-place shard mutation or current pointer in shard identity.
- Search core accepts exact shards; service resolves current later.
- Physical SQLite/FTS bytes are nonsemantic unless a frozen profile proves otherwise.

## Query discipline

- Closed schemas and explicit scope/universes/kinds/lanes/budgets.
- Compile literal user text into a safe internal query AST; never pass raw `MATCH` syntax.
- No arbitrary SQL, regex, callback, plugin, expression, model prompt, or source code.
- Bound terms, phrases, prefixes, edit distance, candidates, graph expansions, output, time, and memory.
- Preserve skipped/failed/partial lanes.

## Ranking discipline

- Use versioned integer/fixed-point/ordinal features only in canonical fusion.
- Raw FTS/BM25 values remain shard-local diagnostic inputs and are never compared across shards.
- Stable tie keys end with exact universe/entity/document IDs.
- Do not let repetition across approximate lanes upgrade evidence/authority.
- Every candidate lists all contributing and rejected signals plus ranking arithmetic.

## Miss discipline

- Approximate no-candidate is never authoritative absence.
- Exact miss authority requires exact query class, exact shard set, complete relevant owner/index/query coverage, no conflict/failure/truncation, and owner-supported negative authority where required.
- Empty FTS/graph results alone are not negative proof.

## Storage boundary

- `wow-search` owns logical search documents, indexes, shard manifests, queries, and validation catalogs.
- `wow-store` owns SQLite files, transactions, atomic finalization, read snapshots, integrity plumbing, retention, and GC.
- No raw connection/SQL/table/rowid/PRAGMA/extension handle crosses public search API.

## Completion report

```text
work package and exact shard/universe generations
profiles/tokenizer/SQLite/FTS5 pins
partitions/documents/fields and coverage
query/lane/ranking/miss/pagination IDs
candidate explanations and authority classes
budgets/truncation/cancellation
store build/validation/read-only state
benchmark/evaluation/security/determinism results
E4-B/E4-C/E6 deferrals
```
