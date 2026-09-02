# Architecture

**Status:** normative.

**Planned contract baseline:** E0-A through E7-B, reviewed 2026-09-02.

Detailed package contracts and [`../crates/MANIFEST.json`](../crates/MANIFEST.json) are authoritative for operations, dependencies, implementation states, and freeze gates. This document defines the final planned system shape. Rust implementation has not started.

## 1. Responsibility layers

```text
Foundations
    wow-core
    wow-store

Exact local and reference intelligence
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
    wow-service E5-B and E5-C

Optional external discovery
    wow-cbm E6-A
    wow-service E6-B mapping, selection, and context orchestration

Use cases and product transports
    wow-service
    apps/wow
    apps/wow-reference-builder
    tools/wow-release

Release and installation owners
    E7-B source, build, evidence, signing, distribution,
    installation, migration, update, support, and incident adapters
    behind narrow wow-service ports
```

One public product executable is planned:

```text
wow
    one-shot CLI
    current-user local daemon
    LSP 3.18
    MCP 2025-11-25
    local release verification and explicit update/rollback client
```

`wow-reference-builder` and `wow-release` are internal or administrative service-only clients and are excluded from the default public bundle unless an exact distribution profile includes them.

A documented directory is not an active crate. Implementation begins with I0-A and activates workspace members only when a real owned slice and its tests exist.

## 2. Evidence universes

These universes never merge implicitly:

```text
Reference Pack
first-party addon project
Blizzard UI source project
calibration corpus and sealed holdout
core recognizer publication
external semantic-candidate provider
session-private document overlay
runtime observation
historical generation
release source and build evidence
release bundle and distribution channel
local installation and update state
```

A typed link does not transfer authority.

- Community code is implementation evidence, not Blizzard API authority.
- An external locator can map to an owner record while provider interpretation remains Candidate evidence.
- A document overlay is private session source state, not a saved or published project generation.
- Calibration metrics, review, holdout, signature, publication, canary, and activation are independent.
- A release signature authenticates exact bytes and key scope; it does not prove source correctness or runtime safety.
- A published bundle is not an installed or supported product.
- A successful installation self-check is not proof for every project, client, WoW profile, or runtime scenario.

Every result preserves exact profile, generation, source, effect, release, and installation identities as applicable, plus provenance, confidence, coverage, conflicts, omissions, authorization, and required nonclaims.

## 3. Dependency direction

Maximum edges are in [`../crates/DEPENDENCY_GRAPH.md`](../crates/DEPENDENCY_GRAPH.md).

```text
wow-core -> no framework dependency
wow-store -> wow-core only
owner crates -> foundations and reviewed lower owners only
wow-cbm -> wow-core only
wow-service -> reviewed owner crates through narrow public contracts
applications and tools -> wow-service only among framework crates
```

Owners never depend on service, applications, or tools. Store never interprets domain semantics. Service never reproduces parser, analyzer, project, graph, recognizer, diagnostic, search, context, provider normalization, locator mapping, storage, build, signing, distribution, installation, migration, or rollback algorithms.

Owner-neutral bounded projections cross seams only when a direct dependency would violate ownership. E6 locator mapping, E7-A session overlays, and E7-B generic release storage use this pattern.

## 4. Core identity and result model

`wow-core` owns generic invariants only:

```text
stable typed IDs and digests
profiles and exact generations
provenance and confidence
coverage partitions and negative authority
conflicts, omissions, and evidence references
status, result, error, cancellation, and budget primitives
OperationId + CanonicalRequestDigest
canonical ordering and serialization
```

It does not own project, graph, storage, provider, editor, release, or transport semantics.

`Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed, `OutcomeUnknown`, `ResynchronizationRequired`, and `NotEvaluated` are never coerced to proof, clean Negative, complete, or pass. Clean absence requires exact owner scope and complete relevant coverage with no blocker.

## 5. Reference domain

One immutable Reference Pack binds exact:

```text
WoW flavor, Interface, and build
pinned Blizzard source snapshot and digest
APIDocumentation and generated implementation inputs
builder, evaluator, schema, and tool versions
reviewed corrections
raw unknown-field preservation
coverage and negative-authority partitions
license, notice, and provenance
```

Pipeline:

```text
pinned source
-> restricted declarative APIDocumentation evaluation
-> raw metadata retention
-> schema-aware normalization
-> structural TOC, XML, and Lua extraction by approved owners
-> digest-bound corrections
-> differential oracle reports
-> immutable candidate
-> publication and fresh read-back
-> exact retained ReferenceView
```

Arbitrary Lua is not executed. Unknown or unsupported fields narrow capability and remain visible. Annotations are projections, not canonical storage for raw metadata, restrictions, provenance, and coverage.

## 6. Project and analyzer domain

One saved project generation binds exact source, configuration, and profile inputs:

```text
registered source roots and snapshot
ReferenceView
TOC variants and load order
XML templates and mixins
Lua physical and virtual units
Emmy implementation and profile
recognizer, rule, and graph profiles
publication and retention state
```

Pipeline:

```text
bounded nonexecuting source acquisition
-> TOC, XML, and load interpretation
-> Emmy VFS and semantic analysis
-> normalized syntax and semantic facts
-> declarative recognizer proposals
-> graph partition validation and publication
-> capability-declared diagnostics
-> immutable ProjectGeneration
```

EmmyLua is the sole correctness-path Lua parser and analyzer and is pinned behind one compatibility-probed adapter.

Analyzed repositories are untrusted. No hooks, installers, build scripts, repository tools, or Lua execute during indexing. A permitted symbolic current selector is resolved once by service and replaced with an exact retained identity.

## 7. Graph, recognizers, and diagnostics

Independent relation axes include:

```text
lexical
ownership, package, and module
TOC and load dependency or order
XML and frame object
inheritance and mixin
registration, event, callback, hook, style, element, and plugin
lifecycle
state root and state path
call
cross-generation lineage and change
```

There is no universal parent. Every graph assertion binds producer, evidence, confidence, generation, coverage, conflicts, and producer partition.

Production recognizers are universal declarative patterns over normalized owner facts. They cannot branch on repository, addon, owner, path, popularity, label, split, reviewer, holdout, canary, provider, or model identity.

Diagnostics combine generic Emmy results, exact ReferenceView, project and graph facts, and WoW rule providers. Every rule declares required capabilities. Missing capability is `NotEvaluated`; clean output requires complete relevant coverage. Exact mechanical edits require exact guards; otherwise remediation remains a plan, Candidate, or disabled action.

## 8. Search, lineage, and context

Search authority order is explicit:

```text
exact canonical identity
explicit aliases, deprecations, replacements, and transitions
namespace, member, and prefix
receiver, signature, type, and restriction shape
bounded fuzzy and text lanes
FTS5 over exact-generation records
bounded graph-neighborhood candidates
optional external semantic candidates
```

Similarity retrieves Candidates. It cannot establish lineage, replacement, migration safety, impact, absence, or edit authority.

Lineage and impact bind exact before and after generations and preserve ambiguity and proof ceilings.

Context uses exact roots and exact retained views:

```text
ContextUniverseSet
-> ProjectMap
-> L0 skeleton
-> L1 skeleton
-> optional bounded L2 source
-> ContextSemanticPack
-> rendered artifacts
```

Rendering is not semantic truth. Source, provider, client, and model text is data, not instructions.

## 9. Restriction and runtime model

Restriction facets are open and versioned. Unknown facets remain raw and make dependent checks `NotEvaluated`.

Static analysis proceeds only through demonstrated capability:

```text
API contract facets
local operation and guard analysis
bounded interprocedural summaries for stable direct calls
structured runtime evidence
```

Data, state, hotfix, combat, taint, or Secret-dependent behavior requires an exact client-build scenario and runtime evidence when runtime proof is claimed.

Patch-sensitive knowledge routes through the current [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb), then exact pinned Blizzard, Reference, and runtime evidence. Stable contracts link rather than copy mutable facts.

## 10. Storage

`wow-store` owns generic physical persistence:

```text
registered schemas and migrations
SQLite and WAL ownership
immutable content-addressed objects
append-only catalog, effect, and audit records
published-inactive and fresh read-back protocols
snapshot, lease, retention, and GC
backup, restore, corruption, and reconciliation
```

Domain owners supply logical schemas, canonical bytes, prepared operations, validation, and retention edges. Store never interprets project, graph, provider, session, release, installation, or support semantics.

Raw SQL, connection, transaction callback, table or row ID, filesystem root, and physical object key never cross service or application seams.

SQLite remains the baseline local substrate. No vector, graph, search, or daemon database service enters the default path without measured unique necessity and an accepted ADR.

## 11. Governed recognizer lifecycle — E5

```text
E5-A
    admitted corpus, provenance, labels, and splits
    shadow packs, mutations, metrics, and candidate artifacts

E5-B
    durable runs
    independent review authorization
    sealed holdout authorization, audit, and consumption
    PromotionSubmission

E5-C
    independent submission revalidation
    distinct CorePackArtifact
    attestations and signatures
    PublishedInactive and read-back
    exact canary and finite rollout
    profile current CAS and explicit LKG
    rollback, revocation, deactivation, and partition closure
```

Every gate is independent. Historical project and graph generations remain immutable. Public product release and distribution remain E7-B, separate from internal core-pack publication.

## 12. Optional external discovery — E6

### E6-A

`wow-cbm` accepts one reviewed provider descriptor, capability intersection, exact external-state class, and bounded allow-listed query:

```text
StableExternalGeneration
ObservedMutableGeneration
OpaqueExternalState
```

Every result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider labels, rank, score, top, sole, repetition, stable state, and zero result never raise authority. Scores remain provider-local. Provider paths, URIs, revisions, symbols, and spans are `UnverifiedProviderLocator` data and are not opened by E6-A.

### E6-B

Service resolves one exact provider configuration, obtains nonsecret use authorization, acquires a narrow host-owned session, registers durable operation identity, invokes E6-A, and publishes immutable results.

Only `wow-project` or `wow-reference` maps locator fields against one exact owner generation. `ExactMapped` proves locator-to-owner-record identity only. Caller selection is explicit and is not verification. Context receives one exact mapped root; provider fields remain a separate Candidate sidecar.

Provider failure is lane-local. No hidden fallback to another provider, cache, model, web, or local search. No generic MCP or provider database lifecycle and no secret material in E6 public seams.

## 13. Sessions and document overlays — E7-A

Canonical contracts:

```text
wow-service/e7-a/frontend-session-operation-registry
apps/wow/e7-a/frontend-transports
```

The service exposes a closed content-addressed `FrontendOperationRegistry` and exact compatibility profiles. Runtime negotiation only narrows.

A frontend or service session binds exact client, protocol, registry, workspace, project, profile, overlay, authorization, privacy, resource, operation, response-journal, lease, and close state. Session changes publish immutable generations. Semantic requests capture one exact retained session snapshot.

Workspace folders, MCP roots, cwd, document URIs, Git roots, and WoW installations are untrusted registration candidates. They never become project authority by transport convention.

Unsaved buffers are project-owned immutable overlay generations:

```text
exact saved ProjectGeneration or explicit no-base profile
+ exact project-owned file identity
+ session, client, document URI, and document version
+ exact UTF-8 bytes and line-index digest
-> immutable FrontendDocumentOverlay
-> exact analyzer and owner results for that overlay
```

LSP 3.18 uses incremental `textDocument/didChange`. A full-document change is an exact replacement. Stale, skipped, repeated-with-different-content, or out-of-order changes preserve `ResynchronizationRequired`. Position conversion occurs only against the exact negotiated overlay line index.

Save, close, and watched-file notifications do not silently prove disk bytes or publish a new project generation. Unsaved bytes are private and memory-only by default unless an exact retention profile says otherwise.

## 14. Product transports — E7-A

One `wow` executable provides:

```text
one-shot CLI
wow-local-jsonrpc/1 over current-user named pipe or Unix-domain socket
LSP 3.18 over stdio
MCP 2025-11-25 over stdio
optional MCP 2025-11-25 Streamable HTTP on loopback only,
explicit, authenticated, Origin-validated, and disabled by default
```

LSP and MCP are explicit modes and never silently auto-start, connect to, or fall back to the daemon. No default public or remote listener exists.

Each semantic command, method, tool, resource operation, or state-changing notification maps to one exact service operation. There is no reflection, generic `call_service`, arbitrary tool, raw RPC, shell, script, plugin, callback, or model path.

The initial LSP profile covers exact implemented diagnostics, hover, definition, references, symbols, completion, signature help, guarded code actions, and call hierarchy. Exact and Candidate locations remain distinguishable. The framework never automatically applies edits or executes commands.

The initial MCP profile is fixed and read-only. It omits prompts, sampling, elicitation, tasks, server-requested roots, arbitrary tools, provider effects, governance effects, source mutation, and release effects. A model invocation is never user authorization.

Progress is bounded and nonsemantic. Large artifacts use exact digest-bound streams. Backpressure cannot silently drop, reorder, or duplicate semantic requests. Disconnect does not prove cancellation or no effect. Retained responses can be replayed without reexecution. Multi-client workspace, overlay, authorization, source, operation, result, stream, and journal state is isolated.

## 15. Release lifecycle — E7-B

A compiled binary is not a release. The release pipeline is:

```text
exact source tree, Cargo.lock, toolchain, target, features, and dependency closure
-> typed isolated build executor
-> at least two independent unsigned builds
-> reproducibility comparison
-> artifact self-description and complete required tests
-> SBOM, provenance, license, notices, and checksums
-> portable and platform signatures
-> deterministic ReleaseBundle
-> exact ReleaseSupportMatrix and immutable ReleaseCandidate
-> provider-neutral publication and public read-back
-> guarded channel expected-current CAS
-> signed update manifest
-> staged verified installation, migration, current CAS, and self-check
-> explicit LastKnownRunnable installation
-> exact rollback, revocation, retirement, and incident lifecycle
```

The first target intent is Windows x86-64 MSVC. It is not supported until the exact complete Windows build, path, ACL, named-pipe, console, LSP, MCP, code-signing, installation-helper, migration, update, rollback, clean-machine, real-addon, and support suites pass.

Unsigned semantic artifacts define reproducible build identity. Platform signing or notarization follows unsigned digest freeze and maps the exact unsigned artifact to a separate signed artifact.

SBOM, provenance, licenses, notices, checksums, test reports, benchmarks, signatures, bundle, support matrix, release candidate, channel record, and installation record are independent immutable artifacts. Missing required evidence is blocked or `NotEvaluated`.

Reference Packs, core packs, provider adapters, and other data artifacts retain independent identities, signatures, compatibility, and update lifecycles. An offline bundle names exact members and never copies floating current data.

## 16. Distribution and trust — E7-B

Release distribution uses narrow provider-neutral publisher and reader ports. GitHub Releases may implement one adapter, but repository, tag, account, CI job, asset name, upload success, TLS, or provider status is not authorization or artifact trust.

Assets are immutable by digest. A release object cannot be repaired in place by replacing a ZIP under the same identity.

Publication separates:

```text
candidate and channel-plan validation
immutable object materialization or upload
exact public read-back and digest verification
signed release and update manifest publication
expected-current channel CAS
retention, audit, and close
```

Portable and platform signatures use exact domain-separated targets and independent trust-root policy. Private signing, distribution, build, provider, and installation credentials remain inside protected adapters and never enter the repository, public configuration, fixtures, logs, bundles, or canonical results.

## 17. Installation, update, and rollback — E7-B

Updates are explicit by default. The baseline performs no startup or hidden update check, download, install, telemetry, crash upload, or remote configuration.

Keep separate:

```text
update manifest discovered and verified
UpdateAvailable
target bundle materialized or downloaded
bundle verified
installation plan validated
staging prepared
current installation and data backed up or retained
store and configuration migration prepared and applied
executable replacement helper handoff
current installation CAS
post-install self-check
Installed or Updated
LastKnownRunnable designation
cleanup eligibility
rollback planned, applied, and validated
```

The public app never overwrites its running executable or constructs arbitrary helper commands. The Windows baseline uses an exact separately verified installation-owner helper or replacement protocol with opaque plan identity and durable reconciliation.

Store and configuration migrations use registered owner operations with exact source and target schema, backup, crash recovery, read-back, and rollback compatibility. No raw SQL or arbitrary migration script crosses the service or application seam.

LastKnownRunnable is an explicit retained qualified installation record, never inferred from previous, newest, version, or directory position. Rollback creates new immutable current, migration, self-check, audit, and incident records and never rewrites failed release history.

Revocation, retirement, and incidents remain distinct:

- revocation makes an exact artifact, manifest, release, key, target, or profile ineligible under policy;
- retirement ends ordinary support or channel eligibility without necessarily implying insecurity;
- incident records preserve affected identities, evidence, uncertainty, containment, remediation, and public advisory state.

## 18. Service, effects, and closure

`wow-service` owns:

```text
strict request validation
one-time symbolic selector resolution
exact retained owner acquisition
cross-owner compatibility validation
narrow port sequencing
durable operation and idempotency state
conservative result envelopes
retention, audit, and close-before-success
```

Every externally observable or durable effect registers `OperationId + CanonicalRequestDigest` before dispatch. Same ID and same digest returns or reconciles the same effect; same ID with another digest fails.

Timeout, disconnect, cancellation, process loss, serialization failure, or output failure after dispatch can produce `OutcomeUnknown`. Blind redispatch is forbidden. Public success waits for required read-back and validation, retention, audit, and reverse-order close.

Applications and tools parse, frame, transport, and render one service operation. They never own domain, release, or installation semantics.

## 19. Security and privacy

Public seams structurally reject or isolate:

```text
arbitrary source, Lua, repository, build, release, installer, or migration execution
raw SQL, database, provider, filesystem, process, and owner handles
generic MCP, tool, RPC, shell, script, plugin, callback, or model paths
private signing, provider, build, distribution, deployment, and installation material
unbounded source, graph, response, archive, queue, and stream data
source, provider, client, or release text as control instructions
implicit current, network, project, provider, installation, and update behavior
cross-client source, overlay, result, stream, and journal access
```

Effective output is the intersection of source, provider, project, session, consumer, privacy, license, notice, and distribution policies. Unknown state narrows or denies. Logs and errors default to stable IDs, counts, stages, and reason codes and redact source, secrets, private paths, session capabilities, owner handles, and raw payloads.

## 20. Launch and implementation

Launch states are defined in [`LAUNCH_GATES.md`](LAUNCH_GATES.md):

```text
R0 first runnable: I0-A through I0-F
A0 useful internal alpha: I1 through I3
A1 developer preview: I4 plus one complete I7-A frontend
B0 governed beta: I5; optional I6
V1-RC: selected scope plus complete I7-A and candidate I7-B pipeline
V1 public supported release: publication, install, update, rollback, and support closure
```

The workspace plan is [`WORKSPACE_AND_BUILD_PLAN.md`](WORKSPACE_AND_BUILD_PLAN.md). Agent order is [`IMPLEMENTATION_HANDOFF.md`](IMPLEMENTATION_HANDOFF.md). Required commands are [`CONFORMANCE_COMMANDS.md`](CONFORMANCE_COMMANDS.md). Current state is [`PROJECT_COMPLETION_MATRIX.md`](PROJECT_COMPLETION_MATRIX.md).

The next repository action is I0-A `wow-core`, not another speculative architecture package. An architecture change now requires a concrete implementation-discovered failure and the smallest tested seam or ADR change.
