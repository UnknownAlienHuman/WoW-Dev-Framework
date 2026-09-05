# Implementation handoff

> Current implementation and update policy: [status ledger](IMPLEMENTATION_STATUS.md). Earlier bootstrap schedules below are design history, not instructions to recreate the workspace or permanently pin versions.

**Status:** normative execution plan after architecture/documentation freeze.

## Handoff state

```text
documentation frontier: E7-B / planned architecture
implementation: partial executable foundation and source bridge
active workspace: wow-core, wow-reference
next owner implementation: I0-C / real wow-emmy analyzer adapter
full R0, runtime, installation and release gates: NotEvaluated
```

This document turns the completed contracts into agent-sized implementation work. It does not authorize bypassing freeze gates or writing all crates at once.

## Global execution rule

One agent owns one implementation package/crate/worktree until it is merged, quarantined or deleted. An agent reads repository instructions, the target package and all prerequisite public contracts before coding.

Every implementation PR states:

```text
owned work package and crate
exact prerequisite commits/profiles/fixtures
public operations/types implemented
new dependencies/features/platform code
fixtures and checksum fields populated
commands/tests with pass/fail/skipped
NotEvaluated capabilities and blockers
launch gate advanced or unchanged
```

No package is marked implemented from compilation alone.

## I0-A — `wow-core` E0-A

### Responsibility

Implement the irreducible shared semantic primitives:

```text
stable typed IDs and digests
ProfileId and exact generation identities
provenance and confidence
coverage partitions and negative authority
conflicts, omissions and evidence references
status/result/error/cancellation/budget primitives
canonical ordering and serialization
OperationId + CanonicalRequestDigest primitives
```

### Inputs

- `crates/wow-core/` complete E0-A package;
- root glossary/provenance/decisions;
- frozen Rust/toolchain/dependency profile;
- E0-A fixtures/checksums.

### Hard stops

- no store/project/graph/service semantics;
- no generic string IDs without validation/canonicalization;
- no wrapper type with no invariant;
- no hidden timestamps/paths/process IDs in semantic identities;
- no empty/default success or unknown-to-negative coercion;
- no broad abstraction for future packages without two concrete owned consumers.

### Definition of done

All E0-A acceptance/property/canonical/mutation/resource tests pass; required fixture/checksum fields are populated; crate has no forbidden dependency; API is sufficient for the exact E0-B–F crossing data already documented.

## I0-B — `wow-reference` E0-B fixture slice

Implement one exact immutable ReferenceView fixture, profile/source identities, lookup, raw unknown fields, coverage and negative authority. No downloader or full ReferenceStore yet.

Done when clean found/missing/partial/conflict/`NotEvaluated` cases and canonical fixture bytes pass against `wow-core`.

## I0-C — `wow-emmy` E0-C adapter

Pin one exact upstream Rust EmmyLua implementation and expose only the documented adapter. Implement source/VFS/profile initialization, syntax/semantic facts, generic diagnostics, source coordinates, cancellation and capability reports.

Done when compatibility probes, malformed source, Unicode/range, deterministic repeat and clean shutdown pass. No second Lua parser.

## I0-D — `wow-project` E0-D fixture slice

Build one immutable project generation from explicit frozen source/configuration, ReferenceView and analyzer inputs. Own source handles and nonexecuting file acquisition.

Done when source/profile/generation closure, malformed/path attacks, partial analyzer capability, cancellation and deterministic project bytes pass.

## I0-E — `wow-rules` E0-E

Implement the first bounded rules:

```text
one generic Emmy diagnostic projection
one WoW API existence/signature diagnostic
one local Secret/restriction operation diagnostic
```

Each declares exact capability/coverage requirements and root-cause folding. Autofix only with exact guards; otherwise plan/candidate.

Done when clean/finding/partial/conflict/`NotEvaluated`/cancelled/malformed/mutation cases pass.

## I0-F — `wow-service` + `apps/wow`

Implement thin status/check orchestration and CLI:

```text
wow status
wow check
```

Service acquires exact owners, validates closure, invokes operations and closes resources. CLI parses/prints only.

R0 is complete only when both commands run from a clean build against frozen fixtures, canonical JSON/text/exit codes pass, broken pipe/cancellation/resource closure work and all E0 checksum manifests are populated.

## I1 — storage and full Reference Pack

### I1-A `wow-store`

Implement registered schemas/migrations, content-addressed objects, immutable publication/read-back, WAL/transactions, leases/retention/GC, backup/restore and exact effect reconciliation.

### I1-B `wow-reference`

Implement pinned source inputs, restricted APIDocumentation evaluation, raw metadata, normalization, corrections, persistent ReferenceStore/View and negative-authority coverage.

### I1-C `wow-annotations`

Implement deterministic annotation projection, source maps, unsupported/loss records and Ketho/Numy differential fixtures.

### I1-D service and `wow-reference-builder`

Implement exact build/validate/rebuild-compare use cases and thin tool.

I1 completes when one real Reference Pack can be independently built, read back, reproduced and queried under one exact WoW profile.

## I2 — graph, recognizers, project indexing and ProjectStore

### I2-A `wow-graph`

Implement typed entities/relations/partitions/queries, explicit axes, producer/evidence/coverage/conflict state and immutable snapshots.

### I2-B `wow-recognizers`

Implement declarative universal rule operators over Emmy/project facts, candidate partitions and graph proposal validation.

### I2-C `wow-project`

Implement TOC variants, XML/templates/mixins, load order, Lua physical/virtual units, incremental invalidation and exact project generation candidates.

### I2-D `wow-store`

Implement the selected ProjectStore WAL manifested-partition publication, current CAS, retained readers and recovery.

I2 completes when a real addon revision is indexed nonexecutingly into exact project/graph generations and diagnostics can distinguish complete, partial and `NotEvaluated` partitions.

## I3 — Blizzard UI source and context

### I3-A project source universe

Implement separate exact Blizzard UI source projects/graphs from pinned content; never merge with user projects or the default Emmy library workspace.

### I3-B `wow-context`

Implement Project Map, L0, L1, bounded L2/source, exact expansion budgets, semantic packs and renderers.

### I3-C service/app

Implement context status/map/inspect/build/continue/validate/render with exact retained view acquisition.

A0 is complete when real addon tasks can obtain trustworthy diagnostics and bounded architecture/source context against one exact Reference/WoW profile.

## I4 — search, lineage, migration and impact

### I4-A `wow-search`

Implement exact-generation shards, B-tree/FTS5 lanes, structured ranking, explanations, honest misses and snapshot-bound continuation.

### I4-B `wow-graph` and producer seams

Implement cross-generation identity/transition/change/migration/static-impact records with proof ceilings and ambiguity.

### I4-C service/app

Implement explicit candidate selection, review authorization, migration validation, impact and search-to-context.

I4 is complete when search/lineage/context tasks on exact real addon generations demonstrate measured useful results without treating similarity as proof.

## I5 — governed recognizer evolution

### I5-A calibration owner

Implement admitted corpus materialization, provenance groups, independent labels, leakage-safe splits, shadow packs, mutations, metrics, candidate and deactivation artifacts.

### I5-B review/holdout/submission service

Implement durable runs, authorization, sealed holdout vault/audit/consumption, immutable reviews and PromotionSubmission.

### I5-C core publication lifecycle

Implement independent revalidation, core artifact, attestations/signatures, inactive publication/read-back, canary, finite rollout, profile current CAS, LKG, rollback/revocation/deactivation and stale partition closure.

I5 is complete only with admitted real corpora and external authorization/holdout/signing/canary evidence. It is not required for R0/A0.

## I6 — optional external semantic candidate lane

### I6-A `wow-cbm`

Implement reviewed provider descriptors, state classes, bounded typed transport, Candidate-only normalization, provider-local scores, unverified locators, zero-result honesty and cache/continuation.

### I6-B service/owner seams/app

Implement exact provider configuration/session authorization, durable result catalogs, project/reference owner mapping, explicit selection and exact-root context sidecar.

E6 remains disabled by default until one real adapter demonstrates unique benefit and all credential/privacy/license/degradation tests pass.

## I7-A — product host and editor clients

### Service/session layer

Implement static operation registry/compatibility manifest, immutable sessions, project/profile binding, project-owned overlays, editor use cases, streams/backpressure and recovery.

### `apps/wow`

Implement one binary with direct CLI, local named-pipe daemon, standalone LSP stdio and standalone MCP stdio.

Implement the exact E7-A LSP 3.18 synchronization contract: incremental `textDocument/didChange`, with a full-document change accepted as an exact replacement. Advertise it only after the overlay, position-encoding, version, resynchronization, and client conformance suites pass. Keep the developer exposure read-only by default.

A1 is complete when an admitted editor/MCP client can use diagnostics, hover, navigation, symbols, search/context and guarded code actions on real addons with exact version/session behavior and measured latency/resources.

## I7-B — release, install and support

### Release service

Implement source/materialization/build/reproducibility/evidence/signing/bundle/support/candidate/channel/update/install/rollback/revocation/retirement/incident operations.

### Internal tool

Implement `wow-release` as a strict service-only client.

### Public update client

Implement `wow version`, local verification and explicit update check/plan/apply/rollback/reconcile.

### First target

Complete Windows x86-64 MSVC build, code signing where required, portable bundle, local installation/helper/update/rollback, daemon/LSP/MCP client tests and clean-machine rehearsal.

V1 is complete only after the entire E7-B candidate/channel/install/support matrix passes and a signed exact public release is read back and installed/updated/rolled back successfully.

## Cross-cutting implementation responsibilities

### Contract/fixture validator

Implement deterministic repository tooling that validates:

```text
JSON/schema parsing and duplicate keys
contract ID uniqueness
normative/member path closure
public operation and CLI/tool mapping closure
fixture case IDs against test matrices
fixture error codes against error models
dependency graph and manifest consistency
implementation-state/freeze-gate/checksum closure
Markdown internal links and fences
forbidden files/secrets/placeholders
```

This validator is a real build/test command before CI is added.

### Canonicalization

Each owner defines canonical semantic bytes; one shared `wow-core` foundation handles generic primitives only. Physical database/archive/protocol bytes remain separate from semantic identities where documented.

### Failure injection

Every effecting owner exposes deterministic test hooks/adapters for failure before/after prepare/dispatch/commit/read-back/retention/close/response delivery. Production has no generic fault-injection public API.

### Security

Threat-model tests are mandatory, not follow-up cleanup. No source execution, generic shell/tool/RPC, raw SQL, secrets in public seams, implicit network/current state or unbounded inputs.

### Benchmarks

Benchmarks record exact build/profile/corpus/hardware class and thresholds. A missing benchmark is `NotEvaluated`; a faster result cannot override correctness/security/coverage gates.

## Pull-request granularity

A preferred implementation PR:

```text
implements one coherent contract slice
adds/finalizes its fixtures and checksums
contains focused public API and tests
has no unrelated refactor
updates manifest/router/launch gate only for real progress
leaves later packages documented but not activated
```

Large package work can be split by internal phases only when each slice has a real independently testable owner boundary and no fake pass/stub public operation.

## When to change architecture

Implementation difficulties are not automatically architecture defects. Request a seam/ADR change only with:

```text
exact failing contract/use case
why the owning package cannot implement it
why existing public seam is insufficient
smallest proposed crossing data/operation
cycle/identity/security/privacy/license/evidence impact
fixtures/mutations proving the need
migration and compatibility consequences
```

Otherwise implement the accepted contract.

## Completion rule

The project is fully implemented only when [`PROJECT_COMPLETION_MATRIX.md`](PROJECT_COMPLETION_MATRIX.md) has no required `not-started`, blocked, skipped or `NotEvaluated` implementation gate for the selected V1 scope and the exact E7-B release candidate passes public read-back, clean install, update, rollback and verification.
