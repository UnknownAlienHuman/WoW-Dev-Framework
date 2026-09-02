# Architecture

**Status:** normative.

**Current contract baseline:** E0-A through E7-A, reviewed 2026-09-02.

Detailed package contracts and [`../crates/MANIFEST.json`](../crates/MANIFEST.json) are authoritative for active operations, dependencies, states and implementation gates. This document defines the system-wide shape.

## 1. Responsibility layers

```text
Foundations
    wow-core
    wow-store

Exact local/reference intelligence
    wow-reference
    wow-annotations
    wow-emmy
    wow-project
    wow-graph
    wow-recognizers
    wow-rules
    wow-search
    wow-context

Governed recognizer evolution
    wow-recognizers E5-A
    wow-service E5-B/E5-C

Optional external discovery
    wow-cbm E6-A
    wow-service E6-B mapping/selection/context orchestration

Use cases and frontend sessions
    wow-service through E7-A
    apps/wow one-shot CLI, local daemon, LSP and MCP

Release/distribution
    E7-B next
```

Crate directories are implementation contracts, not active Rust crates. Implementation remains E0-first.

## 2. Evidence universes

These remain distinct:

```text
Reference Pack
first-party addon project
Blizzard UI source project
calibration corpus and holdout
core recognizer publication
external semantic provider
unsaved document overlay
runtime observation
historical generation
```

Links cross universes only through an owner contract and never transfer authority. Community code can demonstrate an implementation but not define Blizzard API truth. An external locator can map to an owner record while its provider interpretation remains Candidate. An overlay can replace exact source bytes for one session without becoming a saved project generation. Static evidence is not runtime proof.

Every result carries exact identities, provenance, confidence, coverage, conflicts, omissions and nonclaims.

## 3. Dependency direction

Maximum edges are in [`../crates/DEPENDENCY_GRAPH.md`](../crates/DEPENDENCY_GRAPH.md).

```text
wow-core -> no framework dependency
wow-store -> wow-core only
owner crates -> never service/application/transports
wow-cbm -> wow-core only
wow-context -> no wow-cbm/service dependency
apps/transports -> wow-service only
```

`wow-service` coordinates narrow public owner ports. It cannot reproduce parser, analyzer, project, graph, recognizer, diagnostics, search, context, storage, mapping, signing or provider-normalization algorithms.

Owner-neutral bounded projections cross seams when a direct dependency would violate ownership. E6 locator mapping and E7 workspace/overlay inputs follow this rule.

## 4. Reference domain

A Reference Pack binds exact flavor/Interface/build, Blizzard source snapshot/digest, generated documentation inputs, builder/evaluator/schema versions, corrections, raw unknown fields, coverage/negative-authority partitions and license/provenance state.

```text
pinned source
-> restricted APIDocumentation evaluation
-> raw metadata retention
-> schema-aware normalization
-> structural source extraction
-> reviewed corrections and differential reports
-> immutable pack candidate
-> publication + fresh read-back
-> exact retained ReferenceView
```

Arbitrary Lua is never executed. Unknown/unsupported fields narrow capability. Annotations are projections, not canonical restriction/source/coverage storage.

## 5. Project and overlay domain

A saved project generation binds exact source/config/profile, ReferenceView, TOC/load variants, XML, Lua units, analyzer, recognizer/rule/graph profiles and publication state.

```text
bounded nonexecuting source acquisition
-> TOC/XML/load interpretation
-> Emmy VFS/analysis
-> normalized facts
-> recognizer/graph proposals
-> graph validation/publication
-> diagnostics
-> immutable ProjectGeneration
```

E7 adds explicit session-scoped overlays:

```text
explicit workspace registration
+ exact saved base or explicit no-base
+ document URI/version/full content
-> immutable FrontendDocumentOverlay
-> exact overlay analyzer input
-> overlay-aware owner results
```

Each change produces a new overlay snapshot. Stale/out-of-order versions require resynchronization. Unsaved bytes are private and memory-only by default. Saved and overlay evidence remain distinguishable.

## 6. EmmyLua and diagnostics

EmmyLua is the sole Lua parser/analyzer on the correctness path and is pinned behind one compatibility-probed adapter.

Diagnostics combine generic Emmy findings, exact ReferenceView, exact saved/overlay project/load/graph facts and restriction capability state. Missing capability yields `NotEvaluated`. Root causes/downstream symptoms remain distinct. Exact edits require document/source/content/range guards; otherwise output is a plan/disabled action.

`wow-emmy` emits canonical UTF-8 byte ranges. E7 transport position conversion occurs against the exact overlay line index; analyzer owners do not speak LSP/MCP.

## 7. Graph and recognizers

Graph axes remain independent: lexical, ownership, load, object, inheritance, registration, lifecycle, state, call and lineage. Every assertion binds producer/evidence/confidence/generation/coverage/conflicts/partition.

Recognizers consume normalized owner facts. Production rules never branch on repository/addon/owner/path/popularity/labels/splits/reviewers/holdout/canary/model/client identities.

Pack changes create new project/graph generations. Historical generations remain immutable.

## 8. Search, lineage and context

Search prioritizes exact identities and explicit aliases/transitions/replacements before structured shape, fuzzy/text, graph and optional external candidates. Similarity only retrieves Candidates.

Lineage/migration/static impact use exact before/after generations and typed evidence. Name/path/fingerprint/rank alone is insufficient.

Context uses exact roots and retained views:

```text
ContextUniverseSet
-> ProjectMap
-> L0
-> L1
-> ContextSemanticPack
-> rendered artifacts
```

L2 source requires explicit privacy/license/budget policy. Rendering/source/provider/client text is not semantic truth or instruction.

## 9. Restriction and runtime model

Restriction facets are open/versioned. Unknown facets remain raw and block dependent checks. Static analysis covers only demonstrated capabilities. State/data/hotfix/combat/taint/Secret-dependent behavior requires exact client-build runtime evidence when runtime proof is needed.

Patch-sensitive guidance routes through the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb), pinned Blizzard source and required runtime probes.

## 10. E5 governed recognizer lifecycle

```text
E5-A admitted corpus, shadow pack, mutations, metrics, candidate
E5-B durable runs, independent review, sealed holdout, PromotionSubmission
E5-C independent revalidation, CorePackArtifact, attestations/signing,
     PublishedInactive, read-back, canary, rollout, activation, LKG,
     rollback/revocation/deactivation and partition closure
```

Every gate remains independent. Activation is profile-specific CAS. LKG is explicit. Rollback creates new immutable records. E5-C internal publication is not public release distribution.

## 11. E6 optional external discovery

E6-A accepts a reviewed descriptor/capability set, explicit stable/mutable/opaque external state and closed bounded query through an allow-listed transport. Every result is `semantic_candidate + Candidate`, has no negative authority, and stores provider paths/URIs/symbols/spans as `UnverifiedProviderLocator`.

E6-B acquires a secret-isolated provider session, registers durable query identity, publishes results, asks exact project/reference owners to map locators, records explicit selection, and passes one exact mapped root to normal context. Provider metadata remains a separate sidecar. Mapping/selection/context never verifies provider interpretation. Failure is lane-local and has no hidden fallback.

## 12. Storage

`wow-store` owns generic physical persistence only: schemas/migrations, SQLite/WAL profiles, immutable objects, append-only catalog/effect/audit records, read-back protocols, leases/retention/GC, backup/restore and corruption handling.

For E7 it may store registry generations, bounded durable session metadata, operation tickets, response delivery journals and leases. Unsaved overlay bodies remain memory-only by default. Store never interprets protocol, editor, provider, mapping, diagnostic or context semantics.

Raw SQL, connections, physical keys, transaction callbacks and filesystem roots never cross public service/application seams.

## 13. Service and operation registry

`wow-service` owns strict request validation, one-time selector resolution, exact owner acquisition, compatibility validation, narrow port sequencing, durable effect/reconciliation state, conservative envelopes, retention/audit and close-before-success.

E7 adds an immutable `FrontendOperationRegistry`. Each entry binds one service operation, exact request/result/error schemas, owner capabilities, effect/authorization class, privacy/license, budgets, cancellation/progress and frontend names. Runtime negotiation can narrow only. Missing implementations are not advertised. There is no reflection or generic `call_service`.

## 14. Frontend sessions and protocols

A `FrontendSession` binds protocol profile, client/consumer identity, registry generation, explicit workspaces, overlays, operation tickets, response journal, leases and close state. Client identity is compatibility/provenance input, not authorization.

Initial profiles:

```text
cli-one-shot-v1
wow-local-jsonrpc/1 over current-user local IPC
LSP 3.18 over stdio
MCP 2025-11-25 over stdio
MCP 2025-11-25 Streamable HTTP on loopback only, explicit/disabled by default
```

LSP projects implemented exact diagnostics/hover/definition/references/symbol/completion/signature-help/guarded-code-action/call-hierarchy operations. It does not introduce editor-specific semantic forks or automatic edits.

MCP exposes fixed implemented read-only tools and exact resources. It initially omits prompts, sampling, elicitation, tasks, server-requested roots and effecting tools. A model invocation is never user authorization.

The local daemon is foreground/current-user IPC. No default TCP or remote service.

## 15. Progress, cancellation, reconnect and delivery

Wire request IDs and durable operation IDs are separate. Disconnect is not cancellation. Progress is bounded/non-authoritative and can be coalesced. Final results/errors/state changes outrank progress/logs.

Delivery state is separate from service completion. A retained final response can be replayed to an authorized session without reexecution. Timeout/disconnect after possible effect can yield `OutcomeUnknown`; blind retry is forbidden.

Sessions isolate workspaces, overlays, authorization, private source, provider access, operations, results and journals. No background work remains after required close unless a future explicitly owned durable-job contract exists.

## 16. Security and privacy

Public seams structurally reject/contain:

```text
arbitrary Lua/source/repository execution
raw SQL/database/provider handles
generic MCP/tool/RPC/shell/script/plugin/model execution
private signing/provider/deployment/session material
remote listeners by default
unbounded source/graph/protocol/result streams
source/provider/model/client text as control instructions
cross-client source/overlay/result access
```

Output is the intersection of source/provider/consumer/privacy/license/notice/redistribution policies. Unknown narrows/denies. Logs/errors use stable IDs/counts/stages/reason codes and redact source/secrets/private paths/handles.

## 17. Launch and release layering

[`LAUNCH_GATES.md`](LAUNCH_GATES.md) separates:

```text
first runnable: E0-A through E0-F
useful internal alpha: E1 through E3
developer preview: E4 + one complete minimal E7-A frontend
governed beta: E5; E6 optional
release candidate/public v1: implemented E7-A + E7-B release gates
```

E7-B next owns reproducible builds, exact packages/manifests/checksums/signatures/SBOM/provenance, install/update/rollback/retirement, compatibility/support/privacy/incident policy and public distribution. No CI/release workflow exists before real executable commands and an explicit release owner.