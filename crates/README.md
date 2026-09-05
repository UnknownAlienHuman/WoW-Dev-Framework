# Crate implementation contracts

**Planned architecture and documentation:** complete through E7-B. **Implementation:** not started.

A documented directory is not an activated Rust crate. No `Cargo.toml`, `.rs` placeholder, broad trait surface, or workspace membership is created before the owned implementation slice, toolchain and dependency profile, fixtures, and first-commit freeze gate are ready.

## Required reading

1. [`../AGENTS.md`](../AGENTS.md)
2. [`AGENTS.md`](AGENTS.md)
3. [`MANIFEST.json`](MANIFEST.json)
4. [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
5. [`WORKSTREAMS.md`](WORKSTREAMS.md)
6. [`../docs/LAUNCH_GATES.md`](../docs/LAUNCH_GATES.md)
7. [`../docs/WORKSPACE_AND_BUILD_PLAN.md`](../docs/WORKSPACE_AND_BUILD_PLAN.md)
8. [`../docs/IMPLEMENTATION_HANDOFF.md`](../docs/IMPLEMENTATION_HANDOFF.md)
9. [`../docs/CONFORMANCE_COMMANDS.md`](../docs/CONFORMANCE_COMMANDS.md)
10. the target crate, application, or tool router and complete owned contract
11. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.
12. the actual addon repository and local instructions for addon-facing work

## Final production ownership

| Component | Primary ownership | Planned frontier | First implementation |
|---|---|---:|---:|
| `wow-core` | typed IDs and digests, profiles and generations, evidence, confidence, coverage, conflicts, canonical results | E0-A | I0-A |
| `wow-store` | generic immutable objects, catalogs, effects, CAS, leases, retention, GC, migrations, and recovery | E7-B storage seams | I1-A |
| `wow-reference` | Reference Pack and View, raw metadata, corrections, coverage, transitions, and exact locator mapping | E6-B mapping seam | I0-B / I1-B |
| `wow-annotations` | deterministic annotation projections, source maps, loss, and parity | E1-C | I1-C |
| `wow-emmy` | pinned upstream Lua analyzer adapter, facts, diagnostics, and overlay analysis | E7-A overlay seam | I0-C |
| `wow-project` | source, TOC, XML, load order, project generations, reindexing, locator mapping, and document overlays | E7-A overlay seam | I0-D / I2-C |
| `wow-graph` | typed graph, relation partitions, lineage, migration, impact, and closure | E5-C and E7 inputs | I2-A |
| `wow-recognizers` | universal structural rules, calibration, and core-pack semantics | E5-C | I2-B |
| `wow-rules` | capability-gated diagnostics, findings, and remediation tiers | E7-A live diagnostics seam | I0-E |
| `wow-search` | exact-generation retrieval, ranking, explanations, and continuation | E4-A | I4-A |
| `wow-context` | Project Map, L0, L1, bounded L2, semantic packs, and rendering | E6-B handoff | I3-B |
| `wow-cbm` | optional external Candidate-only normalization | E6-A | I6-A |
| `wow-service` | multi-owner use cases, sessions, effects, release lifecycle, and canonical envelopes | E7-B | I0-F onward |

## Applications and tools

```text
apps/wow
    public product executable
    one-shot CLI, local daemon, LSP 3.18, MCP 2025-11-25
    local release verification and explicit update/rollback client

apps/wow-reference-builder
    internal Reference Pack build, validation, and rebuild-comparison client

tools/wow-release
    internal release source, build, evidence, signing, bundle, channel,
    revocation, retirement, and reconciliation client
```

Every application and tool depends on `wow-service` only among framework crates and invokes one service operation per semantic command, method, tool, or resource request unless a composite workflow is itself one documented service operation.

## Package stack

```text
E0-A through E0-F
E1-A through E1-D
E2-A through E2-D
E3-A through E3-C
E4-A through E4-C
E5-A through E5-C
E6-A through E6-B
E7-A through E7-B
```

The next work package is not E8. It is **I0-A / `wow-core` E0-A** implementation.

## E5 separation

```text
E5-A calibration evidence and shadow candidates
E5-B durable runs, review, sealed holdout, and PromotionSubmission
E5-C immutable core artifact, signing, inactive publication, canary,
     guarded activation, finite rollout, LKG, rollback, revocation, and closure
```

No metric, review, holdout result, signature, publication, or canary state silently authorizes another gate.

## E6 separation

```text
E6-A wow-cbm owns pure external Candidate normalization
E6-B wow-service coordinates configuration, session, result catalog,
     exact owner mapping, caller selection, and exact-root context
```

E6 remains optional and may ship disabled. Mapping and selection never verify provider interpretation.

## E7-A separation

```text
owner crates
    project workspace and overlay identity
    Emmy overlay analysis
    diagnostics, search, and context results
    generic store, response journal, lease, and retention

wow-service
    immutable FrontendOperationRegistry
    session, workspace, project, profile, and document orchestration
    exact request, result, effect, and delivery state

apps/wow
    one-shot CLI
    foreground local daemon
    LSP 3.18 stdio
    MCP 2025-11-25 stdio
    optional local-only MCP HTTP, disabled by default
```

LSP 3.18 uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Transports do not own semantic algorithms, advertise missing capabilities, infer workspaces, expose generic tools, treat disconnect as cancellation, or mutate source automatically.

## E7-B separation

```text
release owners and adapters
    exact source materialization, typed build execution, SBOM and provenance,
    signing, provider-neutral distribution, installation, migration, and support

wow-service
    exact release lifecycle orchestration, gates, durable effects,
    expected-current CAS, reconciliation, retention, audit, and envelopes

apps/wow/e7b
    local verification and explicit update, rollback, and reconciliation client

tools/wow-release
    internal one-command-to-one-service-operation release client
```

A compiled binary, successful build, signature, uploaded archive, channel record, or installation never substitutes for the remaining independent release gates.

## Implementation and launch order

```text
I0-A wow-core
-> I0-B reference fixture + I0-C Emmy adapter
-> I0-D project fixture
-> I0-E rules
-> I0-F service + wow status/check              = R0

-> I1 persistent Reference stack
-> I2 graph, recognizers, project, ProjectStore
-> I3 Blizzard UI source and context             = A0

-> I4 search, lineage, migration, impact
-> selected I7-A frontend                        = A1

-> I5 governed recognizer lifecycle
-> optional I6 external provider lane            = B0 scope

-> complete I7-A host
-> I7-B build, sign, bundle, install, update      = V1
```

See [`../docs/PROJECT_COMPLETION_MATRIX.md`](../docs/PROJECT_COMPLETION_MATRIX.md).

## Ownership rules

- Exact owner facts stay inside owner crates and cross seams as immutable typed records or views.
- `wow-core` and `wow-store` never absorb domain semantics to solve dependency problems.
- `wow-service` coordinates and validates; it never reproduces owner algorithms.
- Applications, transports, and tools parse, frame, and emit only.
- Allowed dependency lists are maxima; the active implementation slice is narrower.
- Every durable or external effect uses exact operation identity, reconciliation, retention, audit, and close-before-success.

## Evidence and security rules

- No mixed profile or generation result and no empty default success.
- Missing capability, evidence, adapter, test, benchmark, client, platform, or runtime input is `NotEvaluated` or blocked, not pass.
- Similarity, rank, score, top, sole, repeated, and provider labels remain Candidate signals.
- Review, holdout, signing, publication, activation, distribution, installation, support, and runtime states remain independent.
- No arbitrary source or repository execution, raw SQL or database handles, generic MCP, tool, RPC, shell, script, plugin, or model executor.
- No secret signing, provider, build, distribution, deployment, or installation material in repository, public configuration, fixtures, logs, or results.
- No implicit network, current, project, provider, installation, or update behavior.
- No CI or release workflow until real frozen commands exist and an explicit owner and gate require it.

## First implementation target

```text
owner: wow-core
implementation package: I0-A
contract: crates/wow-core/CONTRACT.json
launch gate: R0
first workspace members: crates/wow-core only
```

Freeze the exact Rust toolchain and minimal dependencies, implement the complete E0-A invariant types and tests, populate its fixtures and checksum gate, and merge it before activating sibling crates.
