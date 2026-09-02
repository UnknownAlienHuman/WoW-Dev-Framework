# Architecture decisions

**Status:** normative.

This register converts accepted architecture choices into stable repository policy. A decision remains accepted until a later ADR explicitly supersedes it and records migration consequences. Detailed package decisions may narrow these rules but cannot silently contradict them.

## Status vocabulary

- **Accepted** — implementation must conform.
- **Proposed** — requires review and an acceptance experiment.
- **Superseded** — retained for history; the replacement decision controls.
- **Rejected** — explicitly outside the current design.

## Accepted decisions

### ADR-001 — Upstream EmmyLua dependency, not a fork

**Decision:** use the upstream Rust analysis library pinned behind one adapter.

**Consequence:** updates are compatibility-probed and rollback does not require maintaining a permanent fork.

### ADR-002 — One host merges generic and WoW diagnostics

**Decision:** `wow-emmy` exposes coherent generic analyzer inputs/results; `wow-rules` owns WoW diagnostic algorithms; `wow-service` coordinates; frontends remain thin.

**Consequence:** source spans, profile identity, capability state and output ordering remain coherent without frontend analyzer forks.

### ADR-003 — Upstream provider gaps are isolated by our host

**Decision:** missing upstream registration is handled behind a reviewed adapter. Upstream changes are optional, not delivery blockers.

**Consequence:** the integration remains replaceable and probe-gated.

### ADR-004 — Ketho is a compatibility oracle

**Decision:** use Ketho annotation behavior, field corrections and output semantics for parity fixtures; do not copy editor-setting mutation policy.

**Consequence:** familiar annotations without editor-dependent correctness.

### ADR-005 — Numy is a differential oracle

**Decision:** use Numy FramexmlAnnotations for comparison/corpus conventions; canonical FrameXML facts come from structural source analysis.

**Consequence:** disagreements become explicit coverage/triage records.

### ADR-006 — Raw metadata and annotations are separate projections

**Decision:** retain all raw Blizzard metadata and unknown fields independently of generated annotations.

**Consequence:** restriction/future fields are not lost when an annotation format cannot express them.

### ADR-007 — Blizzard content is canonical; acquisition provider is provenance

**Decision:** Gethe or another mirror may acquire a snapshot, but exact materialized Blizzard content/digest defines the platform input.

**Consequence:** acquisition remains replaceable while content identity stays stable.

### ADR-008 — One active profile per project generation

**Decision:** diagnostics use one selected exact profile. Historical, PTR, beta, regional and flavor data remain separate.

**Consequence:** no cross-profile availability/signature/restriction leakage.

### ADR-009 — WoW ownership is multi-axis

**Decision:** store lexical, ownership, load, object, inheritance, registration, lifecycle, state, call and lineage relations independently.

**Consequence:** queries expose explicit chains instead of one ambiguous parent.

### ADR-010 — Recognizers emit universal roles

**Decision:** production packs are declarative structural rules and do not branch on addon/repository/owner/path/popularity identity.

**Consequence:** calibration improves universal coverage without product-specific matcher modes.

### ADR-011 — EmmyLua is the sole correctness-path Lua parser

**Decision:** recognizers and analyzer-dependent rules consume canonical Emmy syntax/semantic facts.

**Consequence:** no dialect/span/parser disagreement on the correctness path.

### ADR-012 — External Codebase Memory is optional and isolated

**Decision:** external discovery uses reviewed typed E6 transport contracts only. Framework code never writes provider databases or exposes a generic provider/MCP/tool proxy.

**Consequence:** broad discovery can fail without degrading exact local capability.

### ADR-013 — Own a small exact WoW sidecar graph

**Decision:** store TOC/XML/load/API/UI/state/restriction/lineage facts in a domain-specific SQLite graph until measured need justifies another substrate.

**Consequence:** exact correctness does not depend on a generic external graph schema.

### ADR-014 — Structured evidence ranks before similarity

**Decision:** exact identity and explicit alias/deprecation/replacement/transition/lineage/shape evidence precede fuzzy, text and semantic similarity.

**Consequence:** similarity retrieves Candidates but cannot prove migration, impact, absence or authorize a fix.

### ADR-015 — SQLite is the initial persistence/search/graph substrate

**Decision:** use SQLite B-tree indexes, FTS5, adjacency tables, WAL for mutable owner state, immutable objects and bounded memory projections.

**Consequence:** no graph/search/vector server in v1 without measured unique need.

### ADR-016 — Restriction facets are open

**Decision:** preserve unknown facets raw and make dependent rules `NotEvaluated`.

**Consequence:** new Blizzard fields degrade honestly rather than appearing safe.

### ADR-017 — Community addons are examples, not platform authority

**Decision:** third-party implementations provide structural/implementation evidence only.

**Consequence:** patch-sensitive contracts are revalidated against pinned Blizzard source and runtime evidence.

### ADR-018 — External source acquisition is explicit and non-vendored

**Decision:** a later approved source-owner contract may materialize an exact external revision. E6 does not clone/follow provider locators, own provider index lifecycle or redistribute repositories.

**Consequence:** acquisition, provider lifecycle, locator mapping and candidate querying remain separate effects.

### ADR-019 — Skeleton-first agent reads

**Decision:** agents receive L0/L1 structure before L2/full source.

**Consequence:** smaller context and explicit source access.

### ADR-020 — Project architecture memory is generated

**Decision:** generate a portable Project Map from exact project/graph generations rather than personal/manual memory cards.

**Consequence:** users and agents receive the same repository-derived architecture state.

### ADR-021 — MIT/public release target

**Decision:** framework code/templates target MIT; third-party/Blizzard/provider artifacts retain separate provenance/license/notice/redistribution decisions.

**Consequence:** repository visibility never broadens external rights.

### ADR-022 — No default component without justification

**Decision:** a component enters the default path only for unique correctness responsibility or measured task benefit.

**Consequence:** architecture does not grow from fashion, provider marketing or arbitrary crate counts.

### ADR-023 — External provider authority has a hard Candidate ceiling

**Decision:** every E6 result remains `semantic_candidate + Candidate` with no negative authority regardless of provider labels, rank, score, repetition, stable state or zero result.

**Consequence:** external discovery cannot silently become source/platform/runtime truth.

### ADR-024 — Source locator mapping belongs to exact owners

**Decision:** only `wow-project` or `wow-reference` maps an external locator into one exact retained owner record through an owner-neutral projection.

**Consequence:** service/application cannot inspect paths or choose mappings; `ExactMapped` proves locator identity only.

### ADR-025 — External candidate selection is explicit and non-authoritative

**Decision:** the caller supplies exact candidate/mapping IDs and `Selected`, `Rejected` or `Deferred`; service records but never infers it.

**Consequence:** rank/score/name/position never becomes automatic selection or edit/promotion authority.

### ADR-026 — External provider metadata remains a sidecar

**Decision:** exact context comes from normal owner contracts; provider labels/scores/snippets/summaries/traces remain in `ExternalCandidateSidecar`.

**Consequence:** external metadata cannot contaminate `ContextSemanticPack` truth.

### ADR-027 — External provider/session effects are durable and secret-isolated

**Decision:** E6-B uses exact nonsecret configuration/authorization references, narrow host-owned sessions and durable effect receipts.

**Consequence:** response loss becomes `OutcomeUnknown`; provider integration leaks no credentials/private endpoints/process/database handles into public data.

### ADR-028 — Launch states are separate gates

**Decision:** documented architecture, first runnable, internal alpha, developer preview, governed beta, release candidate and public v1 are separate states in `LAUNCH_GATES.md`.

**Consequence:** later documentation does not block E0 bootstrap and a compiling binary is not mislabeled as supported.

### ADR-029 — Frontends are thin service-only transports

**Decision:** CLI, daemon, LSP, MCP and future frontends depend on `wow-service` only; one semantic transport request maps to one service operation unless the workflow is itself a documented service operation.

**Consequence:** transports cannot change authority, bypass owners or duplicate business logic.

### ADR-030 — Close-before-success and exact response-loss recovery

**Decision:** every durable/external effect uses exact operation/request identity; required retention/audit and reverse-order closure finish before public success.

**Consequence:** timeout/disconnect cannot be treated as no effect and cleanup/retry cannot become unowned background work.

### ADR-031 — One immutable frontend operation registry

**Decision:** visible CLI/daemon/LSP/MCP semantic operations come from a reviewed immutable registry bound to exact service schemas, implementation capabilities, effect/authorization class and privacy/license policy.

**Consequence:** runtime reflection, generic `call_service`, dynamic tool discovery and placeholder capability advertisement are forbidden.

### ADR-032 — One `wow` binary may host initial frontends

**Decision:** one executable may provide one-shot CLI, foreground local daemon, LSP stdio and MCP stdio/local-only HTTP modes.

**Consequence:** no artificial protocol crates/binaries are required until measurement demonstrates independent packaging/isolation needs.

### ADR-033 — Initial protocol versions are pinned

**Decision:** initial compatibility profiles bind LSP 3.18, MCP revision 2025-11-25 and `wow-local-jsonrpc/1`.

**Consequence:** protocol upgrades create new reviewed profiles, schema fixtures and compatibility evidence rather than silently changing behavior.

### ADR-034 — Workspaces are explicit and unsaved documents are immutable overlays

**Decision:** a client explicitly registers workspace/project/profile state; each unsaved document version is a session-scoped project-owned immutable UTF-8 overlay snapshot.

**Consequence:** no cwd/Git/editor/WoW auto-discovery, no in-place project mutation, and stale/out-of-order changes require resynchronization.

### ADR-035 — Transport positions are projections, not owner coordinates

**Decision:** owner source ranges use canonical UTF-8 byte offsets. LSP UTF-16/UTF-8 positions are converted only against the exact negotiated overlay line index.

**Consequence:** editor encoding cannot alter source identity and invalid boundaries fail rather than drift.

### ADR-036 — Initial MCP surface is fixed and read-only

**Decision:** MCP 2025-11-25 initially exposes fixed implemented read-only tools and exact resources over stdio. Prompts, sampling, elicitation, tasks, arbitrary roots, generic tools and effecting default tools are absent.

**Consequence:** model invocation is never user authorization and MCP cannot become a generic agent execution path.

### ADR-037 — Local hosting is the default security posture

**Decision:** daemon uses current-user named pipe/Unix socket; MCP HTTP is explicit loopback-only, Origin-validated, authenticated and disabled by default; no remote listener is supported in E7-A.

**Consequence:** the first frontend layer has no accidental network service or cross-user default exposure.

### ADR-038 — Disconnect, cancellation, progress and delivery are independent

**Decision:** disconnect does not cancel, progress does not complete, and response delivery state does not redefine service operation state. Retained responses may be replayed without reexecution.

**Consequence:** backpressure/reconnect cannot duplicate effects or fabricate success.

### ADR-039 — Multi-client isolation includes unsaved source

**Decision:** sessions independently scope workspaces, overlays, authorization, provider access, operations, results, progress and response journals. Unsaved source is memory-only by default.

**Consequence:** shared daemon/frontends cannot leak one client's private working state into another.

### ADR-040 — E7-A transport readiness is not release readiness

**Decision:** executable protocols do not establish reproducible packaging, install/update integrity, compatibility support or public distribution. Those are E7-B gates.

**Consequence:** a working LSP/MCP binary cannot be released as supported v1 without the release lifecycle evidence.

## Corrected earlier assumptions

```text
Gethe-first
-> exact Blizzard content/digest first; mirror is provenance.

external Emmy process only
-> pinned upstream Rust analyzer behind one adapter.

ast-grep recognizers
-> declarative recognizers over canonical Emmy facts.

Codebase Memory-only graph
-> optional external Candidate discovery plus exact framework graph.

provider result -> stable source handle
-> UnverifiedProviderLocator -> exact owner mapping -> explicit selection.

provider/session/index lifecycle inside wow-cbm
-> E6-A pure candidate owner, E6-B orchestration, later explicit host lifecycle contract.

current-only reference
-> exact active target plus separately configured historical/transition generations.

personal knowledge cards
-> generated Project Map and public rule packs.

all documentation required before first run
-> E0-A through E0-F is the first runnable gate.

separate semantic LSP/MCP implementations
-> one service operation registry and thin protocol projections.

editor buffer mutates project generation
-> immutable session overlay; saved publication is separate.

MCP exposes every service method
-> fixed read-only allow-list under exact registry/profile.

disconnect means operation stopped
-> explicit cancellation and exact reconciliation.
```

## Rejected directions

- dozens of artificial microcrates as an implementation target;
- custom graph/search/vector servers in v1 without benchmarks;
- internal vector database by default;
- hardcoded addon/repository-specific production logic;
- deep interprocedural Secret flow before local rules work;
- editor-setting mutation as correctness mechanism;
- direct provider database writes or provider lifecycle inside E6 semantics;
- generic MCP/tool/RPC/shell/plugin/model escape hatches;
- provider labels/scores/zero results as authority;
- service/application-side locator mapping;
- implicit top/sole/best candidate selection;
- provider metadata inside exact context truth;
- provider failure disabling exact local workflows;
- full Blizzard UI source in default model context;
- one generic graph parent relation;
- workspace/Git/WoW auto-discovery as semantic configuration;
- editor-specific analyzer/diagnostic forks;
- default remote daemon/MCP listener;
- automatic `WorkspaceEdit` application without exact guards;
- prompts/sampling/elicitation/tasks in initial MCP profile;
- response replay that reexecutes service;
- decorative CI/release automation before executable commands and E7-B gates.

## Proposed decisions

Candidate decisions are tracked in [`IDEAS.md`](IDEAS.md). They become accepted only after a documented experiment, explicit owner, migration analysis and ADR update.