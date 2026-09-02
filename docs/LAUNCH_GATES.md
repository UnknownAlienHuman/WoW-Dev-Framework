# Launch gates

**Status:** normative operational plan.

This document separates four different states that must not be conflated:

```text
documented architecture
first runnable executable
useful internal/developer alpha
public supported v1 release
```

A completed documentation package is not implemented software. A compiling binary is not a useful alpha. An internal alpha is not a supported public release.

## Current state

```text
documentation frontier: E6-B
implementation frontier: not-started
Cargo workspace: absent
Rust source: absent
CI/workflows: absent
real end-to-end implementation evidence: absent
```

E6-B completes the planned pre-transport service architecture. E7 remains the documentation frontier for concrete LSP/MCP/daemon/session and release/distribution surfaces.

## Gate R0 — first runnable bootstrap

**Goal:** one deterministic local executable path, not the full product.

Critical implementation order:

```text
E0-A wow-core
-> E0-B frozen wow-reference fixture
+  E0-C pinned wow-emmy adapter
-> E0-D minimal wow-project generation
-> E0-E bounded wow-rules diagnostics
-> E0-F wow-service + apps/wow
```

Required completion evidence:

- activate only the E0 Cargo workspace members and dependency edges;
- implement canonical identities, profiles, generations, evidence, coverage, results, errors, and serialization in `wow-core`;
- freeze one exact ReferenceView fixture and its checksums;
- pin one upstream EmmyLua implementation behind the documented adapter and pass its contract probes;
- build one exact minimal project generation from frozen source/configuration fixtures;
- execute the E0 diagnostic rules with clean, finding, partial, conflict, `NotEvaluated`, malformed-input, cancellation, and deterministic-repeat cases;
- expose `wow status` and `wow check` through one service call per command;
- pass exact canonical byte, exit-code, profile-isolation, resource-bound, and cross-platform path tests for the selected launch platform profile;
- populate every required E0 implementation/profile/fixture/checksum field currently left null.

**R0 does not require E1–E7 implementation.** It is the earliest point at which the repository becomes a runnable project rather than a documentation scaffold.

## Gate A0 — useful internal alpha

**Goal:** analyze a real addon repository against one exact current WoW profile with trustworthy local results.

Required packages:

```text
E1 Reference Pack build/validation
E2 graph + recognizers + project indexing + ProjectStore
E3 Blizzard UI source universe + Project Map/L0/L1/context
```

Required evidence:

- build and read back one real immutable Reference Pack from pinned source inputs;
- implement TOC/XML/load/source indexing without executing analyzed repositories;
- publish one real addon project generation and one separate Blizzard UI source generation;
- run EmmyLua analysis, declarative recognizers, graph publication, diagnostics, and bounded context generation;
- prove clean-negative authority only for complete exact partitions;
- test malformed/untrusted repositories, load variants, partial source, profile mismatch, Secret/restriction unknowns, cancellation, recovery, retention, and deterministic rebuilds;
- recheck every patch-sensitive claim through the current `wow-addon-engineering-kb`, pinned Blizzard source, and required runtime probes before declaring profile support.

A0 may remain CLI-only. LSP/MCP and public packaging are not required yet.

## Gate A1 — developer preview

**Goal:** make the internal alpha efficient enough for repeated real development work.

Required packages:

```text
E4 exact-generation search
E4 lineage/migration/static-impact candidates
E7-A minimal supported transport/session surface
```

Required evidence:

- exact/alias/deprecation/replacement lanes precede fuzzy/semantic candidates;
- search explanations expose lanes, scores, coverage, and Candidate ceilings;
- context/search continuations remain exact-generation-bound;
- one supported frontend, normally CLI plus one LSP or MCP integration, maps each tool/command to one service operation;
- cancellation, disconnect, reconnect, broken pipe, concurrent requests, backpressure, output limits, and session closure are tested;
- no generic tool-call or shell escape hatch bypasses service contracts;
- real addon task evaluation demonstrates measurable benefit over manual repository reading.

A1 is the first sensible external developer preview. It is not yet a stable public v1.

## Gate B0 — governed beta

**Goal:** enable safe evolution of structural recognizers and optional external semantic candidates.

### E5 requirements

Before enabling automated calibration/core-pack lifecycle:

- admit real licensed calibration corpora with provenance groups and leakage-safe splits;
- obtain independent labels/reviews and sealed-holdout evidence;
- run real mutations, per-case metrics, graph/security/determinism tests, and measured thresholds;
- build, attest, sign, publish inactive, read back, canary, roll out, designate LKG, and prove rollback/stale-partition closure;
- preserve all authorization, response-loss, retention, audit, license, privacy, and nonclaim boundaries.

E5 is not required for the first hard-coded/frozen diagnostic bootstrap. It is required before claiming safe automated core-pack promotion or rollout.

### E6 requirements

The external provider lane is optional and may ship disabled. Before enabling it:

- implement and probe at least one reviewed E6-A provider adapter;
- implement E6-B exact configuration/session authorization, durable query/result catalogs, project/reference owner mapping, explicit selection, and exact-root context handoff;
- prove `semantic_candidate + Candidate`, zero-result honesty, provider-local scoring, optional degradation, no hidden fallback, and no credential/source leakage;
- demonstrate task benefit after candidate verification cost.

E6 unavailability must never block exact local operation.

## Gate V1 — public supported release

**Goal:** a reproducible, installable, supportable product release.

Required E7 documentation and implementation:

```text
E7-A LSP/MCP/CLI-daemon/session protocols and capability negotiation
E7-B packaging, release artifacts, signatures/checksums, update channels,
     compatibility/support policy, rollback and retirement
```

Required release evidence:

- frozen supported OS/architecture/client/profile matrix;
- reproducible release build from pinned Rust/toolchain/dependency inputs;
- exact binaries/packages, checksums, signatures, SBOM, license/notices, provenance attestations, and verification instructions;
- secure configuration and credential setup that never writes secrets into repository/config examples/logs;
- installer/package layout and uninstall/upgrade/rollback behavior;
- immutable Reference Pack/core-pack/provider-adapter compatibility manifests;
- end-to-end tests on every supported platform/profile, including corrupted stores, interrupted upgrades, response loss, cancellation, stale generations, and rollback;
- documented data locations, retention/GC, backup/restore, privacy, telemetry policy, and incident response;
- user documentation for installation, project registration, profile selection, `status`, `check`, search/context, diagnostics, troubleshooting, and safe removal;
- performance/resource budgets measured on representative small, medium, and large addons;
- a release candidate tested on real addon repositories without executing their code;
- no unresolved release-blocking security, data-loss, profile-mixing, false-negative-authority, credential-leak, or rollback issue.

CI/release automation may be added only when it runs the real frozen commands and has an explicit owner. Decorative workflows remain forbidden.

## Critical-path summary

```text
First runnable project:
    implement E0-A -> E0-F

Useful internal alpha:
    R0 + E1 + E2 + E3

Developer preview:
    A0 + E4 + minimal E7-A frontend

Governed beta/full planned intelligence:
    A1 + E5; optionally E6 when enabled

Public supported v1:
    selected beta scope + complete E7-A/E7-B release gates
```

## Current blockers

At the current repository state, every implementation gate is still open:

```text
no Cargo workspace or Rust source
no implemented owner ports or adapters
no frozen implementation commits
no populated checksum manifests for implementation
no real Reference Pack or project generation
no real analyzer/diagnostic/search/context execution
no admitted calibration corpus or published core pack
no live external provider adapter/mapping/context evidence
no LSP/MCP/daemon transport
no packaged or signed release artifact
no current-client release validation
```

The shortest valid route to a runnable result is therefore E0 implementation, not more high-level architecture work after E7 contracts are frozen.