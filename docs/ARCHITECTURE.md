# Architecture

**Status:** normative.

**Current contract baseline:** E0-A through E6-B, reviewed 2026-09-02.

The detailed package contracts and [`../crates/MANIFEST.json`](../crates/MANIFEST.json) are authoritative for active operations, dependencies, states, and implementation gates. This document defines the system-wide shape and invariants.

## 1. System shape

WoW Dev Framework has five separated responsibility layers:

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
    wow-service E6-B owner mapping/selection/context orchestration

Use cases and transports
    wow-service
    apps/wow and future E7 thin CLI-daemon/LSP/MCP hosts
```

Crate directories document future libraries; they are not active crates until their implementation freeze gates pass. The implementation frontier remains E0 even though documentation extends through E6-B.

## 2. Evidence universes

The following universes never merge implicitly:

```text
Reference Pack
first-party addon project
Blizzard UI source project
calibration corpus and holdout
core recognizer publication
external semantic candidate provider
runtime observation
historical generation
```

A record may link exact identities across universes only through an owning contract. Linkage does not transfer authority.

Examples:

- a community addon can demonstrate an implementation pattern but not define the Blizzard API contract;
- a provider locator can map to a project source record but its summary/trace remains external Candidate evidence;
- a static source fact does not prove runtime state;
- a calibration metric or review does not activate a core pack;
- a signature proves exact bytes/key/profile binding, not semantic or runtime correctness.

Every result carries exact profile/generation identities, provenance, confidence, coverage, conflicts, omissions, and relevant nonclaims.

## 3. Dependency direction

Dependencies point toward narrower foundations. Maximum permitted edges are defined in [`../crates/DEPENDENCY_GRAPH.md`](../crates/DEPENDENCY_GRAPH.md).

Core rules:

```text
wow-core depends on no framework crate
wow-store depends only on wow-core
owner crates never depend on wow-service or applications
wow-cbm depends only on wow-core
wow-context does not depend on wow-cbm
applications/transports depend on wow-service only
```

`wow-service` coordinates narrow owner ports. It cannot reproduce parser, analyzer, project, graph, recognizer, search, context, storage, mapping, signing, or provider-normalization algorithms.

An owner-neutral bounded projection may cross a seam when a direct dependency would violate ownership. E6-B locator mapping uses this pattern: project/reference owners consume typed locator fields without depending on `wow-cbm`.

## 4. Reference domain

A Reference Pack is immutable and profile-specific. One build binds:

```text
WoW flavor and Interface/build identity
pinned Blizzard UI/source snapshot and digest
APIDocumentation and generated implementation inputs
builder/evaluator/schema versions
reviewed correction set
raw unknown-field preservation
coverage and negative-authority partitions
license/notice/provenance state
```

Pipeline:

```text
pinned Blizzard source snapshot
-> restricted declarative APIDocumentation evaluation
-> raw metadata retention
-> schema-aware normalization
-> structural TOC/XML/Lua extraction through approved owners
-> digest-bound reviewed corrections
-> differential oracle reports
-> immutable Reference Pack candidate
-> publication and fresh read-back validation
-> exact retained ReferenceView
```

Arbitrary Lua is never executed. Unsupported constructs and unknown fields reduce the affected capability/coverage and remain visible; they are not dropped or assumed safe.

Annotations are consumer projections, not the canonical store for Secret/restriction metadata, raw unknown fields, source spans, correction provenance, or coverage.

## 5. Project domain

One project generation binds exact source/configuration/profile inputs:

```text
configured project roots and source snapshot
selected ReferenceView
TOC variants and load order
XML templates/mixins
Lua physical and virtual units
EmmyLua implementation/profile
recognizer/rule/graph producer profiles
publication and retention state
```

Pipeline:

```text
bounded nonexecuting source acquisition
-> TOC/XML/load interpretation
-> Emmy VFS and semantic analysis
-> normalized syntax/semantic facts
-> declarative recognizer proposals
-> graph partition validation/publication
-> capability-declared diagnostics
-> immutable ProjectGeneration publication
```

Analyzed repositories are untrusted input. The framework does not execute hooks, installers, build scripts, repository tools, or Lua merely to index a project.

Mutable working state stays inside its owner. Readers consume immutable generations and never mix `current` data resolved at different times. A symbolic current selector is resolved once by service, replaced with an exact identity, retained, and never refreshed inside the operation.

## 6. EmmyLua and diagnostics

EmmyLua is the sole Lua parser/analyzer on the correctness path. It is pinned behind one adapter and compatibility-probed before activation.

Diagnostics combine:

```text
upstream generic Emmy findings
framework WoW diagnostic providers
exact ReferenceView facts
exact project/load/graph facts
restriction capability state
```

A rule declares its required capabilities. Missing capability yields `NotEvaluated` or typed unavailability, never clean success. Root causes and downstream symptoms remain distinct. Autofixes require exact mechanically checkable preconditions; otherwise output is a plan or Candidate.

No user/editor configuration is silently overwritten. Generated configuration/artifacts use explicitly owned project paths and profiles.

## 7. Graph and recognizers

The graph preserves independent relation axes:

```text
lexical containment
package/module/namespace ownership
TOC/load dependency and order
XML/frame object parentage
inheritance/mixin composition
event/callback/hook/style/element/plugin registration
lifecycle ownership
state-root/state-path access
call evidence
cross-generation lineage/change
```

There is no universal `parent` relation. Every assertion binds producer, evidence, confidence, profile/generation, coverage, conflicts, and partition ownership.

Recognizers operate only on normalized owner facts. Production rules are universal structural patterns and cannot branch on repository, addon, owner, path, popularity, labels, split membership, reviewer, holdout, canary, or model identity.

Producer partitions are immutable per generation. Activating, rolling back, revoking, or replacing a recognizer pack creates new project/graph generations; historical generations are never rewritten.

## 8. Search, lineage, and context

Search authority order is explicit:

```text
exact canonical identity
explicit aliases/deprecations/replacements/transitions
namespace/member/prefix
receiver/signature/type/restriction shape
bounded fuzzy/text lanes
FTS5 over exact-generation documents and skeletons
graph-neighborhood candidates
optional external semantic candidates
```

Similarity retrieves Candidates; it cannot establish lineage, replacement, negative authority, migration safety, impact, or edit authorization. Ranking explanations expose lanes, exact inputs, scores, coverage, and proof ceilings.

Lineage and static impact operate over exact before/after generations and typed evidence. Path/name/fingerprint/search rank alone is insufficient.

Context uses exact roots and exact retained project/reference/graph views:

```text
ContextUniverseSet
-> ProjectMap
-> L0 skeleton
-> L1 skeleton
-> ContextSemanticPack
-> rendered artifacts
```

L2/full source is requested explicitly under privacy/license/budget policy. Context rendering is not semantic truth. Source/provider text is data, never framework or agent instruction.

## 9. Restriction and runtime model

Restriction facets are open and versioned. Unknown facets remain raw and make dependent checks `NotEvaluated`.

Static Secret/restriction analysis progresses by demonstrated capability:

```text
API contract facets
direct local operations and guard dominance
bounded interprocedural summaries for stable direct calls
structured runtime evidence
```

Static analysis never freezes a permanent runtime spell/client-state whitelist. State-, hotfix-, data-, combat-, taint-, or restriction-dependent behavior requires an exact client-build scenario and runtime evidence when the task needs runtime proof.

Patch-sensitive guidance is routed through the current external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb), then confirmed against pinned Blizzard source and required runtime probes. The KB is linked, not copied into stable contracts.

## 10. Governed recognizer lifecycle (E5)

E5 separates authority gates:

```text
E5-A admitted calibration corpus, shadow pack, mutations, metrics, candidate
E5-B durable runs, independent authorization/review, sealed holdout, PromotionSubmission
E5-C independent revalidation, distinct CorePackArtifact, attestations/signing,
     PublishedInactive, read-back, canary, finite rollout, activation, LKG,
     rollback/revocation/deactivation and stale-partition closure
```

No metric, graph validation, review, holdout result, submission, signature, inactive publication, canary, rollout stage, or repository identity automatically authorizes the next gate.

Activation is execution-profile-specific and guarded by exact expected-current compare-and-swap. Last-known-good is explicitly designated, not inferred as previous/newest. Rollback creates new immutable effect/reindex/closure records and never rewrites history.

Public package distribution remains E7-B; E5-C internal publication is not a downloadable release.

## 11. Optional external discovery (E6)

### E6-A owner boundary

`wow-cbm` consumes:

```text
reviewed provider descriptor
negotiated capability intersection
StableExternalGeneration | ObservedMutableGeneration | OpaqueExternalState
closed bounded query
already-acquired allow-listed transport
```

It returns immutable loss-preserving external candidate records. Every result is:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider labels, top/sole/repeated results, stable state, and high scores cannot raise authority. Scores/ranks remain provider-local. Zero results do not prove absence.

Provider path/URI/revision/symbol/span/digest fields remain `UnverifiedProviderLocator`. E6-A never opens or maps them.

### E6-B orchestration boundary

`wow-service` may:

```text
resolve one exact provider configuration
obtain nonsecret credential-use authorization
acquire one narrow host-owned session
register durable operation identity
invoke E6-A
publish/read back immutable result artifacts
ask project/reference owners to map one exact locator
record caller-supplied Selected | Rejected | Deferred
invoke normal context with one exact mapped root
compose a separate ExternalCandidateSidecar
```

Only project/reference owners map locators. Mapping statuses preserve exact mapped, multiple, authoritative no-mapping, partial no-mapping, conflict, `NotEvaluated`, and failure. `ExactMapped` proves locator-to-owner-record identity only.

Selection is explicit and is not verification, semantic acceptance, lineage, replacement, impact, edit authorization, or core-pack admission. Provider metadata never enters `ContextSemanticPack` truth.

Provider failure is lane-local. There is no hidden fallback to another provider, stale cache, model, web, local search, or broader query.

Provider installation, startup, configuration mutation, index/import/delete lifecycle, database access, arbitrary MCP/tools, and secret material are outside E6 semantic operations.

## 12. Storage

`wow-store` owns generic physical persistence only:

```text
explicit schemas and migrations
SQLite profiles and WAL ownership
immutable content-addressed objects
append-only catalog/effect/audit records
published-inactive then fresh read-back validation protocols
exact snapshot/lease/retention/GC
backup/restore and corruption handling
```

Domain owners supply logical schema, canonical bytes, prepared operations, validation rules, and retention edges. Store never interprets project, graph, recognizer, provider, mapping, selection, or context semantics.

Raw SQL, database connections, row IDs, physical object keys, transaction callbacks, and filesystem roots never cross public service/application seams.

No graph server, vector database, or separate search daemon is introduced without measured unique necessity.

## 13. Service and applications

`wow-service` is the only production coordination layer. It owns:

```text
strict request validation
one-time symbolic selector resolution
exact retained owner acquisition
cross-owner compatibility validation
narrow port sequencing
durable effect/idempotency/reconciliation state
conservative result envelopes
retention/audit and reverse-order close-before-success
```

Applications and E7 transports depend on `wow-service` only. One valid command/tool/request maps to one service operation unless a higher-level workflow is itself a documented service operation.

CLI/LSP/MCP/daemon layers cannot resolve authority, inspect lower stores, execute source, expose generic tool/shell/RPC escape hatches, or change semantic output.

## 14. Durable effects and failure behavior

Effecting operations register `OperationId + CanonicalRequestDigest` before dispatch. Same ID/same digest returns or reconciles the same effect; same ID/different digest fails.

A timeout, cancellation, disconnect, serialization failure, or process loss after dispatch can yield `OutcomeUnknown`. Response loss does not prove effect absence. Blind redispatch is forbidden until exact owner reconciliation.

Public success is not finalized before mandatory retention, audit, and reverse resource closure. Cancellation stops new work but does not erase durable evidence or spawn unowned background cleanup.

Other failure rules:

- stale/missing profile: reject or explicit unavailable state;
- failed partition: unaffected partitions remain usable and dependent capabilities become incomplete;
- conflicting evidence: retain conflict, never choose newest/majority by default;
- unknown upstream field: preserve and narrow capability;
- malformed/untrusted input: bounded typed failure;
- provider unavailable: external lane unavailable, exact local lane unchanged;
- runtime behavior not proved: `NotEvaluated`/`Possible` plus required scenario.

## 15. Security and privacy

Public seams reject or structurally isolate:

```text
arbitrary Lua/source execution
repository hooks/installers/build scripts
raw SQL and database handles
generic MCP/tool/RPC calls
shell/script/plugin/model-prompt escape hatches
private signing/provider/deployment material
private endpoints/process/client handles
unbounded source, graph, response, or continuation data
source/provider text as control instructions
```

Output is the intersection of source, provider, consumer, privacy, license, notice, and redistribution policies. Unknown state denies or narrows; higher layers cannot widen lower restrictions.

Logs/errors default to stable IDs, counts, stages, statuses, and reason codes rather than source, secrets, private paths, private cohort/provider data, or raw owner handles.

## 16. Launch and release layering

Launch states are defined in [`LAUNCH_GATES.md`](LAUNCH_GATES.md):

```text
first runnable bootstrap: E0-A through E0-F
useful internal alpha: E1 through E3
external developer preview: E4 plus minimal E7-A frontend
governed beta: E5; E6 optional when enabled
public supported v1: selected beta scope plus E7-A/E7-B gates
```

E7-A owns supported CLI-daemon/LSP/MCP session and transport contracts. E7-B owns reproducible packaging, checksums/signatures/SBOM/provenance, installer/update channels, compatibility/support policy, rollback/retirement, incident response, and public distribution.

No CI or release workflow is added before it executes real frozen commands with an explicit owner and corresponding launch gate.