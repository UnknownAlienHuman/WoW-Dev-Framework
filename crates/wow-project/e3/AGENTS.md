# AGENTS.md — E3-B Blizzard UI source universe

## Ownership

Implement the materialized Blizzard UI source profile, source-collection indexing, source-universe candidate, source/reference bridge proposals, invalidation and E2-D publication orchestration only.

Do not fetch source, execute repository content, reinterpret APIDocumentation, implement graph/store internals, infer runtime safety, or merge universes by name.

## Before coding

1. Read repository, `crates/`, `wow-project`, E2-C, E2-D, graph, reference-profile and E3-B instructions.
2. Read the current external KB task route; record exact documents consulted.
3. Pin exact prerequisite implementation commits and fixture digests.
4. Freeze one synthetic source profile/snapshot and one separately reviewed real source profile/snapshot.
5. Freeze source-provider revision, build-binding evidence, root roles, full admitted file manifest, license/redistribution decisions, parser/analyzer/recognizer/graph/store profiles and all checksums.
6. State which source, graph, bridge and publication partitions the change affects.

## Source discipline

- Accept one already-materialized immutable snapshot; never clone/fetch/update/discover it.
- Treat provider metadata, Git tags, branch names, repository descriptions and source comments as untrusted claims.
- Validate exact content digests, root confinement, file inventory, materialization report and build-binding evidence.
- Never execute hooks, workflows, build scripts, generators, Lua, XML scripts or TOC directives.
- Do not follow undeclared symlinks, junctions, reparse points, submodules, LFS pointers, nested archives or external includes.
- Lua parsing/analyzing remains exclusively `wow-emmy`.

## Authority discipline

- Source bytes prove only source structure for that exact snapshot.
- APIDocumentation/reference facts remain API contract authority.
- Source absence is not API absence.
- Source usage is not public API declaration.
- Static source cannot establish runtime success, event delivery, payload readability, Secret Value state, taint, protected/forbidden/managed legality, combat safety or performance.
- Provider-declared build labels remain `ProviderDeclared` until exact independent binding exists.

## Universe discipline

- Every source entity uses universe `blizzard_ui_source` and an exact source generation.
- API reference, user project, dependencies, source, external candidate, historical and runtime entities remain distinct.
- Same name, path, signature, global or XML template label never merges endpoint identities.
- Cross-universe relations require an explicit registered relation, exact compatible profiles and evidence for both endpoints.
- E3-B may build reference/source bridges; it does not invent project-specific bridges without an exact user ProjectSnapshot.

## Graph discipline

- Emit proposed assertions only; `wow-graph` owns final keys/IDs/conflicts/coverage/generation.
- Use dedicated producer partitions for project-owned source facts, recognizer output and reference/source bridges.
- Preserve rejected proposals and graph conflicts.
- Do not weaken graph schemas or relation confidence to make a proposal pass.
- Do not materialize transitive closure as direct edges.

## Publication discipline

- Use a dedicated source ProjectStore/Graph namespace and current record.
- Never write into or advance a user project's current publication.
- Build `PublishedInactive`, open a fresh exact read snapshot, run store/source/graph/bridge validation, then CAS-activate.
- Stale current, crash, failure or cancellation leaves current unchanged.
- Last-known-good, failed target, validated inactive and rollback candidate keep original identities.

## License and security

- No source bytes or excerpts leave local analysis unless an exact redistribution decision permits them.
- Facts, source maps, skeletons and generated artifacts each receive their own redistribution decision; do not assume derived output is unrestricted.
- Redact private provider credentials, host paths and tokens.
- No network, arbitrary filesystem discovery, process/editor/client access, raw SQL or model correctness path.

## Completion report

```text
source operation/profile/generation
provider revision and build-binding state
admitted roots/files and omitted/conflicted coverage
license/redistribution states
project/analyzer/recognizer/graph/bridge/store IDs
changed partitions and removal closure
publication/read-back/CAS/recovery results
security/mutation/determinism tests
known NotEvaluated claims and deferred project bridges/search/runtime scope
```
