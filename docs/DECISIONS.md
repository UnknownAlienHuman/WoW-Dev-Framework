# Architecture decisions

**Status: normative**

This register converts the v8.0 architecture decisions into stable English repository policy. A decision remains accepted until replaced by a new ADR that states migration consequences.

## Status vocabulary

- **Accepted** — implementation must conform.
- **Proposed** — requires review and an acceptance experiment.
- **Superseded** — retained for history; replacement decision controls.
- **Rejected** — explicitly outside the current design.

## Accepted decisions

### ADR-001 — Upstream EmmyLua dependency, not a fork

**Decision:** use the upstream Rust analysis library pinned behind one adapter.

**Consequence:** upstream updates are compatibility-probed and can be rolled back without maintaining a permanent analyzer fork.

### ADR-002 — One host merges generic and WoW diagnostics

**Decision:** `wow-emmy-ls`/core runs built-in Emmy diagnostics and WoW providers within one project generation.

**Consequence:** source spans, profile identity, capability state, and output ordering are coherent.

### ADR-003 — External provider gap is handled by our host

**Decision:** the current lack of public upstream checker registration is handled in the framework host. An upstream provider PR is optional.

**Consequence:** product delivery does not depend on upstream acceptance.

### ADR-004 — Ketho is a compatibility oracle

**Decision:** use Ketho's annotation behavior, field corrections, and output semantics for parity fixtures. Do not copy its editor-setting mutation policy.

**Consequence:** familiar annotations without editor-dependent correctness.

### ADR-005 — Numy is a differential oracle

**Decision:** use Numy FramexmlAnnotations for comparison and corpus conventions; canonical FrameXML facts come from structural source parsing.

**Consequence:** disagreements become coverage/triage records rather than silent replacements.

### ADR-006 — Raw metadata and annotations are separate projections

**Decision:** retain all raw Blizzard metadata and unknown fields independently of generated annotations.

**Consequence:** Secret/restriction and future fields are not lost when a target annotation format cannot express them.

### ADR-007 — Blizzard content is canonical; provider is provenance

**Decision:** Gethe or another mirror may acquire a snapshot, but the materialized Blizzard UI content and exact digest define the platform input.

**Consequence:** equivalent official/local inputs can produce the same logical pack.

### ADR-008 — One active profile per project generation

**Decision:** diagnostics use one selected profile. Historical, PTR, beta, and flavor data remain separate.

**Consequence:** no cross-profile signature or restriction leakage.

### ADR-009 — WoW ownership is multi-axis

**Decision:** store lexical, ownership, load, object, inheritance, registration, lifecycle, state, and call relationships independently.

**Consequence:** queries expose explicit chains instead of one ambiguous parent.

### ADR-010 — Recognizers emit universal roles

**Decision:** framework packs are declarative data over structural facts; production behavior does not branch on addon repository names.

**Consequence:** calibration corpora improve coverage without creating product-specific modes.

### ADR-011 — Emmy is the sole correctness-path Lua parser

**Decision:** WoW recognizers consume canonical Emmy syntax/semantic facts.

**Consequence:** no parser/dialect/span disagreement inside the correctness path.

### ADR-012 — Codebase Memory remains external and unchanged

**Decision:** use documented transport only; never mutate its database.

**Consequence:** broad semantic discovery is optional and independently upgradable.

### ADR-013 — Own a small WoW sidecar graph

**Decision:** store TOC/XML/load/API/UI/state/restriction/lineage facts in a domain-specific SQLite graph until a supported external import ABI exists.

**Consequence:** exact WoW correctness does not depend on a generic graph schema.

### ADR-014 — Structured evidence ranks before similarity

**Decision:** exact, alias, deprecation, replacement, lineage, and shape signals precede fuzzy, text, and semantic similarity.

**Consequence:** similarity generates candidates but cannot prove migration or authorize a fix.

### ADR-015 — SQLite is the first storage/search/graph substrate

**Decision:** use SQLite B-tree indexes, FTS5, adjacency tables, WAL for project state, and bounded in-memory graph projections.

**Consequence:** no graph/search/vector server in v1 without measured need.

### ADR-016 — Restriction facets are open

**Decision:** preserve unknown facets raw and make dependent rules `NotEvaluated`.

**Consequence:** new Blizzard fields degrade honestly instead of being ignored or treated as safe.

### ADR-017 — Community addons are examples, not platform authority

**Decision:** third-party implementations provide structural and implementation evidence only.

**Consequence:** patch-sensitive contracts are revalidated against pinned Blizzard source.

### ADR-018 — External source is cloned on demand, not vendored

**Decision:** store manifests, revisions, licenses, digests, roles, and source handles; do not redistribute full external repositories.

**Consequence:** smaller releases and clearer license boundaries.

### ADR-019 — Skeleton-first agent reads

**Decision:** agents receive L0/L1 skeletons before L2/full source.

**Consequence:** smaller context, explicit source handles, and fewer unnecessary repository reads.

### ADR-020 — Project architecture memory is generated

**Decision:** generate a compact portable Project Map from the project graph rather than maintaining personal/manual memory cards.

**Consequence:** every user and agent receives the same repository-derived state.

### ADR-021 — MIT/public release target

**Decision:** the tool and generated templates use MIT, with visible/unobfuscated public development as the release target.

**Consequence:** third-party notices and source provenance are retained; private bootstrap status does not change the intended license.

### ADR-022 — No default component without justification

**Decision:** a component enters the default path only for unique correctness responsibility or measured agent-task benefit.

**Consequence:** architecture does not grow from fashion, donor availability, or arbitrary crate counts.

## Corrected earlier assumptions

The accepted architecture explicitly corrected:

```text
Gethe-first
→ Blizzard UI first; Gethe is an acquisition provider.

external Emmy process only
→ embed the public upstream Rust library behind one adapter.

ast-grep recognizers
→ declarative recognizers over canonical Emmy facts.

Codebase Memory-only graph
→ optional broad graph plus a small exact WoW graph.

current-only reference
→ one active target plus configured historical lineage.

personal knowledge cards
→ generated Project Map and public rule packs.
```

## Rejected directions

- implementation targets based on dozens of artificial microcrates;
- custom graph/search servers in v1;
- an internal vector database;
- hardcoded addon-specific production logic;
- deep interprocedural Secret flow before local rules work;
- editor-setting mutation as a correctness mechanism;
- direct Codebase Memory database writes;
- full Blizzard UI source in default model context;
- a single generic parent relation for WoW ownership.

## Proposed decisions

Candidate decisions are tracked in `IDEAS.md`. They become accepted only after a documented experiment and ADR update.
