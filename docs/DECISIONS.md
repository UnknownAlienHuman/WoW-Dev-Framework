# Architecture decisions

**Status:** normative.

A decision remains accepted until a later ADR explicitly supersedes it and records compatibility and migration consequences. Package-local decisions may narrow these rules but cannot silently contradict them.

## Status vocabulary

- **Accepted** — implementation must conform.
- **Proposed** — requires an explicit owner, experiment, fixtures, and review.
- **Superseded** — retained for history; the replacement controls.
- **Rejected** — outside the current design.

## Accepted decisions

### ADR-001 — Use upstream EmmyLua behind one adapter

Use an updateable upstream Rust analysis dependency behind the adapter. Record the exact dependency selected for each tested build; no revision is permanently current. Do not maintain a default fork or a second correctness-path Lua parser.

### ADR-002 — One host merges generic and WoW diagnostics

`wow-emmy` exposes coherent analyzer inputs and results; `wow-rules` owns WoW diagnostic algorithms; `wow-service` coordinates; frontends remain thin.

### ADR-003 — Upstream registration gaps are isolated by our adapter

Product delivery does not depend on an upstream provider change. The adapter remains narrow, replaceable, and compatibility-probed.

### ADR-004 — Ketho is the primary Rust annotation-service donor

Revised by explicit owner direction on 2026-09-05. Port Ketho/vscode-wow-api's
source-loading, normalization/correction, type-lowering, annotation-emission and
editor-independent consumer behavior into Rust. Ketho is an implementation
baseline as well as the semantic/output parity baseline, not only an oracle.
The module map and migration order are in [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md).
Owner direction requires zero Python code or interpreter invocation in the
repository. The old source producers and interpreter-driven tests are retired,
not kept as a fallback. Native source generation and Rust-only CI are mandatory.
Existing wire importers remain compatibility readers; retired functionality is
explicitly listed in the implementation ledger rather than described as ported.
Current Gethe data, not a historical Ketho snapshot, defines current Blizzard
facts. Preserve required third-party notices; do not copy source execution or
editor-setting/diagnostic-suppression policies as hidden product behavior.

The pure renderer may be ported/tested in wow-annotations before the persistent
E1 ReferenceView adapter exists. This activates that executable slice only; it
does not complete E1-C, infer reference coverage, or bypass the later source-map,
loss and EmmyLua/LuaLS consumer-probe contracts.

### ADR-005 — Numy is a differential oracle

Use Numy annotations and corpus conventions for comparison. Canonical FrameXML facts still come from structural owner analysis.

### ADR-006 — Raw metadata and annotation projections are separate

Retain all raw Blizzard fields and unknowns independently of generated annotation formats.

### ADR-007 — Blizzard content is canonical; acquisition provider is provenance

The exact materialized source snapshot and digest define the input. Gethe or another mirror records acquisition provenance only.

### ADR-008 — One exact active profile per project generation

Historical, PTR, beta, regional, and flavor data remain separate. No mixed-profile result.

### ADR-009 — WoW ownership is multi-axis

Lexical, ownership, load, object, inheritance, registration, lifecycle, state, call, and lineage relations remain distinct; there is no universal parent.

### ADR-010 — Production recognizers emit universal structural roles

Rules cannot branch on repository, addon, owner, path, popularity, label, split, reviewer, holdout, canary, provider, or model identity.

### ADR-011 — EmmyLua is the sole correctness-path Lua parser

Recognizers and analyzer-dependent rules consume canonical Emmy syntax and semantic facts.

### ADR-012 — External Codebase Memory is optional and isolated

External discovery uses reviewed typed E6 contracts only and can fail without lowering exact local, reference, or project capability.

### ADR-013 — Own a small exact WoW sidecar graph

Use domain-specific SQLite graph records until measured need and a supported external ABI justify another substrate.

### ADR-014 — Structured evidence ranks before similarity

Exact identity, explicit aliases, deprecations, replacements, transitions, lineage, and compatible shape evidence precede fuzzy, text, and semantic Candidates.

### ADR-015 — SQLite is the baseline local storage, search, and graph substrate

Use B-tree indexes, FTS5, adjacency tables, WAL, immutable objects, and bounded in-memory projections. No service proliferation without measured need.

### ADR-016 — Restriction facets are open

Preserve unknown restriction metadata raw and make dependent rules `NotEvaluated` rather than assuming safe.

### ADR-017 — Community addons are examples, not platform authority

They provide implementation and structural evidence. Patch-sensitive contracts are validated against exact Blizzard, Reference, and runtime evidence.

### ADR-018 — External source acquisition is explicit and non-vendored

A later exact source-owner contract may materialize a pinned external revision. E6 does not clone, follow provider locators, or redistribute full repositories.

### ADR-019 — Skeleton-first agent reads

Use Project Map, L0, and L1 before bounded L2 or full source.

### ADR-020 — Project architecture memory is generated

Generate compact project maps from exact project and graph generations instead of manual personal knowledge cards.

### ADR-021 — Framework code targets MIT and public development

Third-party, provider, addon, and Blizzard artifacts retain separate provenance, license, notice, privacy, and redistribution decisions.

### ADR-022 — No default component without unique responsibility or measured benefit

Architecture does not grow from fashion, provider marketing, or arbitrary crate counts.

### ADR-023 — External provider authority has a hard Candidate ceiling

Every E6 result is `semantic_candidate + Candidate`, `negative_authority=unavailable`, regardless of labels, rank, score, repetition, stable state, or zero result.

### ADR-024 — External locator mapping belongs to exact source owners

Only `wow-project` or `wow-reference` maps a bounded owner-neutral locator projection. `ExactMapped` proves locator identity only.

### ADR-025 — External candidate selection is explicit and non-authoritative

The caller supplies exact candidate and mapping IDs plus `Selected`, `Rejected`, or `Deferred`. Selection is not verification, edit authorization, lineage, replacement, impact, or promotion.

### ADR-026 — External provider metadata remains a sidecar

Exact context comes from normal owner roots. Provider labels, scores, snippets, summaries, and traces remain outside `ContextSemanticPack` truth.

### ADR-027 — External provider and session effects are durable and secret-isolated

E6-B uses exact nonsecret configuration and authorization references plus host-owned narrow sessions. Response loss becomes `OutcomeUnknown`.

### ADR-028 — Launch states are separate gates

Documented architecture, implemented package, first runnable, useful alpha, developer preview, governed beta, release candidate, and public v1 are distinct states.

### ADR-029 — Frontends and internal clients are service-only

CLI, daemon, LSP, MCP, Reference builder, update client, and release tool depend on `wow-service` only among framework crates. One semantic request maps to one service operation unless the composite is itself a documented service operation.

### ADR-030 — Close-before-success and exact response-loss recovery

Every effect uses exact operation and request identity. Required validation, read-back, retention, audit, and reverse-order closure precede public success.

### ADR-031 — One immutable frontend operation registry

Visible CLI, daemon, LSP, and MCP semantic operations come from a reviewed content-addressed registry bound to exact service schemas, implementation capabilities, effect and authorization class, privacy, license, and resource policy.

### ADR-032 — One `wow` binary hosts the initial product frontends

One executable may provide one-shot CLI, foreground local daemon, LSP stdio, MCP stdio, optional explicit local MCP HTTP, and the release-verification and update client. Separate semantic frontend crates are not required without measured packaging or isolation need.

### ADR-033 — Initial protocol versions are pinned

Initial compatibility profiles bind LSP 3.18, MCP revision 2025-11-25, and `wow-local-jsonrpc/1`. Protocol upgrades create new reviewed profiles and fixtures.

### ADR-034 — Workspaces are explicit and unsaved documents are immutable project-owned overlays

A client explicitly registers workspace, project, and profile state. Every unsaved document version is an immutable UTF-8 overlay snapshot owned by `wow-project`.

LSP 3.18 uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Stale, skipped, repeated-with-different-content, or out-of-order versions require explicit resynchronization.

### ADR-035 — Transport positions are projections, not owner coordinates

Owner source ranges use canonical UTF-8 byte offsets. LSP UTF-16 or UTF-8 positions are converted only against the exact negotiated overlay line index.

### ADR-036 — Initial MCP surface is fixed and read-only

MCP 2025-11-25 initially exposes fixed implemented read-only tools and exact resources. Prompts, sampling, elicitation, tasks, server-requested roots, arbitrary tools, provider effects, governance effects, source mutation, and release effects are absent.

### ADR-037 — Local hosting is the default security posture

The daemon uses current-user named pipe or Unix-domain socket. Optional MCP HTTP is explicit, loopback-only, authenticated, Origin-validated, and disabled by default. No remote listener is supported in the E7-A baseline.

### ADR-038 — Disconnect, cancellation, progress, and delivery are independent

Disconnect does not cancel, progress does not complete, and response delivery state does not redefine service operation state. Retained responses may be replayed without reexecution.

### ADR-039 — Multi-client isolation includes unsaved source

Sessions independently scope workspaces, overlays, authorization, provider access, operations, results, progress, streams, and response journals. Unsaved source is private and memory-only by default.

### ADR-040 — E7-A transport readiness is not release readiness

Executable protocols do not establish reproducible packaging, installation and update integrity, compatibility support, or public distribution. Those are E7-B gates.

### ADR-041 — A compiled binary is not a release

Source closure, build execution, reproducibility, tests and evidence, signatures, bundle, support matrix, channel publication, installation, update, rollback, and runtime correctness remain independent states.

### ADR-042 — Unsigned semantic artifacts define reproducible build identity

At least two independent exact unsigned builds are compared before target-specific signing or notarization. Platform signature variance cannot redefine unsigned source-build identity.

### ADR-043 — Release builds use a typed narrow executor

`wow-service` supplies a frozen build plan over verified source, dependency, and toolchain inputs. No arbitrary shell, Cargo, rustc, linker, environment, HTTP, SQL, provider API, or callback surface crosses the release seam.

### ADR-044 — Supply-chain evidence is first-class

SBOM, provenance, licenses and notices, checksums, public schemas, compatibility registry, test, security, and benchmark reports, signatures, and verification instructions are immutable release artifacts.

### ADR-045 — One public binary has separate data-pack lifecycles

The default public product contains one `wow` executable. Reference Packs, core recognizer packs, provider adapters, and other data artifacts remain separately identified, signed, compatible, and updateable. Offline bundles must name exact members.

### ADR-046 — Windows x86-64 MSVC is the first target intent

No target is advertised as supported until its exact build, IPC, filesystem and ACL, console, client, signing, installation-helper, migration, update, rollback, clean-machine, and real-addon suites pass.

### ADR-047 — GitHub Releases is a provider adapter, not architecture, authorization, or trust

Distribution contracts are provider-neutral. Repository, tag, account, CI identity, release status, successful upload, TLS, and asset name do not authorize publication or authenticate content.

### ADR-048 — Release assets and channel records are immutable by exact digest

No in-place archive replacement. Channel changes use exact expected-current compare-and-swap to a signed release and update manifest, followed by public read-back.

### ADR-049 — Updates are explicit and staged by default

No hidden startup check, download, install, telemetry, crash upload, or remote configuration. Check, materialize, verify, stage, backup, migrate, activate, self-check, designate LKR, clean up, and roll back are separate effects.

### ADR-050 — Running Windows binary replacement uses an exact verified helper or installation-owner plan

The public app never overwrites itself or constructs arbitrary helper commands, paths, or URLs. Helper handoff and every partial effect are durable and reconcilable.

### ADR-051 — Store and configuration migrations are registered owner effects

No raw SQL or arbitrary scripts in service, app, or release tool. Forward and rollback compatibility, verified backup and restore, crash recovery, and read-back are required.

### ADR-052 — LastKnownRunnable installation is explicit

Rollback targets are exact retained qualified installation records, never inferred from previous, newest, version, filename, directory, or current pointer position.

### ADR-053 — Revocation, retirement, and incidents are distinct

Revocation addresses unsafe or ineligible artifacts, manifests, releases, keys, targets, or profiles. Retirement ends ordinary support or channel eligibility. Incidents preserve affected scope, evidence, uncertainty, containment, remediation, and public advisory state. None rewrites history.

### ADR-054 — Support claims are exact matrix claims

Support is limited to tested target, OS, architecture, runtime, path, IPC, protocol, store, schema, data-pack, WoW-profile, client, feature, resource, installation, update, and rollback combinations. Portability intent is not support.

### ADR-055 — CI invokes real tested commands and owns no semantics

CI is added only after commands exist and pass locally. It cannot hide skipped gates, reimplement release logic in YAML or shell, access secrets through public payloads, or publish merely because tests pass.

### ADR-056 — Architecture planning is complete through E7-B

Follow the current implementation ledger and the Ketho Rust port route, not the historical I0-A bootstrap instruction. New architecture requires a concrete implementation or test failure, the smallest proposed seam or ADR change, and compatibility, security, privacy, license, supply-chain, and evidence analysis.

## Corrected earlier assumptions

```text
Gethe-first
-> exact Blizzard content and digest first; acquisition provider is provenance.

external Emmy process only
-> exact upstream Rust implementation behind one adapter.

ast-grep recognizers
-> declarative recognizers over canonical Emmy facts.

Codebase Memory-only graph
-> optional external Candidate discovery plus exact framework-owned graph.

provider result -> source handle
-> UnverifiedProviderLocator -> owner mapping -> explicit caller selection.

provider and session lifecycle inside wow-cbm
-> E6-A pure Candidate owner, E6-B service orchestration, separate host adapters.

LSP and MCP as generic tool gateways
-> static service-operation mappings with no arbitrary invoke.

mutable editor document inside service
-> immutable project-owned overlay snapshots and exact session state.

full-document-only LSP baseline
-> canonical LSP 3.18 profile uses incremental changes and accepts full-document replacement exactly.

one successful build -> reproducible release
-> at least two independent unsigned builds plus complete release evidence.

GitHub tag or release -> trusted release
-> provider-neutral exact bundle, signatures, public read-back, and channel CAS.

self-update by overwriting wow.exe
-> staged exact installation-owner helper, migration, self-check, LKR, and rollback.

all documentation required before first run
-> E0-A through E0-F is R0; later packages layer on top.
```

## Rejected directions

- artificial microcrate proliferation without an owned reusable responsibility;
- custom graph, search, or vector servers in v1 without benchmarks;
- repository or addon-specific production recognizer rules;
- editor settings mutation as correctness;
- direct external provider database writes;
- generic MCP, tool, RPC, shell, script, plugin, model, process, SQL, provider API, or installer escape hatches;
- provider labels, scores, or zero results as authority;
- service or application-side source mapping or implicit top or sole selection;
- provider metadata inside exact context truth;
- provider failure disabling exact local workflows;
- default remote daemon or MCP listener;
- automatic source-edit application or execute-command in the baseline;
- one successful build, CI badge, tag, upload, checksum, signature, or installation as release readiness;
- in-place release asset replacement;
- hidden update, telemetry, crash upload, or remote configuration;
- raw installer and migration scripts or ad hoc self-overwrite;
- previous or newest installation inferred as LKR or rollback target;
- decorative CI and release automation before real commands and evidence.

## Proposed decisions

Future candidates remain in [`IDEAS.md`](IDEAS.md). After architecture freeze, a proposal requires a concrete implementation-discovered need, explicit owner, experiment, fixtures, security, privacy, license, and supply-chain analysis, and migration consequences.
