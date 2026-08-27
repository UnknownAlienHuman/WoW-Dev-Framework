# Architecture

**Status: normative**

**Source baseline:** architecture v8.0, reviewed 2026-08-15

## 1. System shape

WoW Dev Framework consists of three tightly connected correctness domains and one optional discovery bridge:

```text
Reference domain
    wow-reference-builder
    → immutable Reference Pack

Project analysis domain
    wow-emmy-core / wow-emmy-ls
    → generic + WoW diagnostics for one project generation

WoW intelligence domain
    wow-index
    → project/UI graph, exact search, lineage, skeletons, planning

Optional discovery domain
    wow-cbm-bridge
    → broad semantic candidates from an installed Codebase Memory MCP server
```

The public CLI, MCP, and LSP frontends route through one service/use-case layer. They are different transports over the same contracts, not separate implementations.

## 2. Truth stores and generations

### Reference generation

A Reference Pack is immutable and profile-specific. It contains platform facts produced from one pinned Blizzard UI snapshot, one builder version, one schema set, and one reviewed correction set.

### Project generation

One mutable project actor owns the active Emmy analysis and project index. File updates produce an immutable `ProjectGeneration` snapshot. Readers never combine facts from different project generations.

### External candidate generation

Codebase Memory and cloned external repositories have their own revision/generation identity. Their output is never silently merged into `Proven` platform or project facts.

Every answer identifies the generations and profile used.

## 3. Reference pipeline

```text
pinned Blizzard UI snapshot
    ├── Blizzard_APIDocumentation
    ├── Blizzard_APIDocumentationGenerated in TOC order
    ├── Blizzard_Deprecated and transition material
    ├── Interface/AddOns TOC/XML/Lua packages
    └── optional interface resources

→ restricted declarative APIDocumentation evaluator
→ schema-aware lowering with raw unknown-field preservation
→ structural TOC/XML/Lua extraction
→ reviewed digest-bound corrections
→ Ketho and Numy differential reports
→ immutable Reference Pack
```

Arbitrary Lua is never executed. Unsupported constructs create ingestion diagnostics and reduce the affected partition's coverage. They do not disappear.

The Reference Pack contains both raw metadata and projections. Annotations are a consumer format, not the canonical store for fields such as Secret arguments/returns, predicates, unknown metadata, source spans, or correction provenance.

See `REFERENCE_PACK.md`.

## 4. Project analysis pipeline

```text
workspace configuration
+ selected Reference Pack
+ first-party addon Lua/XML/TOC
+ declared libraries

→ update Emmy VFS and semantic index
→ run built-in Emmy diagnostics
→ extract canonical syntax/semantic facts
→ update TOC/XML/load/project graph partitions
→ run capability-declared WoW diagnostic providers
→ fold downstream symptoms into root causes
→ publish one normalized result for one ProjectGeneration
```

The full Blizzard UI implementation tree is not placed in the ordinary Emmy library workspace. Generated annotations, project-declared libraries, and narrow referenced stubs are used instead.

No user or workspace editor configuration is overwritten. Generated configuration lives under `.wow/generated/<profile>/<generation>/`.

See `EMMYLUA_AND_DIAGNOSTICS.md`.

## 5. WoW graph

A single generic `parent` relation is invalid for WoW. The graph preserves independent axes:

- lexical containment;
- package/module/namespace ownership;
- TOC/load-unit ordering and dependency;
- frame/XML object parentage;
- inheritance and mixin composition;
- event, callback, hook, style, element, and plugin registration;
- lifecycle ownership;
- state-root and state-path access;
- proven or possible calls;
- build lineage.

Nodes and relation kinds use open string registries with versioned semantics. Every emitted edge includes source evidence, extractor or recognizer identity, generation, confidence, and coverage.

The persistent graph uses SQLite adjacency tables. Only a bounded requested neighborhood is projected into memory.

See `GRAPH_SEARCH_AND_PLANNING.md`.

## 6. Recognizers

Recognizers operate on normalized Emmy syntax/semantic facts plus TOC/XML facts. They emit universal roles.

```text
receiver.NewModule("Name")
→ node role=module
→ created_by factory
→ owner/load evidence
```

A pack may be named after a calibration corpus such as Ace3 or oUF, but its contents are declarative structural patterns. Removing a pack may reduce coverage; it must not change core semantics. Production code never contains `if repository == "ElvUI"`-style branches.

The correctness path uses one Lua parser: Emmy. Tree-sitter or ast-grep may exist in external discovery systems but cannot produce canonical WoW facts without verification.

## 7. Search and lineage

Search lanes run in authority order:

```text
query classification
→ exact active canonical name
→ aliases, deprecations, replacements, and lineage
→ namespace/member/prefix
→ receiver, signature, return, and restriction shape
→ bounded edit-distance/trigram candidates
→ FTS5 over documentation and L0/L1 skeletons
→ graph-neighborhood expansion
→ optional Codebase Memory semantic candidates
→ deterministic reranking with explanations
```

An explicit replacement edge outranks every fuzzy hit. Name similarity cannot establish lineage or authorize an autofix.

Historical lineage combines source-control move evidence, normalized syntax fingerprints, signature/receiver shape, ownership/load neighborhoods, call/API/event neighborhoods, and transition documentation. Path identity alone is insufficient because Blizzard package layouts move across builds.

## 8. Skeletons and context

The framework exposes three source detail levels:

- **L0** — signature, owner/load chains, direct API/event/state roles, and neighborhood counts.
- **L1** — signature, branch/loop/call structure, early returns, and state effects with bodies collapsed.
- **L2** — exact source span/full source.

Default agent context uses L0/L1. Full source requires an explicit source handle request.

`.wow/ARCHITECTURE.generated.md` is a generated Project Map capped at approximately 2 KB. It records the active profile, load skeleton, module/service/state owners, registries, extension points, invariants, known dynamic gaps, and workaround debt.

## 9. Secret and restriction model

Restrictions use an open facet registry. Unknown facets are preserved raw and make dependent checks `NotEvaluated`.

Static Secret analysis progresses by capability:

1. API contract facets;
2. direct local operations and guard dominance;
3. bounded interprocedural summaries for stable direct calls;
4. structured runtime evidence.

Deep flow is not a prerequisite for the local MVP. Static analysis never hard-codes a permanent runtime spell whitelist.

See `SECRET_VALUES_AND_RESTRICTIONS.md`.

## 10. Storage

```text
reference.sqlite
    immutable, read-only, one profile

project.sqlite
    mutable WAL database, rebuildable per repository/generation

external.sqlite
    optional external manifest and source-handle metadata

objects/
    content-addressed compressed raw/skeleton artifacts
```

SQLite provides exact indexes, FTS5, adjacency tables, transactions, portability, and inspectability. A graph server, vector database, and separate search daemon are excluded from v1 unless benchmarks prove a unique requirement.

## 11. Process topology

Normal interactive operation uses one `wow` process or daemon and, optionally, an already installed Codebase Memory process. Crates do not imply processes. Reference build CI may run differential oracle jobs separately.

## 12. Planned Rust boundaries

```text
wow-core          IDs, profiles, evidence, findings, stable handles
wow-store         SQLite schemas, migrations, content-addressed objects
wow-reference     APIDoc lowering and Reference Pack read/build
wow-annotations   Ketho-compatible and dialect projections
wow-emmy          upstream adapter, project actor, diagnostic registry
wow-project       Lua/TOC/XML workspace and incremental generations
wow-graph         universal nodes/relations, lineage, bounded queries
wow-recognizers   declarative matcher and calibration packs
wow-search        exact/migration/shape/FTS/graph ranking
wow-rules         API/load/event/Secret/overlay/project diagnostics
wow-cbm           optional MCP candidate bridge
wow-context       skeletons, Project Map, context budgets
wow-service       transport-independent use cases
wow-app           CLI, MCP, and LSP binaries
```

`wow-testkit` and `wow-eval` are development-only. The names describe planned responsibilities, not a mandate to create every crate before E0. Crates are introduced only when boundaries are proven by tests.

## 13. Public result envelope

Every query or diagnostic is normalized around:

```text
profile and reference generation
project/external generation where applicable
entity or finding identity
source handles and evidence provenance
confidence
coverage status and missing capabilities
explanation/ranking signals
stable detail handles
```

The precise contract is defined in `PROVENANCE_AND_COVERAGE.md` and will become a versioned public schema before external release.

## 14. Failure behavior

- Stale or missing profile: reject or return explicit unavailable status.
- Failed source partition: keep unaffected partitions usable; mark dependent capabilities incomplete.
- Unknown upstream field: preserve and quarantine; do not drop.
- Emmy update loses a mandatory capability: block activation and retain last-known-good.
- Codebase Memory unavailable: exact local/reference workflow remains functional.
- Conflicting oracles: retain both evidence records and open a triage gap.
- Dynamic runtime behavior not statically provable: return `NotEvaluated` or `Possible`, plus the required runtime scenario.

## 15. Deployment and release model

Normal users consume prebuilt, checksum-verified Reference Packs. They do not need Ketho, Numy, PHP, LuaRocks, VS Code, or the full Blizzard source tree.

Production release work belongs to E7 and includes prebuilt binaries, pack publication, SBOM, dependency policy, compatibility reports, rollback, and last-known-good activation. No release workflow is added before executable validation exists.
