# Launch gates

**Status:** normative operational plan.

This document separates states that must not be conflated:

```text
documented architecture
implemented package
first runnable executable
useful internal alpha
developer preview
governed beta
release candidate
public supported v1
```

A completed documentation package is not implemented software. A compiling binary is not a useful alpha. A working LSP or MCP transport is not a supported public release.

## Current state

```text
planned architecture/documentation: complete through E7-B
implementation frontier: not-started
next owned package: I0-A / wow-core E0-A
Cargo workspace: absent
Cargo.lock and rust-toolchain: absent
Rust source: absent
CI/workflows: absent
real end-to-end implementation evidence: absent
supported targets and public releases: none
```

E7-A completes the canonical frontend operation-registry, session, overlay, daemon, LSP, and MCP contracts. E7-B completes the source/build/reproducibility/evidence/signing/bundle/channel/install/update/rollback/support contracts. Implementation begins with I0-A, not another documentation package.

## Gate R0 — first runnable bootstrap

**Goal:** one deterministic local executable path, not the full product.

Critical order:

```text
I0-A / E0-A wow-core
-> I0-B / E0-B frozen wow-reference fixture
+  I0-C / E0-C pinned wow-emmy adapter
-> I0-D / E0-D minimal wow-project generation
-> I0-E / E0-E bounded wow-rules diagnostics
-> I0-F / E0-F wow-service + apps/wow
```

Required evidence:

- activate only the implemented E0 Cargo workspace members and reviewed dependency edges;
- implement canonical identities, profiles, generations, evidence, coverage, conflicts, results, errors, cancellation, budgets, and serialization in `wow-core`;
- freeze one exact ReferenceView fixture and checksums;
- pin one upstream EmmyLua implementation behind the documented adapter and pass compatibility probes;
- build one exact minimal project generation from frozen source and configuration fixtures without executing repository code;
- execute E0 diagnostics for clean, finding, partial, conflict, `NotEvaluated`, malformed input, cancellation, and deterministic repeat;
- expose `wow status` and `wow check` through one service call per command;
- pass canonical byte, exit-code, profile-isolation, resource-bound, broken-pipe, and selected-platform path tests;
- populate every required E0 implementation, profile, fixture, and checksum value.

R0 does not require E1 through E7 implementation. It is the earliest runnable project state.

**Current state:** blocked; every required implementation item is absent.

## Gate A0 — useful internal alpha

**Goal:** analyze a real addon repository against one exact WoW profile with trustworthy local results.

Required packages:

```text
R0
+ E1 Reference Pack build and validation
+ E2 graph, recognizers, project indexing, and ProjectStore
+ E3 Blizzard UI source, Project Map, L0/L1/L2, and context
```

Required evidence:

- build, read back, and reproduce a real immutable Reference Pack from pinned inputs;
- implement bounded nonexecuting TOC, XML, load, and Lua source indexing;
- publish one real addon project generation and a separate Blizzard UI source generation;
- run EmmyLua, declarative recognizers, graph publication, diagnostics, and bounded context;
- prove clean negative authority only for complete exact partitions;
- test malformed and untrusted repositories, load variants, partial source, profile mismatch, unknown restriction fields, cancellation, recovery, retention, and deterministic rebuilds;
- recheck patch-sensitive claims through the currently selected Blizzard source and required runtime probes before declaring profile support.

A0 may remain one-shot CLI-only.

**Current state:** blocked by R0 and all E1 through E3 implementation.

## Gate A1 — developer preview

**Goal:** make the internal alpha efficient and safe for repeated real development work.

Required packages:

```text
A0
+ E4 exact-generation search, lineage, migration, and static impact
+ one complete implemented E7-A frontend profile
```

At least one complete frontend beyond one-shot CLI is required:

```text
LSP 3.18 stdio
or
MCP 2025-11-25 stdio
```

The local daemon and optional local MCP HTTP profile may remain disabled. Disabled capabilities are not advertised.

Required evidence:

- exact, alias, deprecation, replacement, and transition lanes precede fuzzy or semantic candidates;
- search explanations expose lanes, scores, coverage, and Candidate ceilings;
- context and search continuations remain exact-generation-bound;
- immutable operation registry maps one frontend request to one service operation;
- explicit workspace, project, and profile registration with exact overlay versions;
- UTF-8 and UTF-16 positions, incremental synchronization, full-document replacement, stale-version resynchronization, and unsaved-source privacy tests for LSP when enabled;
- fixed read-only tools and resources, strict structured output, and no generic, effecting, or model-controlled tool path for MCP when enabled;
- cancellation, disconnect, reconnect, broken pipe, response loss, concurrency, backpressure, output limits, cross-client isolation, and session closure tests;
- real addon task evaluation shows measurable benefit over manual repository reading;
- preview compatibility manifest names exact supported platforms, clients, profiles, and known gaps.

A1 is the first sensible external developer preview. It is not stable v1.

**Current state:** blocked by A0, E4, and E7-A implementation.

## Gate B0 — governed beta

**Goal:** enable safe recognizer evolution and optional external semantic candidates.

### E5 requirements

Before automated calibration and core-pack lifecycle:

- admit real licensed corpora with provenance groups and leakage-safe splits;
- obtain independent labels, reviews, and sealed-holdout evidence;
- run real mutations, per-case metrics, graph, security, and determinism tests with measured thresholds;
- build, attest, sign, publish inactive, read back, canary, roll out, designate LKG, and prove rollback and stale-partition closure;
- preserve authorization, response loss, retention, audit, license, privacy, and nonclaim boundaries.

E5 is not required for the frozen E0 diagnostic bootstrap. It is required before claiming safe automated core-pack promotion and rollout.

### E6 requirements

The external provider lane is optional and may ship disabled. Before enabling it:

- implement and probe at least one reviewed E6-A adapter;
- implement E6-B exact configuration and session authorization, durable query/result catalogs, project/reference mapping, explicit selection, and exact-root context handoff;
- prove Candidate authority, zero-result honesty, provider-local scoring, optional degradation, no hidden fallback, and no credential or source leakage;
- demonstrate net task benefit after verification cost;
- include exact provider adapter, version, state, and compatibility in the preview or beta manifest.

E6 unavailability never blocks exact local operation.

**Current state:** blocked by A1 and selected E5 or E6 implementation evidence.

## Gate V1-RC — release candidate

**Goal:** produce an exact candidate artifact set that can be installed and tested but is not yet the supported public release.

Requires implemented selected product scope plus complete E7-A host and E7-B release services:

- freeze the supported OS, architecture, client, WoW, and profile matrix;
- freeze Rust toolchain, dependencies, build scripts, materialization, and reproducible release profile;
- produce at least two independent matching unsigned builds for every reproducibility claim;
- build exact binary, data, configuration, schema, and release manifests;
- generate required checksums, SBOM, provenance, licenses, notices, test reports, and detached signatures or attestations through secret-isolated adapters;
- create the deterministic portable bundle and any selected installer package;
- validate install, first run, upgrade, data migration, rollback, uninstall, and data retention;
- run supported-client tests and admitted real-addon evaluations;
- prove no development-only or internal files are in end-user packages unless the exact bundle manifest requires them;
- run corruption, interruption, response-loss, stale-generation, downgrade, revocation, and recovery tests;
- keep public stable publication disabled until V1 acceptance.

**Current state:** blocked; no implementation, reproducible build, signature, bundle, installer, or platform evidence exists.

## Gate V1 — public supported release

**Goal:** a reproducible, installable, updateable, rollback-capable, and supportable public product.

Initial target intent:

```text
x86_64-pc-windows-msvc
Windows x86-64 exact support profile
one-shot CLI
local named-pipe daemon
LSP 3.18 stdio
MCP 2025-11-25 stdio
explicit release verification and update client
```

This is an intent, not current support.

Required release evidence:

- all V1-RC gates pass for every claimed supported platform and profile;
- exact public artifacts, SHA-256 manifest, signatures and attestations, SBOM, license and notices, and verification instructions;
- secure configuration and credential setup without secrets in the repository, examples, logs, or normal command line;
- documented binary, data, configuration, cache, log, crash, backup, restore, retention, and GC locations;
- immutable Reference Pack, core-pack, and provider-adapter compatibility manifests;
- provider-neutral immutable publication, public read-back, signed release and update manifests, and guarded channel expected-current CAS;
- no asset mutation under the same release identity and no latest or tag selection shortcut;
- clean supported Windows installation with exact staging, path, ACL, lock, helper, and self-check behavior;
- registered store and configuration migrations with verified backup, crash recovery, and rollback compatibility;
- explicit update check, plan, materialization, verification, apply, reconciliation, and rollback from one exact prior candidate;
- exact LastKnownRunnable designation and retention of a compatible rollback target;
- end-to-end tests for corrupted stores, interrupted install or update, response loss, cancellation, stale state, helper failure, migration failure, and rollback;
- user documentation for install, project registration, profile selection, status and check, diagnostics, search, context, LSP and MCP setup, update, rollback, troubleshooting, and removal;
- performance and resource budgets measured on representative small, medium, and large addons;
- current target-client and profile validation using the external KB route, pinned Blizzard source, and required runtime probes;
- no unresolved release-blocking security, data-loss, profile-mixing, false-negative-authority, credential-leak, cross-client-isolation, update-integrity, migration, or rollback issue;
- support owner, compatibility window, incident process, revocation behavior, and retirement criteria.

CI and release automation may be enabled only when they invoke the real frozen commands, have an explicit owner, and protect signing and update secrets. Decorative workflows remain forbidden.

**Current state:** blocked; no supported target, candidate, public artifact, installation, update, or support evidence exists.

## Critical path

```text
First runnable:
    I0-A -> I0-F

Useful internal alpha:
    R0 + I1 + I2 + I3

Developer preview:
    A0 + I4 + one complete E7-A frontend

Governed beta:
    A1 + I5; optionally I6

Release candidate:
    selected beta scope + complete I7-A + candidate I7-B pipeline

Public supported v1:
    V1-RC + public publication, install, update, rollback, and support gates
```

## Current blockers

```text
no Cargo workspace, lockfile, Rust toolchain, or Rust source
no implemented owner ports, service operations, or protocol adapters
no frozen implementation commits or populated implementation checksums
no real Reference Pack, project generation, analyzer, diagnostic, search, or context execution
no admitted calibration corpus or published core pack
no live external provider adapter, mapping, or context evidence
no live LSP, MCP, daemon, client, or platform tests
no independent reproducible build or release pipeline
no packaged, signed, or attested public artifact
no install, migration, update, reconciliation, or rollback rehearsal
no current-client release validation
```

The architecture documentation is complete through E7-B. Additional architecture text does not advance launch readiness. The next step is I0-A `wow-core` implementation.
