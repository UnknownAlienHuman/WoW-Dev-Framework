# Launch gates

**Status:** normative operational plan.

This document separates states that must not be conflated:

```text
documented architecture
first runnable executable
useful internal alpha
developer preview
governed beta
public supported v1
```

A completed documentation package is not implemented software. A compiling binary is not a useful alpha. A working LSP/MCP transport is not a supported public release.

## Current state

```text
documentation frontier: E7-A
implementation frontier: not-started
Cargo workspace: absent
Rust source: absent
CI/workflows: absent
real end-to-end implementation evidence: absent
```

E7-A completes the frontend operation-registry/session/daemon/LSP/MCP contracts. E7-B remains the final documentation package for build/package/distribution/update/support. Implementation still begins with E0.

## Gate R0 — first runnable bootstrap

**Goal:** one deterministic local executable path, not the full product.

Critical order:

```text
E0-A wow-core
-> E0-B frozen wow-reference fixture
+  E0-C pinned wow-emmy adapter
-> E0-D minimal wow-project generation
-> E0-E bounded wow-rules diagnostics
-> E0-F wow-service + apps/wow
```

Required evidence:

- activate only E0 Cargo workspace members and reviewed dependency edges;
- implement canonical identities, profiles, generations, evidence, coverage, conflicts, results, errors and serialization in `wow-core`;
- freeze one exact ReferenceView fixture and checksums;
- pin one upstream EmmyLua implementation behind the documented adapter and pass compatibility probes;
- build one exact minimal project generation from frozen source/config fixtures without executing repository code;
- execute E0 diagnostics for clean, finding, partial, conflict, `NotEvaluated`, malformed input, cancellation and deterministic repeat;
- expose `wow status` and `wow check` through one service call per command;
- pass canonical byte, exit-code, profile-isolation, resource-bound and selected-platform path tests;
- populate every required E0 implementation/profile/fixture/checksum value.

R0 does not require E1–E7 implementation. It is the earliest runnable project state.

## Gate A0 — useful internal alpha

**Goal:** analyze a real addon repository against one exact WoW profile with trustworthy local results.

Required packages:

```text
E1 Reference Pack build/validation
E2 graph + recognizers + project indexing + ProjectStore
E3 Blizzard UI source + Project Map/L0/L1/context
```

Required evidence:

- build/read back a real immutable Reference Pack from pinned inputs;
- implement bounded nonexecuting TOC/XML/load/Lua source indexing;
- publish one real addon project generation and a separate Blizzard UI source generation;
- run EmmyLua, declarative recognizers, graph publication, diagnostics and bounded context;
- prove clean negative authority only for complete exact partitions;
- test malformed/untrusted repositories, load variants, partial source, profile mismatch, unknown Secret/restriction fields, cancellation, recovery, retention and deterministic rebuilds;
- recheck patch-sensitive claims through the current `wow-addon-engineering-kb`, pinned Blizzard source and required runtime probes before declaring profile support.

A0 may remain one-shot CLI-only.

## Gate A1 — developer preview

**Goal:** make the internal alpha efficient and safe for repeated real development work.

Required packages:

```text
E4 exact-generation search/lineage/migration/static impact
implemented minimal E7-A frontend profile
```

At least one complete frontend beyond one-shot CLI is required:

```text
LSP 3.18 stdio
or
MCP 2025-11-25 stdio
```

The local daemon and local MCP HTTP profile may remain disabled. Disabled capabilities are not advertised.

Required evidence:

- exact/alias/deprecation/replacement lanes precede fuzzy/semantic candidates;
- search explanations expose lanes, scores, coverage and Candidate ceilings;
- context/search continuations remain exact-generation-bound;
- immutable operation registry maps one frontend request to one service operation;
- explicit workspace/project/profile registration and exact overlay versions;
- UTF-8/UTF-16 position, incremental sync, stale-version resynchronization and unsaved-source privacy tests for LSP when enabled;
- fixed read-only tools/resources, strict structured output and no generic/effecting/model-controlled tool path for MCP when enabled;
- cancellation, disconnect, reconnect, broken pipe, response loss, concurrency, backpressure, output limits, cross-client isolation and session closure tests;
- real addon task evaluation shows measurable benefit over manual repository reading;
- preview compatibility manifest names exact supported platform/client/profile and known gaps.

A1 is the first sensible external developer preview. It is not stable v1.

## Gate B0 — governed beta

**Goal:** enable safe recognizer evolution and optional external semantic candidates.

### E5 requirements

Before automated calibration/core-pack lifecycle:

- admit real licensed corpora with provenance groups and leakage-safe splits;
- obtain independent labels/reviews and sealed-holdout evidence;
- run real mutations, per-case metrics, graph/security/determinism tests and measured thresholds;
- build, attest, sign, publish inactive, read back, canary, roll out, designate LKG, and prove rollback/stale-partition closure;
- preserve authorization, response-loss, retention, audit, license, privacy and nonclaim boundaries.

E5 is not required for the frozen E0 diagnostic bootstrap. It is required before claiming safe automated core-pack promotion/rollout.

### E6 requirements

The external provider lane is optional and may ship disabled. Before enabling it:

- implement/probe at least one reviewed E6-A adapter;
- implement E6-B exact configuration/session authorization, durable query/result catalogs, project/reference mapping, explicit selection and exact-root context handoff;
- prove Candidate authority, zero-result honesty, provider-local scoring, optional degradation, no hidden fallback and no credential/source leakage;
- demonstrate net task benefit after verification cost;
- include exact provider adapter/version/state compatibility in the preview/beta manifest.

E6 unavailability never blocks exact local operation.

## Gate V1-RC — release candidate

**Goal:** produce an exact candidate artifact set that can be installed and tested but is not yet the supported public release.

Requires implemented selected product scope plus E7-A and E7-B build/package contracts:

- freeze supported OS/architecture/client/WoW/profile matrix;
- freeze Rust toolchain/dependencies and reproducible release profile;
- build exact binary/data packs and release manifests;
- generate checksums, SBOM, provenance and detached signatures/attestations under secret-isolated adapters;
- create portable/package/installer candidate according to the selected platform profiles;
- validate install, first run, upgrade, rollback, uninstall and data retention;
- run end-to-end supported-client tests and real addon corpus evaluations;
- prove no development-only/internal files are in end-user packages;
- run corruption, interruption, response-loss, stale generation, downgrade, revocation and recovery tests;
- publish no public stable channel until V1 acceptance.

## Gate V1 — public supported release

**Goal:** a reproducible, installable, supportable public product.

Required release evidence:

- all V1-RC gates pass on every claimed supported platform/profile;
- exact public artifacts, SHA-256 manifest, signatures/attestations, SBOM, license/notices and verification instructions;
- secure configuration/credential setup without secrets in repository/examples/logs;
- documented binary/data/config/cache/log/crash locations and backup/restore/retention/GC;
- immutable Reference Pack/core-pack/provider-adapter compatibility manifests;
- explicit release/update channel policy, verified update metadata and rollback/retirement behavior;
- end-to-end tests for corrupted stores, interrupted install/update, response loss, cancellation, stale generations and rollback;
- user docs for install, project registration, profile selection, status/check, diagnostics/search/context, LSP/MCP setup, troubleshooting and removal;
- performance/resource budgets measured on representative small/medium/large addons;
- current target-client/profile validation using the external KB route, pinned Blizzard source and required runtime probes;
- no unresolved release-blocking security, data-loss, profile-mixing, false-negative-authority, credential-leak, cross-client-isolation, update-integrity or rollback issue;
- support owner, compatibility window, incident process and retirement criteria.

CI/release automation may be enabled only when it runs the real frozen commands, has an explicit owner and protects signing/update secrets. Decorative workflows remain forbidden.

## Critical path

```text
First runnable:
    implement E0-A -> E0-F

Useful internal alpha:
    R0 + E1 + E2 + E3

Developer preview:
    A0 + E4 + one complete minimal E7-A frontend

Governed beta:
    A1 + E5; optionally E6

Release candidate:
    selected beta scope + complete implemented E7-A + E7-B candidate build

Public supported v1:
    V1-RC + public release/update/support gates
```

## Current blockers

```text
no Cargo workspace or Rust source
no implemented owner ports or protocol adapters
no frozen implementation commits or populated checksum manifests
no real Reference Pack/project generation/analyzer/diagnostic/search/context execution
no admitted calibration corpus or published core pack
no live external provider adapter/mapping/context evidence
no live LSP/MCP/daemon client/platform tests
no reproducible/package/install/update pipeline
no packaged, signed or attested release artifact
no current-client release validation
```

After E7-B documentation is frozen, additional architecture work does not advance launch readiness. The next step is E0-A implementation.