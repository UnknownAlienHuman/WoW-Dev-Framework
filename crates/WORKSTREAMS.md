# Agent workstreams and integration order

**Status:** normative implementation routing after E7-B architecture completion.

```text
planned architecture/documentation: complete through E7-B
implementation: not started
next implementation package: I0-A / wow-core E0-A
```

Documentation-ready remains `implementation_state = not-started` until executable owner code, exact fixture and checksum closure, required adapters, probes, benchmarks, platforms, clients, runtime evidence, and acceptance tests exist.

## Global implementation order

```text
I0-A  wow-core
I0-B  wow-reference E0 fixture
I0-C  wow-emmy adapter
I0-D  wow-project E0 fixture
I0-E  wow-rules
I0-F  wow-service + apps/wow status/check

I1-A  wow-store foundation
I1-B  persistent wow-reference
I1-C  wow-annotations
I1-D  service + wow-reference-builder

I2-A  wow-graph
I2-B  wow-recognizers
I2-C  full wow-project indexing
I2-D  ProjectStore publication

I3-A  Blizzard UI source universe
I3-B  wow-context
I3-C  context service + CLI

I4-A  wow-search
I4-B  lineage, migration, and static impact
I4-C  service + CLI

I5-A  calibration owner
I5-B  review, holdout, and submission service
I5-C  core publication, canary, rollout, and rollback

I6-A  optional wow-cbm Candidate owner
I6-B  optional provider session, mapping, selection, and context

I7-A  sessions, overlays, and one-binary CLI, daemon, LSP, and MCP host
I7-B  release build, evidence, signing, bundle, channel, install, update, and support
```

## Global work rules

- One agent owns one primary implementation package, crate, and worktree until merged, quarantined, or deleted.
- Later documentation never bypasses prerequisite implementation and freeze gates.
- Activate only workspace members with a real implemented slice; no empty crates, placeholder public traits, or fake success.
- Missing implementation, tool, probe, benchmark, authorization, signing, provider, protocol, client, platform, runtime, installation, or checksum evidence is blocked or `NotEvaluated`, never pass.
- Patch-sensitive WoW claims route through the current external KB and exact source or runtime evidence.
- Repository, addon, owner, path, provider, popularity, model, client, build, release, and transport identity never becomes hidden semantics or authorization.
- Owner crates never depend on `wow-service`, applications, or tools.
- Applications and tools depend on `wow-service` only among framework crates.
- No generic shell, process, tool, RPC, SQL, provider API, installer, plugin, callback, or model executor.
- No implicit network, current, project, provider, installation, or update behavior.
- No CI or release workflow until a real frozen command exists and an explicit owner and gate require it.

## I0 — first runnable vertical slice

Primary contracts are the root E0 packages under:

```text
wow-core
wow-reference
wow-emmy
wow-project
wow-rules
wow-service
apps/wow
```

### I0-A — `wow-core`

Own:

```text
stable typed IDs and digests
profiles and generations
provenance and confidence
coverage and negative authority
conflicts, omissions, and evidence references
status, result, error, cancellation, and budgets
OperationId + CanonicalRequestDigest
canonical ordering and serialization
```

Hard stops: no project, store, graph, provider, session, release, or service semantics; no speculative wrappers; no timestamps or paths in semantic identity; no unknown-to-negative coercion; no future-only generic traits.

Completion requires every E0-A acceptance, property, mutation, canonicalization, resource, security, and forbidden-dependency case plus checksum closure.

### I0-B and I0-C — reference fixture and Emmy adapter

`wow-reference` implements one exact frozen ReferenceView fixture. `wow-emmy` pins one exact upstream analyzer implementation behind one adapter.

Completion requires source-coordinate, capability, cancellation, malformed and Unicode input, deterministic repeat, cleanup, and negative-authority behavior. No full Reference builder or second parser yet.

### I0-D, I0-E, and I0-F — project, diagnostics, service, CLI

Implement one immutable project fixture and three bounded diagnostics, then exact service orchestration and:

```text
wow status
wow check
```

R0 requires canonical output and exit behavior, clean, finding, partial, conflict, `NotEvaluated`, malformed input, cancellation, resource limits, broken pipe, and close-before-success.

## I1 — Reference Pack and storage

```text
wow-store       generic schema, object, effect, retention, and recovery foundation
wow-reference   exact source evaluation, raw facts, corrections, and ReferenceView
wow-annotations deterministic annotation projection, parity, and loss
wow-service     build, validation, and rebuild comparison
wow-reference-builder thin service client
```

I1 completes when one real Reference Pack can be built twice, compared, published, read back, queried with honest coverage, and restored under exact source, tool, evaluator, license, and checksum profiles.

## I2 — project, graph, and recognizer stack

```text
wow-graph        typed axes, partitions, queries, and evidence
wow-recognizers  universal declarative operators and rules
wow-project      TOC, XML, load, Lua physical and virtual units, invalidation
wow-store        WAL ProjectStore objects, generations, current CAS, recovery
```

I2 completes when a real addon revision is indexed nonexecutingly into exact project and graph generations and diagnostics preserve complete, partial, conflict, and `NotEvaluated` partitions.

## I3 — Blizzard UI and context

```text
wow-project  separate pinned Blizzard UI source project
wow-context  Project Map, L0, L1, bounded L2, semantic and rendered artifacts
wow-service  exact retained context use cases
apps/wow     one-call context commands
```

A0 completes when trustworthy real-addon diagnostics and bounded context work against one exact Reference and WoW profile with privacy, license, resource, cancellation, and recovery evidence.

## I4 — search, lineage, and impact

```text
wow-search  exact-generation structured, FTS5, text, and Candidate lanes
wow-graph   cross-generation lineage, change, migration, and impact
wow-service explicit selection, validation, and context handoff
apps/wow    thin commands
```

Completion requires exact lanes and proof ceilings before Candidate lanes, honest misses, snapshot-bound continuation, and real task benefit without treating similarity as authority.

## I5 — governed recognizer evolution

### I5-A

Implement admitted corpus, provenance groups, independent labels, leakage-safe splits, shadow packs, mutation and anti-overfitting, per-case metrics, graph and security reports, candidate and deactivation artifacts.

### I5-B

Implement durable runs, independent review authorization, sealed holdout authorization, vault, audit, disclosure and consumption state, and immutable `PromotionSubmission`.

### I5-C

Implement independent revalidation, distinct core artifact, attestations and signatures, inactive publication and read-back, exact canary, finite rollout, profile current CAS, LKG, rollback, revocation, deactivation, and stale partition closure.

E5 requires real admitted corpora, independent authorization, holdout, signing, and canary evidence. It does not block R0 or A0.

## I6 — optional external provider lane

### I6-A — `wow-cbm`

Implement reviewed descriptor, capability, state, query, transport, and normalization contracts with the hard `semantic_candidate + Candidate` ceiling, provider-local scores, unverified locators, zero-result honesty, continuation, cache, cancellation, and lane-local failure.

### I6-B — service and owner seams

Implement exact provider configuration and session authorization, durable result catalogs, exact project or reference owner mapping, explicit caller selection, and exact-root context with a separate provider sidecar.

Hard stops:

```text
no provider database or index lifecycle in wow-cbm
no generic MCP or provider tool
no cross-provider score fusion or Candidate promotion
no service-side path or source mapping
no implicit top or sole selection
no provider metadata in ContextSemanticPack
no hidden fallback or local capability downgrade
```

E6 can ship disabled. Enable only after one real adapter proves unique benefit and passes credential, privacy, license, outage, degradation, mapping, selection, and resource tests.

## I7-A — product sessions and frontends

Primary routes:

- [`wow-service/e7/`](wow-service/e7/README.md)
- [`../apps/wow/e7/`](../apps/wow/e7/README.md)
- [`wow-project/E7_A_DOCUMENT_OVERLAYS.md`](wow-project/E7_A_DOCUMENT_OVERLAYS.md)
- [`wow-emmy/E7_A_OVERLAY_ANALYSIS.md`](wow-emmy/E7_A_OVERLAY_ANALYSIS.md)
- [`wow-rules/E7_A_LIVE_DIAGNOSTICS.md`](wow-rules/E7_A_LIVE_DIAGNOSTICS.md)
- [`wow-store/E7_A_SESSION_AND_RESPONSE_JOURNAL.md`](wow-store/E7_A_SESSION_AND_RESPONSE_JOURNAL.md)

### Flow

```text
content-addressed FrontendOperationRegistry and compatibility profile
-> immutable frontend and service session generations
-> explicit workspace, project, and profile binding
-> project-owned document overlay generations
-> exact diagnostics and language features
-> bounded progress, streams, and backpressure
-> cancellation, disconnect, replay, reconnect, and reconciliation
-> one wow binary: CLI, local daemon, LSP 3.18, MCP 2025-11-25
```

### Host rules

```text
one-shot CLI direct
local daemon over current-user Windows named pipe or Unix socket
LSP 3.18 over stdio
MCP 2025-11-25 over stdio
optional local MCP Streamable HTTP, explicit and disabled by default
no hidden auto-start, fallback, or default remote listener
```

LSP uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Stale or out-of-order versions require resynchronization. MCP is fixed and read-only by default. Code actions are guarded data and never automatically applied.

A1 uses one complete implemented frontend profile. V1 requires the complete selected E7-A Windows profile and cross-transport conformance.

## I7-B — release and support

Primary routes:

- [`wow-service/e7b/`](wow-service/e7b/README.md)
- [`../apps/wow/e7b/`](../apps/wow/e7b/README.md)
- [`../tools/wow-release/`](../tools/wow-release/README.md)
- [`wow-store/E7_B_RELEASE_STORAGE.md`](wow-store/E7_B_RELEASE_STORAGE.md)

### Flow

```text
exact source tree, lockfile, toolchain, target, and dependencies
-> typed isolated build executor
-> at least two independent unsigned builds
-> reproducibility comparison
-> artifact validation and complete required tests
-> SBOM, provenance, license, notices, and checksums
-> portable and platform signatures
-> deterministic ReleaseBundle
-> support matrix and immutable ReleaseCandidate
-> provider-neutral publication, public read-back, and channel CAS
-> signed update manifest
-> staged verified install, migration, current CAS, self-check, and LKR
-> exact rollback, revocation, retirement, and incident lifecycle
```

### First target intent

```text
x86_64-pc-windows-msvc
```

It remains unsupported until complete Windows build, signing, path and ACL, named-pipe, console, LSP and MCP, helper, migration, clean install, update, rollback, real-addon, and support evidence passes.

### Release hard stops

```text
no arbitrary command, environment, network, SQL, or provider executor
no one-build reproducibility claim
no in-place release asset replacement
no GitHub, tag, CI, or account identity as authorization or trust
no hidden update, telemetry, crash upload, or remote configuration
no app-side self-overwrite or raw migration script
no previous or newest LKR and rollback inference
no blind build, sign, upload, install, update, rollback, revoke, or retire retry
```

CI is a late E7-B phase after exact commands exist. It invokes the service-backed release tool; it does not define a second release pipeline.

## Launch routing

See [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md):

```text
R0 = I0
A0 = R0 + I1 through I3
A1 = A0 + I4 + one complete I7-A frontend
B0 = A1 + I5; optional I6
V1-RC = selected scope + complete I7-A + candidate I7-B pipeline
V1 = V1-RC + public publication, install, update, rollback, and support evidence
```

## Seam request format

Only when implementation proves the accepted seam insufficient, state:

```text
requesting package and crate
owning crate
exact failing operation or crossing data
rejected workaround
why the current seam cannot implement it
smallest proposed seam
cycle, identity, security, privacy, license, supply-chain, and evidence impact
fixtures and mutations proving the need
compatibility, migration, and freeze consequences
```

Do not implement a missing seam in the wrong crate.

## Next work

```text
I0-A / crates/wow-core E0-A
```

Finish, merge, and close that worktree before activating I0-B or I0-C.
