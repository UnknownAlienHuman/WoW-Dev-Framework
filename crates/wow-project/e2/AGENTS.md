# AGENTS.md — `wow-project` E2-C

## Scope

Implement only exact project-source snapshot validation, TOC/XML/load parsing/modeling, Lua-unit materialization, analyzer/recognizer/graph-proposal orchestration, incremental invalidation, and ProjectIndexCandidate validation.

Persistent ProjectStore and atomic ProjectSnapshot/GraphSnapshot publication remain E2-D.

## Before coding

1. Read repository/crate/E0-D/E2-C contracts and current KB routes.
2. Verify exact implemented/frozen `wow-core`, `wow-emmy`, `wow-graph`, and `wow-recognizers` boundaries.
3. Freeze TOC/XML dialect profiles, source snapshot profile, graph/recognizer profiles, synthetic and pinned addon fixtures, invalidation vectors, and checksums.
4. State which package/TOC variant/files/partitions/generation are affected.
5. State whether a claim is project-source-derived, analyzer-derived, recognizer-derived, graph-validated, or runtime-unverified.

## Source snapshot discipline

- Consume one closed materialized snapshot with exact repository/revision/root/file digests.
- Do not discover a floating branch or installed addon.
- Host adapters may materialize bytes under explicit roots; the project library receives validated manifests/bytes or object handles.
- Do not follow symlinks/reparse points/submodules by default.
- Never execute hooks, scripts, generators, package managers, tests, Lua, XML handlers, or TOC content.

## TOC discipline

- Parse lexical records and preserve unknown directives/tokens.
- Select one exact variant per package/flavor target; never merge directives/files across variants.
- Preserve semantic file order and dependency/SavedVariables ordinals.
- LOD/bootstrap metadata does not prove full load, frame existence, callback readiness, or runtime success.
- Missing/unknown mandatory directives or files downgrade exact capabilities.

## XML discipline

- Streaming/bounded parsing; DTD/external entities/network expansion disabled.
- Preserve unknown elements/attributes and exact source spans.
- Includes/scripts resolve only within declared snapshot roots/universes.
- Inline Lua becomes a source-mapped virtual Lua unit for `wow-emmy`; never execute or parse it in XML code.
- XML inheritance/parent/template ambiguity remains explicit.

## Analyzer discipline

- `wow-emmy` is the only Lua parser/semantic analyzer.
- Build one exact generation-bound workspace/update and validate returned snapshot against project/profile/reference/pin/config/file/unit manifests.
- Library annotation files remain a separate analyzer role/universe.
- Do not rewrite analyzer facts/findings.

## Recognizer/graph discipline

- Adapt exact project/analyzer facts into E2-B bundles; no hidden source fallback.
- Retain adapter loss, coverage, ambiguity, and exact partition dependencies.
- Recognizers emit proposals only.
- Validate proposals with E2-A graph registry/proposal seams.
- Do not publish graph generations or resolve graph conflicts in E2-C.
- Rejected proposals remain visible in candidate/evaluation reports.

## Invalidation discipline

- Diff canonical snapshots/partitions, not mtimes or watcher events.
- Every derived partition declares exact dependencies.
- Remove stale analyzer facts, recognizer outputs, and graph proposals when inputs change.
- Unknown dependency widens invalidation conservatively; it never reuses stale output.
- Different update orders reaching the same final snapshot yield the same target generation/candidate.

## Evidence and safety

- Static load order is not runtime load success.
- Event registration is not payload readability.
- Hook structure is not taint/combat/protected/managed safety.
- SavedVariables declaration is metadata; never read contents.
- Project source does not become platform authority.
- No API/Secret/taint/runtime verdict in project facts.

## Tests

Every parser/model/invalidation change needs positive, near-negative, unknown/partial, malformed/security, mutation, generation isolation, stale-removal, and deterministic tests. A test must prove the intended path executed.

## Completion report

```text
project/source snapshot/package/variant/generation IDs
TOC/XML/analyzer/recognizer/graph profiles
files/contracts changed
partitions parsed/invalidated/rebuilt
capability/coverage/conflict/truncation state
proposal validation accept/reject counts
checks and pass|fail|skipped
runtime verification still required
E2-D persistence/publication seam impact
```
