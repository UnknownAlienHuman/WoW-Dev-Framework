# E2-C project indexing decisions

**Status:** normative.

## PROJECT-E2-001 — Consume materialized source snapshots

The project library consumes an exact closed manifest and bytes/object handles. Host filesystem discovery/materialization is an explicit adapter responsibility.

## PROJECT-E2-002 — Revision and content identity are separate

Repository/ref provenance is recorded, but canonical source content digests determine the snapshot. A branch name alone is never an index input identity.

## PROJECT-E2-003 — Universes never collapse

First-party project, declared dependency metadata/source, analyzer library, reference, external implementation, runtime, and historical data remain separate roles and identities.

## PROJECT-E2-004 — One selected TOC variant per package

Mainline/flavor variants are parsed independently and selected explicitly. Directives/files/dependencies never merge across variants to make a complete-looking manifest.

## PROJECT-E2-005 — Unknown TOC syntax is preserved

Unknown directives, suffix tokens, malformed-but-recoverable lines, and unsupported semantics remain raw records and coverage blockers; they are not discarded or guessed.

## PROJECT-E2-006 — TOC order is semantic

File entries, dependency declarations, SavedVariables, and applicable ordered records retain source ordinal. Filesystem or map order never substitutes.

## PROJECT-E2-007 — LOD/bootstrap is metadata, not runtime proof

LoadOnDemand and `[Bootstrap]` facts define static phases/roles only. They do not prove full addon initialization, frame creation, event delivery, combat legality, or runtime success.

## PROJECT-E2-008 — Dependencies are explicit configuration

Required/optional dependencies resolve only against declared package manifests/universes. Missing dependencies are never downloaded or discovered automatically.

## PROJECT-E2-009 — XML parser is streaming and nonexecuting

DTD, external entities, network resolution, scripts, handlers, templates, and includes never execute. Entity/expansion/depth/size are bounded.

## PROJECT-E2-010 — XML unknowns are preserved

Unknown elements/attributes/namespaces and unsupported constructs remain raw observations and narrow capability gaps.

## PROJECT-E2-011 — XML includes are source edges

Include/script file references are resolved within the exact snapshot under path/root/cycle/budget policy. Inclusion is not a new filesystem permission.

## PROJECT-E2-012 — Inline XML Lua becomes a virtual source unit

Inline script bytes retain XML file/span/handler/object identity and are submitted to `wow-emmy` as generated virtual project units. XML code never parses Lua.

## PROJECT-E2-013 — `wow-emmy` is the sole Lua analyzer

No Lua parser, symbol resolver, CFG, diagnostic, or type inference is implemented in project.

## PROJECT-E2-014 — Static load model is not runtime state

The load model describes selected manifests, dependencies, ordered units, includes, and phases. It does not assert that a file executed successfully or an object exists at a runtime event.

## PROJECT-E2-015 — SavedVariables roots require TOC declarations

The project model records declarations/scopes only and never reads SavedVariables contents. A Lua global name alone is insufficient.

## PROJECT-E2-016 — Fact adapters are loss-accounted

Every TOC/XML/project/analyzer fact selected for recognizers receives a typed output, Unsupported, NotEvaluated, or adapter-loss record.

## PROJECT-E2-017 — Recognizers remain pure and project-invoked

Project assembles exact E2-B bundles and receives output partitions. Project does not implement or mutate rules to fit its own files.

## PROJECT-E2-018 — Graph proposals are independently validated

Project submits recognizer/project proposals to E2-A validation. Rejections/conflicts stay explicit; project cannot repair them by weakening identity or graph schema.

## PROJECT-E2-019 — E2-C produces a candidate, not persistent publication

`ProjectIndexCandidate` is immutable after validation but is not current/persisted ProjectSnapshot or GraphSnapshot. E2-D owns ProjectStore and atomic publication.

## PROJECT-E2-020 — Project generation includes all output-affecting profiles

TOC/XML dialects, analyzer pin/config, recognizer pack/profile, graph registry, capability/budget policy, source snapshot, and adapter schema enter generation identity when they can change output.

## PROJECT-E2-021 — Incremental invalidation is dependency-driven

Derived partitions declare exact dependencies. Mtime, path presence, or watcher order is never sufficient.

## PROJECT-E2-022 — Unknown invalidation widens conservatively

If a dependency edge or parser impact is unknown, invalidate the containing package/load/XML/project partition rather than reuse stale facts.

## PROJECT-E2-023 — Removed inputs remove derived output

Removed Lua/XML/TOC files, directives, handlers, templates, dependencies, state roots, analyzer facts, recognizer matches, and proposals cannot remain in the target candidate.

## PROJECT-E2-024 — Candidate partiality is explicit

Malformed/unsupported/failed/truncated partitions can coexist only under explicit capability policy. No complete/clean state is synthesized from empty output.

## PROJECT-E2-025 — Native/custom/CVar signals remain separate

Project facts and recognizer bundles preserve the KB signal taxonomy. Event-like names do not imply producer class or payload accessibility.

## PROJECT-E2-026 — Hook/load structure has no safety verdict

Project records structure/source/load reachability only. Taint, protection, managed/private status, combat legality, runtime success, and performance remain separate evidence/rules/probes.

## PROJECT-E2-027 — Dependencies do not become first-party

Supplied dependency source remains a separate universe/role and cannot become primary first-party finding ownership by being loaded earlier.

## PROJECT-E2-028 — No code generation or source mutation

E2-C indexes supplied state only. It does not rewrite TOCs/XML/Lua, generate loaders, install dependencies, edit configuration, or apply fixes.

## PROJECT-E2-029 — Determinism covers candidate semantics

Equivalent final source/profile/tool/rule/registry inputs produce identical source, TOC, XML, load, analyzer binding, recognizer, graph-proposal, invalidation, coverage, and candidate manifests under 1/2/N workers.

## PROJECT-E2-030 — Real addon fixture is pinned and read-only

Before implementation promotion, at least one user-owned addon repository fixture is pinned to a commit and used only as project evidence. Synthetic fixtures remain the primary closed oracle; no production branch depends on the fixture repository name.
