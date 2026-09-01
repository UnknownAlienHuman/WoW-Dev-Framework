# `wow-service` E3-C context acquisition and use-case orchestration

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-service/e3-c/context-acquisition-and-use-cases`

## Mission

Provide one stable transport-independent boundary between callers and the E3-A/E3-B published context system.

```text
validated ContextServiceRequest
+ service acquisition/profile configuration
-> resolve each symbolic publication selector exactly once in frozen order
-> acquire exact retained project/graph/reference views and leases
-> validate cross-owner generation/profile/capability compatibility
-> bind one exact wow-context ContextUniverseSet
-> invoke exactly one wow-context operation
-> validate semantic and optional rendered artifacts
-> construct a draft service outcome
-> close every acquired resource in reverse order
-> publish one canonical ContextServiceResultEnvelope or typed failure/cancellation
```

No successful public result is finalized before mandatory resource closure completes.

## Public operations

```text
context_status
context_map
context_inspect
context_build
context_continue
context_validate
context_render
```

Operation details are in [`CONTEXT_OPERATIONS.md`](CONTEXT_OPERATIONS.md).

## Context-path direct dependencies

```text
wow-core
wow-store
wow-reference
wow-project
wow-graph
wow-context
```

The E3-C context path has no direct use of `wow-emmy`, `wow-recognizers`, `wow-rules`, `wow-search`, `wow-cbm`, `wow-annotations`, or applications. Existing E0/E1 service operations retain their separate contracts.

## Exact selector model

Primary user project:

```text
ExactStoreGeneration(ProjectStoreGenerationId)
ExactPublicationSet(ProjectPublicationSetId)
CurrentPublished(ProjectStoreId, optional expected-current guard)
```

Optional Blizzard UI project uses the same selector family or `Omitted`.

Reference selection is exact: normally derived from the selected project publication binding, with an optional exact caller guard. E3-C never selects a floating current Reference Pack.

`CurrentPublished` is an outer request selector, not a semantic identity. The service records the exact result once and never refreshes it.

## Cross-store current semantics

E3-C does not claim a distributed atomic snapshot across independent user-project, Blizzard UI, and Reference stores. It resolves selectors once in a defined order, acquires immutable retained views, then validates their exact compatibility.

If current records change or selected profiles are incompatible, the operation fails or returns the exact status permitted by its contract. There is no hidden retry, silent rebase, or generation substitution. A caller retry is a new request.

## Exact roots

E3-C accepts only exact E3-B root selectors or the exact acquired project root for `context_map`. It does not perform search, fuzzy matching, path guessing, or natural-language resolution.

## Operation statuses

```text
complete
partial
truncated
not_evaluated
failed
cancelled
```

Precedence for one public result:

```text
failed
cancelled
not_evaluated when no useful semantic artifact exists
truncated when a useful artifact stopped at an explicit hard bound
partial when useful output has other incomplete requested scopes
complete
```

Orthogonal coverage, conflict, omission, continuation, and budget records remain visible.

## Artifact ownership

`wow-context` owns Project Maps, L0/L1 skeletons, semantic packs, renderers, continuations, omissions, and context validation. Service validates and envelopes these artifacts but never edits or reconstructs them.

`wow-project`, `wow-graph`, and `wow-reference` own the exact views and records consumed by context. `wow-store` owns physical snapshots/leases/retention. Service owns acquisition order and orchestration only.

## Application boundary

[`../../../apps/wow/e3/README.md`](../../../apps/wow/e3/README.md) defines thin CLI commands. The application parses bounded transport input, calls these service operations, serializes the result/artifact, maps exit codes, and handles signals. It cannot access lower crates.

## Required reading

1. [`AGENTS.md`](AGENTS.md)
2. [`DECISIONS.md`](DECISIONS.md)
3. [`DATA_MODEL.md`](DATA_MODEL.md)
4. [`CURRENT_RESOLUTION_AND_ACQUISITION.md`](CURRENT_RESOLUTION_AND_ACQUISITION.md)
5. [`LEASES_CANCELLATION_AND_CLOSURE.md`](LEASES_CANCELLATION_AND_CLOSURE.md)
6. [`CONTEXT_OPERATIONS.md`](CONTEXT_OPERATIONS.md)
7. [`ROOT_PROFILE_AND_RENDER_POLICY.md`](ROOT_PROFILE_AND_RENDER_POLICY.md)
8. [`RESULT_ENVELOPE_AND_STATUS.md`](RESULT_ENVELOPE_AND_STATUS.md)
9. [`APPLICATION_BOUNDARY.md`](APPLICATION_BOUNDARY.md)
10. [`SECURITY_AND_PRIVACY.md`](SECURITY_AND_PRIVACY.md)
11. [`ERROR_MODEL.md`](ERROR_MODEL.md)
12. [`TEST_MATRIX.md`](TEST_MATRIX.md)
13. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
14. [`CONTRACT.json`](CONTRACT.json) and [`examples/`](examples/README.md)
15. [`../../wow-context/e3/README.md`](../../wow-context/e3/README.md)
16. [`../../wow-project/e3/README.md`](../../wow-project/e3/README.md)
17. [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md)

## Deferred

- search/fuzzy/natural-language root resolution;
- lineage/migration/patch impact;
- rule execution or finding generation in context operations;
- model/embedding/Codebase Memory lanes;
- physical context cache;
- LSP/MCP/HTTP/daemon transports;
- source/project edits, tool authorization, runtime probes, and releases;
- CI.

## Completion gate

E3-C implementation is complete only when current selectors resolve once, exact selectors never substitute, all acquired views form one validated E3-B universe set, continuation uses exact retained generations, every success/failure/cancellation closes resources, no lower-domain algorithm appears in service or app, all operation statuses/envelopes/exit mappings are deterministic, and all fixture/checksum/security/cancellation/race tests pass.
