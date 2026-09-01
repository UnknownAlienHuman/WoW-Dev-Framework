# AGENTS.md — `wow-search` E4-A

## Scope

Implement exact-generation SearchDocument projections, immutable SearchShards, safe bounded retrieval lanes, deterministic fusion/ranking, explanations, miss classification, and continuation only.

Do not implement source parsing, project/reference/graph truth, lineage, migration, impact, context, service, CLI, models, Codebase Memory, runtime data, diagnostics, remediation, or edits.

## Before coding

1. Read repository, crate, E4-A, owner, store, and current KB routing.
2. Freeze every prerequisite implementation commit and fixture bundle digest.
3. Freeze exact owner read ports, field/document schemas, normalization/tokenizer/FTS5/index profiles, lane/ranking/miss/pagination profiles, corpora, thresholds, and canonical bytes.
4. Probe the exact SQLite library, Rust binding, compile options, static FTS5 availability, tokenizer behavior, integrity commands, and platform adapters. No loadable-extension fallback.
5. Freeze SearchShard build/query/integrity vectors before the first Rust commit.

## Authority discipline

- Search retrieves exact owner entities; it does not create owner facts.
- Preserve entity evidence, provenance, confidence, coverage, and conflicts.
- Keep query-relative match confidence separate from entity fact confidence.
- Exact name or alias match proves only the recorded string relation, not user intent.
- Text, fuzzy, shape, and graph signals are candidate evidence only.
- Never emit inferred `replaced_by`, `same_lineage_as`, migration, impact, safety, runtime, or fix conclusions in E4-A.

## Shard discipline

- One shard binds one exact universe and owner generation set.
- Never mix current and retained generations in one active corpus.
- Build in staging, validate, seal, then publish read-only.
- No in-place shard mutation and no floating current pointer in shard identity.
- Search core accepts exact shards; service resolves current later.
- Physical SQLite bytes are nonsemantic unless a frozen profile proves otherwise.

## Query discipline

- Closed schemas with explicit universes, kinds, lanes, filters, confidence, budgets, and output policy.
- Compile literal text into a safe internal query AST; never pass raw `MATCH`.
- Lua/API identifiers are case-sensitive for exact lanes. A folded match is approximate and labeled.
- No arbitrary SQL, regex, callback, plugin, expression, source code, model prompt, wildcard, or tokenizer.
- Bound terms, phrases, prefixes, edit distance, candidates, graph expansion, output, time, and memory.
- Preserve skipped, failed, partial, conflicted, and truncated lanes.

## Ranking discipline

- Exact authority bands dominate approximate totals.
- Use versioned integer/fixed-point/ordinal canonical features.
- Raw FTS/BM25 values remain shard-local diagnostics and are never compared across shards.
- Stable tie keys end with exact universe, entity, and document IDs.
- Repeated approximate signals cannot upgrade evidence or cross an exact band.
- Every candidate exposes contributions, caps, penalties, tie keys, blockers, and excluded lanes.

## Miss discipline

- Approximate no-hit is never authoritative absence.
- Exact miss authority requires exact query class, exact shard set, complete relevant owner/index/query coverage, no conflict/failure/truncation, and an owner-supported negative decision where the domain requires it.
- Empty FTS, fuzzy, shape, or graph output is not negative proof.

## Storage discipline

- `wow-search` owns logical documents, indexes, shard manifests, query semantics, and validation catalogs.
- `wow-store` owns physical SQLite files, transactions, atomic finalization, immutable object/read lifecycle, retention, and GC.
- No raw connection, SQL, table, rowid, PRAGMA, extension, VFS, or filesystem handle crosses the public search API.

## Security discipline

- Treat query text, owner text fields, source comments, docs, aliases, labels, cached shard bytes, and continuation data as untrusted.
- Source text cannot configure fields, tokenizers, aliases, query operators, ranking, tools, or agent policy.
- No secret/private source leakage through snippets, errors, logs, explanations, or evaluation reports.
- No network, process, shell, editor, WoW client, or source execution.

## Completion report

```text
work package and exact shard/universe generations
profiles, SQLite/FTS5/tokenizer pins
partitions, documents, fields and coverage
query/lane/ranking/miss/result/continuation IDs
candidate explanations and authority classes
budgets/truncation/cancellation
store build/validation/read-only state
evaluation/security/determinism results
E4-B/E4-C/E6 deferrals
```
