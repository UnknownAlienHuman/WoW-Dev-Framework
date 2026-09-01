# E3-C implementation plan

**Status:** normative order; implementation has not started.

## Phase 0 — prerequisite and byte freeze

Before any E3-C Rust/Cargo change:

- implement/freeze all E0-E2 prerequisites plus E3-A source index and E3-B context contracts required by active profiles;
- freeze exact `wow-store`, `wow-project`, `wow-graph`, `wow-reference`, and `wow-context` owner-port APIs;
- freeze selector, compatibility, alias/default, operation, status, failure, privacy, continuation-retention, envelope, canonicalization, app-handoff, and error profiles;
- freeze synthetic, pinned `roth-ui`, pinned Blizzard UI, combined-universe, race/failure, artifact, continuation, privacy, and CLI corpora;
- populate every fixture ID, exact expected JSON/artifact byte digest, exit mapping, benchmark/evaluation gate, and SHA-256 manifest;
- verify E0 status/check and E1 Reference Pack regressions remain green.

## Phase 1 — public service request/result primitives

Implement closed typed requests/selectors/guards/statuses/envelopes/errors and canonicalization. No owner access yet.

Tests: `SVC3-CFG-*`, `SVC3-REQ-*`, envelope identity-DAG mutations.

## Phase 2 — narrow owner ports

Implement only reviewed traits/adapters:

```text
ProjectPublicationAcquirePort
ReferenceAcquirePort
ContextEnginePort
ContinuationRetentionPort or owner-specific typed operations
```

Concrete adapters may call public store/project/graph/reference/context operations. No raw handles cross the seam.

## Phase 3 — selector resolution and acquisition

Implement fixed-order resolve-once logic, exact guards, exact reference derivation, capability collection, compatibility validation, and `ContextUniverseSet` binding.

Tests: `SVC3-SEL-*`, `SVC3-ACQ-*`, current-change and cross-store incompatibility races.

## Phase 4 — lifecycle guard

Implement private acquisition stack/RAII, reverse close, bounded secondary failures, cancellation checkpoints, panic-safe unwind where supported, and `ResourceClosureReport`.

No public success before close.

Tests: `SVC3-LIFE-*`, cancellation at every stage.

## Phase 5 — `context_status`

Implement bounded metadata acquisition and exact capability/profile/generation status without invoking map/build or claiming tests passed.

## Phase 6 — `context_map`

Delegate exact universe binding and Project Map generation/validation. Return unchanged artifact.

## Phase 7 — `context_inspect`

Delegate exact L0/L1 and permitted bounded expansion. Preserve root mapping, paths, confidence, coverage, conflicts, omissions, and continuation.

## Phase 8 — `context_build`

Delegate E3-B semantic pack build, validation, optional renderers, continuation retention admission, and status derivation.

## Phase 9 — `context_continue`

Validate continuation, reopen exact retained generations without current, invoke E3-B continuation, replace/release retention receipts, close resources.

## Phase 10 — `context_validate` and `context_render`

Implement bounded artifact transport values, structural/exact-owner validation, renderer delegation, invalid-artifact semantic payload, and nonrepair behavior.

## Phase 11 — canonical envelopes

Implement result reference closure, status precedence, selector/acquisition metadata, safe closure reports, deterministic ordering, and byte vectors. No CLI fields.

## Phase 12 — `apps/wow` E3-C adapter

Implement only after service result/request bytes freeze. App imports service only, handles config/artifact transport, signals, output modes, exit codes, and no reinvocation.

## Phase 13 — integration and evaluation

Run:

- every service/app test matrix case;
- 1/2/N and shuffled owner response schedules;
- current activation races before/after each resolution step;
- old/new exact readers and continuation retention/GC races;
- failure/cancel/close injection at every stage;
- synthetic, `roth-ui`, Blizzard UI, combined-universe and hostile artifact/source corpora;
- canonical service JSON and direct artifact bytes;
- E0/E1 regression suites;
- checksum verification.

## Suggested internal modules

```text
config
request
selectors
owner_ports
acquisition
compatibility
lease_guard
continuation_retention
operations/status
operations/map
operations/inspect
operations/build
operations/continue
operations/validate
operations/render
status
result
canonical
security
validation
```

Do not create placeholder modules for search, lineage, models, transports, runtime, editing, or releases.

## Deferred

- E4 root search/lineage/impact;
- E5 calibration-pack service operations;
- E6 Codebase Memory candidates;
- E7 LSP/MCP/HTTP/daemon/release operations;
- physical context cache;
- source edits/tool authorization/runtime probes;
- CI unless explicitly requested.
