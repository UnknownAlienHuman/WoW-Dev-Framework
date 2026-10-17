# Documentation index

English documentation is the canonical navigation surface.

```text
planned architecture/documentation: complete through E7-B
implementation frontier: 18 active workspace members; partial executable owners/apps
next owned work: reviewed refactoring plan / Issue #191
first runnable gate: R0 / E0-A through E0-F
```

## Read first

For current work, start with [contributor instructions](../AGENTS.md), the
[reviewed audit and refactoring plan](AUDIT_2026-10-17.md),
[execution model](EXECUTION_MODEL.md) and [work map](PROJECT_WORK_MAP.md).
The design documents below define scope; their historical bootstrap sequences
do not replace the active issue or the executable census.

1. [Project vision](PROJECT_VISION.md)
2. [Architecture](ARCHITECTURE.md)
3. [Provenance, confidence, and coverage](PROVENANCE_AND_COVERAGE.md)
4. [Architecture decisions](DECISIONS.md)
5. [Roadmap](ROADMAP.md)
6. [Launch gates](LAUNCH_GATES.md)
7. [Workspace and build plan](WORKSPACE_AND_BUILD_PLAN.md)
8. [Implementation handoff](IMPLEMENTATION_HANDOFF.md)
9. [Conformance commands](CONFORMANCE_COMMANDS.md)
10. [Project completion matrix](PROJECT_COMPLETION_MATRIX.md)
11. [Machine manifest](../crates/MANIFEST.json)
12. [Dependency graph](../crates/DEPENDENCY_GRAPH.md)
13. [Workstreams](../crates/WORKSTREAMS.md)

## Core component overviews

- [Reference Pack](REFERENCE_PACK.md)
- [EmmyLua and diagnostics](EMMYLUA_AND_DIAGNOSTICS.md)
- [Graph, search, lineage and planning](GRAPH_SEARCH_AND_PLANNING.md)
- [External Codebase Memory bridge](CODEBASE_MEMORY_BRIDGE.md)
- [Secret Values and restrictions](SECRET_VALUES_AND_RESTRICTIONS.md)
- [Security model](SECURITY_MODEL.md)
- [Test strategy](TEST_STRATEGY.md)
- [Agent workflow](AGENT_WORKFLOW.md)

## Package routes

### E0–E4 exact local intelligence

- E0 and E1 routes live under their owning crate/application directories.
- [E2 typed graph](../crates/wow-graph/e2/README.md)
- [E2 structural recognizers](../crates/wow-recognizers/e2/README.md)
- [E2 project indexing](../crates/wow-project/e2/README.md)
- [E2 ProjectStore](../crates/wow-store/e2/README.md)
- [E3 Blizzard UI source](../crates/wow-project/e3/README.md)
- [E3 context owner](../crates/wow-context/e3/README.md)
- [E3 context service](../crates/wow-service/e3/README.md)
- [E4 exact-generation search](../crates/wow-search/e4/README.md)
- [E4 lineage, migration, and static impact](../crates/wow-graph/e4/README.md)
- [E4 service and CLI](../crates/wow-service/e4/README.md)

### E5 governed recognizer evolution

- [E5-A calibration owner](../crates/wow-recognizers/e5/README.md)
- [E5-B durable review, holdout, and submission](../crates/wow-service/e5/README.md)
- [E5-B CLI](../apps/wow/e5/README.md)
- [E5-C core-pack publication lifecycle](../crates/wow-service/e5c/README.md)
- [E5-C CLI](../apps/wow/e5c/README.md)

### E6 optional external semantic candidates

- [E6-A Candidate-only provider bridge](../crates/wow-cbm/e6/README.md)
- [E6-B provider session, result, mapping, selection, and context service](../crates/wow-service/e6/README.md)
- [E6-B CLI](../apps/wow/e6/README.md)
- [Project locator mapping seam](../crates/wow-project/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [Reference locator mapping seam](../crates/wow-reference/E6_B_EXTERNAL_LOCATOR_MAPPING.md)
- [Exact context handoff seam](../crates/wow-context/E6_B_EXTERNAL_CONTEXT_HANDOFF.md)

### E7-A sessions and developer frontends

- [Canonical service session and operation registry](../crates/wow-service/e7/README.md)
- [Canonical `wow` daemon, LSP, and MCP host](../apps/wow/e7/README.md)
- [Project-owned document overlays](../crates/wow-project/E7_A_DOCUMENT_OVERLAYS.md)
- [Overlay analyzer seam](../crates/wow-emmy/E7_A_OVERLAY_ANALYSIS.md)
- [Live diagnostics seam](../crates/wow-rules/E7_A_LIVE_DIAGNOSTICS.md)
- [Session and response journal seam](../crates/wow-store/E7_A_SESSION_AND_RESPONSE_JOURNAL.md)

Canonical protocol profiles:

```text
wow-local-jsonrpc/1
LSP 3.18
MCP 2025-11-25
```

The E7-A LSP profile uses incremental `textDocument/didChange`; a full-document change is an exact replacement. The default MCP profile is fixed and read-only. No generic tool/RPC/shell, automatic edit application, or default remote listener exists.

### E7-B release and support lifecycle

- [Release/build/sign/bundle/channel/install/update/support service](../crates/wow-service/e7b/README.md)
- [Public release verification and explicit update client](../apps/wow/e7b/README.md)
- [Internal release-engineering client](../tools/wow-release/README.md)
- [Release storage seam](../crates/wow-store/E7_B_RELEASE_STORAGE.md)
- [Release artifact boundary](../release/README.md)

E7-B separates exact source closure, build, independent unsigned reproducibility, tests/evidence, SBOM/provenance/license/notices, signatures, bundle, support matrix, channel publication, installation, update, rollback, revocation, retirement, and incidents. None silently proves another gate.

## Final product flow

```text
exact source/reference/project inputs
-> analyzer/project/graph/diagnostics/search/context
-> optional governed recognizer and external Candidate lanes
-> static service operation registry
-> one wow binary: CLI, daemon, LSP, MCP
-> reproducible unsigned Windows target builds
-> evidence, signatures, bundle, and support candidate
-> provider-neutral publication and public read-back
-> explicit verified install, update, and rollback
```

## Current state

All planned E0-A through E7-B package contracts are documented. Cargo, Rust
implementations, application routes and CI exist. Full package, platform and
release acceptance remains incomplete; see the
[completion matrix](PROJECT_COMPLETION_MATRIX.md).

The current task is to simplify the implemented local product under
[Issue #191](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/191).
Reuse its existing owners and maintain the separate acceptance obligations.

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.
