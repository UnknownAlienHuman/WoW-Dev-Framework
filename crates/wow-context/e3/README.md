# `wow-context` E3-A Project Map and skeleton contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-context/e3-a/project-map-skeleton-progressive-context`

## Mission

E3-A converts one exact published project/reference/graph state into deterministic, evidence-bearing, budgeted context artifacts for humans and agents without turning summaries into authority or bulk-loading the repository.

```text
exact PublicationSet / ProjectSnapshot / GraphSnapshot
+ exact ReferenceView where platform facts are requested
+ versioned context, skeleton, source, budget, tokenizer, and security profiles
+ exact roots and relation lanes
-> validate generation and capability closure
-> build one Project Map
-> build L0 semantic skeletons
-> build selected L1 structural skeletons
-> expand only requested graph/project branches
-> include exact evidence/source handles and bounded faithful excerpts when allowed
-> record omitted, conflicted, partial, unsupported, and truncated scope
-> stop at no-new-evidence, requested completeness, or explicit budgets
-> produce one deterministic ContextBundle with continuation frontier
```

Context is a projection over authoritative project, graph, reference, and evidence records. A convenient summary, role label, skeleton line, or model-facing bundle never becomes source truth by itself.

## Direct dependencies in E3-A

```text
wow-core
wow-reference
wow-project
wow-graph
```

`wow-emmy` facts are consumed through the exact published `ProjectView`/project logical bundles; E3-A does not import raw analyzer internals. `wow-search` remains inactive until E4. Exact roots are supplied directly or resolved by an owning higher layer.

## Owned responsibilities

- context/skeleton/detail/budget/tokenizer/source/security/evaluation profiles;
- exact context input and generation validation;
- deterministic project-wide Project Map;
- L0 entity/module/file/package/API-role skeletons;
- L1 selected signatures, members, direct relations, lifecycle/signal/state summaries, and source-backed skeleton spans;
- relation-lane selection over exact bounded graph queries;
- progressive expansion and continuation frontier;
- no-new-evidence, cycle, depth, coverage, conflict, and budget stopping;
- byte/character/line/node/edge/source/evidence and optional pinned-tokenizer accounting;
- context coverage, projection loss, omission, and truncation records;
- exact source/evidence/provenance handles and bounded faithful source excerpts;
- deterministic ordering, IDs, manifests, and bundle serialization;
- context utility/relevance/redundancy/evidence-closure evaluation fixtures;
- security, privacy, cancellation, and typed errors.

## Explicit non-responsibilities

E3-A does not:

- parse Lua, TOC, XML, SQLite, or arbitrary source;
- run analyzers or recognizers;
- publish project/graph/store generations;
- perform fuzzy, FTS, semantic, migration, or lineage search;
- infer replacements, diagnostics, severity, fixes, taint, combat, Secret, or runtime behavior;
- generate implementation code or paraphrase missing source as if it existed;
- use an LLM/model in the correctness path;
- execute source, repository scripts, plugins, prompts, or tools embedded in comments;
- read SavedVariables, logs, client memory, editor state, or installed addons;
- mutate source, editor, graph, project, store, or context inputs;
- upload context, call networks, or start MCP/LSP;
- add CI.

## Detail levels

### Project Map

Repository/package/load/ownership/role overview with exact entry points, direct structural lanes, capabilities, conflicts, and coverage.

### L0 skeleton

Compact stable identity and public structural surface:

```text
entity kind and exact semantic key
project/reference/graph generations
package/module/file/owner and load position
universal roles and direct important relations
public callable/member/event/state/API-use headings
source/evidence handles
coverage/conflict/partial state
```

No function bodies and no inferred prose semantics.

### L1 skeleton

Requested bounded structural detail:

```text
exact signatures and member positions
selected direct calls/uses/events/callbacks/hooks/state reads-writes
lifecycle/load/ownership chains
selected source-backed declaration/control-flow skeleton nodes
related evidence, conflicts, and reason paths
```

L1 still does not reproduce full source by default. Exact bounded excerpts are a separate explicit source-detail lane.

### Later/deferred detail

Bulk/full source, fuzzy discovery, semantic retrieval, cross-build lineage, and model-synthesized explanations remain outside E3-A.

## Context request

```text
ContextRequest
    exact PublicationSetId / ProjectSnapshotId / GraphSnapshotId
    optional exact ReferenceView/Profile/ReferenceGeneration
    root entity IDs[]
    requested artifact kind = ProjectMap | L0 | L1 | Bundle
    relation lane and direction policy
    source/evidence inclusion policy
    detail profile
    coverage/conflict/confidence policy
    byte/node/edge/source/token budgets
    continuation cursor: optional
    cancellation
```

No implicit current/latest profile or repository scan.

## Public operations

```text
validate_context_profiles
validate_context_request
build_project_map
build_l0_skeletons
build_l1_skeletons
plan_context_expansion
expand_context_frontier
build_context_source_excerpts
build_context_coverage_and_loss
build_context_bundle
continue_context_bundle
measure_context_bundle
compare_context_bundles
validate_context_bundle
```

## Project Map contents

```text
exact project/publication/profile identities
packages, selected TOC variants, load units, files, XML/virtual Lua units
ownership/module/service/library/state-role views
entry points and direct lifecycle/signal/hook/state/API-use lanes
important graph roots and bounded direct neighborhoods
capability/coverage/conflict/truncation status
source/evidence handles and next-detail routes
```

Named addon/framework calibration cannot change production semantics. Pinned real projects are evaluation fixtures only.

## Progressive context discipline

Normal flow:

```text
Project Map
-> exact L0 root skeleton
-> selected L1 lane or neighbor
-> exact evidence/reason path
-> bounded source excerpt only when needed
-> stop at no-new-evidence or request/budget completion
```

E3-A does not dump all files, graph nodes, evidence, or source into one prompt.

## Budget accounting

Canonical budget axes include:

```text
entities and skeleton records
relations and reason-path edges
source/evidence handles
source excerpt bytes/lines
UTF-8 output bytes and Unicode scalar count
structured nodes/fields
optional exact token count under one pinned tokenizer profile
```

A generic guessed “tokens” estimate is not authoritative. Token counts are exact only for an explicit tokenizer/version/config; otherwise report byte/character structural measures and an optional clearly labeled estimate profile.

## Evidence and coverage

Every material context claim links exact project/graph/reference/evidence records or an explicit deterministic derivation. Context coverage is separate from source/graph/reference coverage.

A compact bundle cannot hide:

```text
partial source facts
recognizer ambiguity
graph conflicts
NotEvaluated capabilities
candidate/possible confidence
budget omissions or truncation
unsupported detail
```

## Security

Source/comments/documentation may contain prompts or instructions. Context artifacts quote them as untrusted source data and never elevate them to repository policy. Private paths, credentials, full source bodies, SavedVariables, runtime-sensitive payloads, and unrestricted objects are excluded.

## Required reading

1. repository and `crates/` instructions;
2. [`../../../docs/GRAPH_SEARCH_AND_PLANNING.md`](../../../docs/GRAPH_SEARCH_AND_PLANNING.md);
3. [`../../../docs/PROVENANCE_AND_COVERAGE.md`](../../../docs/PROVENANCE_AND_COVERAGE.md);
4. [`../../../docs/AGENT_WORKFLOW.md`](../../../docs/AGENT_WORKFLOW.md);
5. [`../../wow-project/e2/README.md`](../../wow-project/e2/README.md);
6. [`../../wow-graph/e2/README.md`](../../wow-graph/e2/README.md);
7. [`../../wow-store/e2/README.md`](../../wow-store/e2/README.md);
8. this entire E3-A package and fixtures;
9. current external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb) routes when patch-sensitive interpretation is needed.

## Completion gate

E3-A implementation is complete only when the same exact published state and profiles produce byte-identical Project Map, L0/L1 skeletons, context bundles, loss records, metrics, and continuation results under 1/2/N workers and shuffled input order; every material claim has evidence/derivation closure; source/graph/reference partial state remains visible; bounded expansion stops deterministically; old/new publication sets never mix; no source text becomes instructions; exact-token claims require a pinned tokenizer; pinned synthetic and real-project evaluation corpora show useful compression without missing mandatory structural/evidence records; and all applicable fixtures/checksums pass.
