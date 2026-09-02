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

**Consequence:** upstream updates are compatibility-probed and can be rolled back without maintaining a permanent analyzer fork.

### ADR-002 — One host merges generic and WoW diagnostics

**Decision:** the `wow-emmy` owner exposes coherent generic Emmy and WoW diagnostic inputs/results for one exact project generation; `wow-service` coordinates use cases and transports remain thin.

**Consequence:** source spans, profile identity, capability state, and output ordering remain coherent without duplicating analyzer state across frontends.

### ADR-003 — External provider gap is handled by our host

**Decision:** missing upstream checker/provider registration is handled behind the framework's reviewed host adapter. An upstream provider PR is optional.

**Consequence:** product delivery does not depend on upstream acceptance, while the adapter remains replaceable and compatibility-probed.

### ADR-004 — Ketho is a compatibility oracle

**Decision:** use Ketho annotation behavior, field corrections, and output semantics for parity fixtures. Do not copy editor-setting mutation policy.

**Consequence:** familiar annotations without editor-dependent correctness.

### ADR-005 — Numy is a differential oracle

**Decision:** use Numy FramexmlAnnotations for comparison and corpus conventions; canonical FrameXML facts come from structural source analysis under repository contracts.

**Consequence:** disagreements become coverage/triage records rather than silent replacements.

### ADR-006 — Raw metadata and annotations are separate projections

**Decision:** retain all raw Blizzard metadata and unknown fields independently of generated annotations.

**Consequence:** Secret/restriction and future fields are not lost when a target annotation format cannot express them.

### ADR-007 — Blizzard content is canonical; acquisition provider is provenance

**Decision:** Gethe or another mirror may acquire a snapshot, but the materialized Blizzard UI content and exact digest define the platform input.

**Consequence:** equivalent official/local inputs can produce the same logical pack while acquisition provenance remains explicit.

### ADR-008 — One active profile per project generation

**Decision:** diagnostics use one selected profile. Historical, PTR, beta, regional, and flavor data remain separate.

**Consequence:** no cross-profile signature, availability, or restriction leakage.

### ADR-009 — WoW ownership is multi-axis

**Decision:** store lexical, ownership, load, object, inheritance, registration, lifecycle, state, call, and lineage relationships independently.

**Consequence:** queries expose explicit chains instead of one ambiguous parent relation.

### ADR-010 — Recognizers emit universal roles

**Decision:** framework packs are declarative data over structural facts; production behavior does not branch on addon/repository/owner/path/popularity identity.

**Consequence:** calibration corpora improve universal coverage without creating product-specific modes.

### ADR-011 — Emmy is the sole correctness-path Lua parser

**Decision:** WoW recognizers consume canonical Emmy syntax/semantic facts.

**Consequence:** no parser/dialect/span disagreement inside the correctness path.

### ADR-012 — External Codebase Memory remains optional and isolated

**Decision:** external discovery uses reviewed typed E6 transport contracts only. Framework code never writes provider databases or exposes a generic MCP/tool/RPC escape hatch.

**Consequence:** broad discovery remains independently upgradable and can fail without degrading exact local/reference/project capability.

### ADR-013 — Own a small exact WoW sidecar graph

**Decision:** store TOC/XML/load/API/UI/state/restriction/lineage facts in a domain-specific SQLite graph until a supported external import ABI and measured need justify another substrate.

**Consequence:** exact WoW correctness does not depend on a generic external graph schema.

### ADR-014 — Structured evidence ranks before similarity

**Decision:** exact identity, explicit alias/deprecation/replacement/transition/lineage evidence, and compatible shape signals precede fuzzy, text, and semantic similarity.

**Consequence:** similarity generates Candidates but cannot prove migration, impact, absence, or authorize a fix.

### ADR-015 — SQLite is the first storage/search/graph substrate

**Decision:** use SQLite B-tree indexes, FTS5, adjacency tables, WAL for mutable project state, immutable objects, and bounded in-memory projections.

**Consequence:** no graph/search/vector server in v1 without measured unique need.

### ADR-016 — Restriction facets are open

**Decision:** preserve unknown facets raw and make dependent rules `NotEvaluated`.

**Consequence:** new Blizzard fields degrade honestly instead of being ignored or treated as safe.

### ADR-017 — Community addons are examples, not platform authority

**Decision:** third-party implementations provide structural and implementation evidence only.

**Consequence:** patch-sensitive contracts are revalidated against pinned Blizzard source and required runtime evidence.

### ADR-018 — External source acquisition is explicit and non-vendored

**Decision:** a later approved host/source-owner contract may materialize an exact external repository revision on demand. E6-A/E6-B do not install/index providers, clone/follow provider locators, or redistribute full external repositories.

**Consequence:** source acquisition, provider-index lifecycle, locator mapping, and candidate querying remain separate effects with clear license and trust boundaries.

### ADR-019 — Skeleton-first agent reads

**Decision:** agents receive L0/L1 skeletons before L2/full source.

**Consequence:** smaller context, explicit source handles, and fewer unnecessary repository reads.

### ADR-020 — Project architecture memory is generated

**Decision:** generate a compact portable Project Map from exact project/graph generations rather than maintaining personal/manual memory cards.

**Consequence:** every user and agent receives the same repository-derived state.

### ADR-021 — MIT/public release target

**Decision:** framework code and generated framework templates target MIT with visible public development; third-party/Blizzard/provider artifacts retain separate provenance/license/notice/redistribution decisions.

**Consequence:** repository visibility does not broaden rights to external material or remove release-evidence requirements.

### ADR-022 — No default component without justification

**Decision:** a component enters the default path only for unique correctness responsibility or measured task benefit.

**Consequence:** architecture does not grow from fashion, donor availability, provider marketing, or arbitrary crate counts.

### ADR-023 — External provider authority has a hard Candidate ceiling

**Decision:** every E6 provider result remains `provenance=semantic_candidate`, `confidence=Candidate`, and `negative_authority=unavailable` regardless of provider labels, rank, score, repetition, stable state, or zero result.

**Consequence:** external discovery cannot silently become platform/project/source/runtime truth or clean negative authority.

### ADR-024 — Source locator mapping belongs to exact owners

**Decision:** only `wow-project` or `wow-reference` may map an external locator into an exact retained owner record. They consume an owner-neutral bounded projection and do not depend on `wow-cbm` or `wow-service`.

**Consequence:** service/application cannot inspect paths or choose mappings, dependency direction remains valid, and `ExactMapped` proves locator identity only.

### ADR-025 — External candidate selection is explicit and non-authoritative

**Decision:** the caller supplies exact candidate/mapping IDs and `Selected`, `Rejected`, or `Deferred`. Service records but never infers the decision.

**Consequence:** top/sole/highest-score/nearest/same-name candidates are never automatically selected, and selection does not authorize verification, edits, migration, or promotion.

### ADR-026 — External provider metadata remains a sidecar

**Decision:** exact mapped-root context is produced through normal `wow-context` contracts. Provider labels/scores/snippets/summaries/traces remain in a separate `ExternalCandidateSidecar`.

**Consequence:** mapping and context inclusion cannot contaminate `ContextSemanticPack` truth or graph authority.

### ADR-027 — External provider/session effects are durable and secret-isolated

**Decision:** E6-B uses exact nonsecret provider configuration and authorization references, narrow host-owned sessions, and durable operation/effect receipts. Raw secret material, private endpoints, process/database handles, and provider lifecycle controls do not cross public seams.

**Consequence:** response loss becomes `OutcomeUnknown`, blind redispatch is forbidden, and provider integration remains replaceable without leaking credentials into repository/CLI/results.

### ADR-028 — Launch states are separate gates

**Decision:** documented architecture, first runnable executable, useful internal alpha, developer preview, governed beta, and public supported v1 are separate states defined in `LAUNCH_GATES.md`.

**Consequence:** later documentation does not block the E0 runnable bootstrap, and a compiling binary cannot be mislabeled as a supported release.

### ADR-029 — Frontends are thin service-only transports

**Decision:** CLI, CLI-daemon, LSP, MCP, and future frontends depend on `wow-service` only; one transport request maps to one service operation unless the workflow is itself a documented service operation.

**Consequence:** transport differences cannot change semantic authority, bypass owner validation, expose generic tools, or duplicate business logic.

### ADR-030 — Close-before-success and exact response-loss recovery

**Decision:** every durable/external effect uses exact operation/request identity; required retention/audit and reverse-order resource closure complete before public success.

**Consequence:** timeouts/disconnects cannot be treated as no effect, and cleanup/retry cannot continue as unowned background work.

## Corrected earlier assumptions

```text
Gethe-first
-> Blizzard UI content/digest first; Gethe is acquisition provenance.

external Emmy process only
-> pin the public upstream Rust analysis implementation behind one adapter.

ast-grep recognizers
-> declarative recognizers over canonical Emmy facts.

Codebase Memory-only graph
-> optional external Candidate discovery plus an exact framework-owned WoW graph.

provider result -> stable source handle
-> UnverifiedProviderLocator -> exact project/reference owner mapping -> explicit selection.

provider/session/index lifecycle inside wow-cbm
-> E6-A pure candidate owner, E6-B service orchestration, later explicit host lifecycle contracts.

current-only reference
-> one exact active target plus configured historical/transition generations.

personal knowledge cards
-> generated Project Map and public rule packs.

all planned documentation required before first run
-> E0-A through E0-F is the first runnable gate; later packages layer on top.
```

## Rejected directions

- implementation targets based on dozens of artificial microcrates;
- custom graph/search/vector servers in v1 without benchmarks;
- an internal vector database by default;
- hardcoded addon/repository-specific production logic;
- deep interprocedural Secret flow before local rules work;
- editor-setting mutation as a correctness mechanism;
- direct external provider database writes;
- generic MCP/tool/RPC or shell escape hatches;
- provider labels/scores/zero results as authority;
- service/application-side source locator mapping;
- implicit top/sole/best candidate selection;
- provider metadata inside exact context truth;
- provider failure disabling exact local workflows;
- full Blizzard UI source in default model context;
- a single generic parent relation for WoW ownership;
- decorative CI/release automation before executable commands and launch gates exist.

## Proposed decisions

Candidate decisions are tracked in [`IDEAS.md`](IDEAS.md). They become accepted only after a documented experiment, explicit owner, migration analysis, and ADR update.