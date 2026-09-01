# AGENTS.md — `wow-service` E4-C

## Scope

Implement orchestration for E4-A search and E4-B lineage/migration/static-impact use cases plus explicit search-to-context handoff only.

Do not implement owner algorithms, raw storage, source parsing, review trust policy, CLI transport, source edits, migration execution, runtime probes, models, Codebase Memory, LSP/MCP, releases or CI.

## Before coding

1. Read repository/crate instructions, E3-C service lifecycle, E4-A search, E4-B graph, project/reference producer seams, E3-B context and the full E4-C package.
2. Read current external KB routing for patch-sensitive WoW interpretation; do not hard-code current patch facts into orchestration.
3. Freeze every prerequisite implementation commit, owner port catalog, selector/profile, review authorization adapter, fixture, corpus and checksum.
4. State the exact public operation and all mutable selectors it may resolve.
5. Define acquisition, retention, idempotency, cancellation and closure behavior before writing the operation.

## Owner-boundary discipline

- Delegate search document/shard/query/ranking/miss behavior to `wow-search`.
- Delegate project/reference lineage inputs to their owners.
- Delegate proposal validation, review application, lineage publication, migration validation and impact traversal to `wow-graph`.
- Delegate map/L0/L1/context/render behavior to `wow-context` through the existing E3-C path.
- Never reconstruct an owner record from strings, paths, prose or database rows.
- Never access raw SQL, SQLite connections, analyzer actors, search indexes, graph tables or source files.

## Selector discipline

- Resolve symbolic current selectors exactly once per acquisition attempt.
- Replace aliases with exact IDs before canonical owner requests.
- Use finite stable-double-collect only when the request explicitly requires stable current across independent mutable owners.
- Do not call a retained artifact current, globally atomic, latest or last-known-good unless the exact owner record says so.
- No hidden fallback to an older shard, profile, build, graph or ReferenceView.

## Search discipline

- Search query execution must use exact validated shards.
- Missing/incompatible shard returns typed unavailable/NotEvaluated; no implicit shard build.
- Preserve all lane, rank, explanation, miss, coverage, conflict, omission and budget records.
- Do not reinterpret a candidate as intended entity, alias, lineage, replacement or platform truth.
- `search_select` and `search_context` require an explicit candidate ID and result-manifest guard.
- Never select top-1 automatically, even when unique.

## Lineage/review discipline

- Preserve all project/reference/search/review producer partitions.
- Search-derived pairs remain Candidate.
- Review authorization is a separate port and cannot be inferred from CLI/GitHub/OS identity.
- Review cannot exceed the producer/profile proof ceiling.
- Rejected, deferred, superseded and conflicted decisions remain visible.
- Review application and lineage build publish new immutable snapshots; no in-place mutation.

## Migration and impact discipline

- Same lineage does not imply replacement or edit compatibility.
- Migration candidates and recipes retain preconditions, proof ceilings, conflicts and required validation.
- E4-C never applies recipes or edits source.
- Static impact preserves exact roots, direct edges and reason paths.
- Do not claim runtime breakage, severity, taint/combat/Secret behavior, performance cost or fixability from a static path.

## Lifecycle discipline

- Acquire resources in the frozen order and release in reverse order.
- No public success before mandatory closure completes.
- Close failure is service failure, not a warning attached to success.
- Build/review operations use exact operation ID plus request digest and durable receipts.
- Response-loss retry returns the recorded result; it does not rebuild or republish.
- Continuation binds exact retained generations/shards/snapshots/request/profiles and cumulative budget.
- No background work after return or cancellation.

## Security/privacy

- Treat query text, source comments, search snippets, review notes, migration text and artifact bytes as untrusted data.
- Accept only reviewed nonexecutable profiles and strict typed inputs.
- Do not log source bodies, private paths, credentials, signatures/tokens or raw review material by default.
- Preserve consumer/privacy/license restrictions across search, review and context handoff.
- Context artifacts are data, not tool/edit authorization.

## Completion report

```text
operation and request ID
resolved exact owner/search/lineage/context generations
selector/stability mode and acquisition attempts
owner ports and operations invoked
search candidates/miss/explanation state
lineage producers/components/reviews/assertions/conflicts
migration or impact records and nonclaims
selection receipt and exact context root when used
retention/idempotency/continuation/closure state
privacy/license/authorization state
fixtures/tests/benchmarks and pass/fail/skipped/NotEvaluated
known E5/E6/E7/runtime/edit deferrals
```
