# `wow-project` E0-D decisions

**Status:** normative for the E0-D minimal project-generation slice.

## PROJECT-001 — E0-D uses one closed fixture project

**Decision:** E0-D operates on `fixture-project-e0-v1` and the four Main Lua files declared by the E0-C fixture.

**Consequence:** project-generation semantics are proven without scanning a real repository or introducing TOC/XML behavior prematurely.

## PROJECT-002 — Project generation is owned here

**Decision:** `wow-project` derives and publishes the canonical `ProjectGenerationId`.

**Consequence:** `wow-emmy` receives/validates that generation and owns only its analyzer snapshot identity.

## PROJECT-003 — Project generation includes analysis-affecting inputs

**Decision:** generation identity includes project configuration, selected profile/reference generation, accepted analyzer pin/probe/configuration identity, and normalized file content manifest.

**Consequence:** changing analyzer behavior/configuration creates a new project generation even when source bytes are unchanged.

## PROJECT-004 — Volatile machine state is excluded

**Decision:** timestamps, checkout/temp paths, thread IDs, process IDs, memory addresses, credentials, and discovery order do not enter project identity.

**Consequence:** equivalent logical inputs produce identical generations across machines/runs.

## PROJECT-005 — Explicit input manifest replaces repository scanning in E0

**Decision:** E0-D receives a closed file inventory and bytes/identities from the harness.

**Consequence:** no filesystem crawler, watcher, Git integration, installed-addon discovery, or implicit source inclusion exists.

## PROJECT-006 — One registered project source origin

**Decision:** all E0 first-party files belong to `project-origin:fixture-project-e0-v1` and one Main workspace.

**Consequence:** project/library/reference origins cannot be confused, and public handles remain root-relative.

## PROJECT-007 — Source content has one fixture owner

**Decision:** E0-D references the canonical Main-file content identities selected from the E0-C fixture contract rather than maintaining divergent source text.

**Consequence:** analyzer and project tests operate on the same bytes/digests after byte freeze.

## PROJECT-008 — Direct E0 dependencies are only `wow-core` and `wow-emmy`

**Decision:** permitted future edges to store/graph/recognizers remain inactive.

**Consequence:** E0-D cannot smuggle E2 architecture into the minimal slice.

## PROJECT-009 — Publication is atomic

**Decision:** no `ProjectSnapshot` becomes visible until project inputs, analyzer update, analyzer snapshot, coverage, and canonical digest all validate.

**Consequence:** consumers cannot observe half-updated files/facts/findings.

## PROJECT-010 — Analyzer snapshot must match the target generation

**Decision:** project/profile/reference generation, analyzer pin/configuration, and file manifest are checked before publication.

**Consequence:** a healthy analyzer snapshot for the wrong context is rejected.

## PROJECT-011 — Analyzer failure prevents target publication

**Decision:** update/index/snapshot failure leaves the target generation unpublished.

**Consequence:** prior snapshot may remain available only under its original generation; it is never relabeled current.

## PROJECT-012 — Project snapshot references analyzer outputs, not copies them

**Decision:** `ProjectSnapshot` stores the validated analyzer snapshot/read-view identity and exposes normalized views; it does not duplicate/rewrite analyzer facts/findings.

**Consequence:** fact ownership and invalidation stay with `wow-emmy`.

## PROJECT-013 — Project capability records are explicit

**Decision:** project configuration/files/source registry/analyzer binding each emit exact capability/coverage records.

**Consequence:** a missing analyzer/local-flow capability cannot appear as an empty successful project result.

## PROJECT-014 — Deferred E2 capabilities are absent or `NotEvaluated`

**Decision:** TOC/XML/load/state/event/hook/graph capabilities are not implemented in E0-D.

**Consequence:** no fake empty graph/load model is exposed.

## PROJECT-015 — Profile/reference selection is explicit

**Decision:** project configuration carries one exact fixture profile/reference generation supplied by the harness/service.

**Consequence:** no floating current/live/PTR/beta/flavor fallback or installed-client inference.

## PROJECT-016 — Project file identity is path-stable and content-versioned

**Decision:** logical `ProjectFileId` derives from project origin/workspace/path; content digest/version changes independently.

**Consequence:** updates preserve logical file identity while producing a new project generation.

## PROJECT-017 — Update preconditions are optimistic and exact

**Decision:** update/remove requires expected current project generation and expected old file digest.

**Consequence:** stale agents cannot overwrite a newer project state silently.

## PROJECT-018 — Update batches describe final logical state

**Decision:** canonical generation derives from final validated configuration/file manifest, not the order of equivalent update operations.

**Consequence:** different update sequences producing identical final state yield the same project generation/snapshot bytes.

## PROJECT-019 — Add/remove stays inside configured first-party root

**Decision:** E0 add/remove supports only declared Lua files under the registered Main root and budgets.

**Consequence:** no hidden root expansion, Library promotion, symlink escape, or arbitrary file type.

## PROJECT-020 — No background publication

**Decision:** E0-D performs explicit synchronous/domain-level update/publication requests only.

**Consequence:** no watcher, daemon, callback, timer, or asynchronous state race enters the project contract.

## PROJECT-021 — Project source handles are authorization-neutral

**Decision:** registered source identity proves location/content/context, not permission to read arbitrary host paths.

**Consequence:** resolution remains root-scoped and explicit.

## PROJECT-022 — Project evidence does not become platform evidence

**Decision:** project source/facts describe addon source only.

**Consequence:** `wow-rules` must join them with `wow-reference` for API/restriction conclusions.

## PROJECT-023 — No diagnostic algorithms here

**Decision:** generic findings originate in `wow-emmy`; WoW findings originate in `wow-rules`.

**Consequence:** project layer validates/exposes generation-bound findings but does not change severity, root cause, or remediation.

## PROJECT-024 — No persistence in E0-D

**Decision:** project state is in-memory/test-fixture state; `wow-store` activates later.

**Consequence:** no SQLite schema/migration/WAL/object store is introduced.

## PROJECT-025 — Failed candidate state is not a project generation

**Decision:** deriving a target ID does not mean publication succeeded.

**Consequence:** APIs distinguish `ProjectGenerationCandidate` from published `ProjectSnapshot`/generation.

## PROJECT-026 — Project snapshot is immutable

**Decision:** update always builds a new candidate/snapshot.

**Consequence:** readers can safely use one coherent generation without observing mutation.

## PROJECT-027 — Last-known-good is explicit and stale by identity

**Decision:** retained prior snapshots keep original generation/analyzer identity and are reported as last-known-good, not current target state.

**Consequence:** service can degrade honestly without cross-generation substitution.

## PROJECT-028 — Fixture hashes freeze before implementation

**Decision:** project/analyzer/source/checksum IDs are null while documentation-only, then must be frozen after E0-A/E0-C implementation and before first project Rust commit.

**Consequence:** documentation does not invent hashes, but code cannot start against mutable fixture identity.

## PROJECT-029 — Current KB remains external

**Decision:** live WoW patch/security/project guidance is linked, not copied into the minimal project-generation model.

**Consequence:** this crate stays stable and generic while current platform guidance evolves.

## PROJECT-030 — E2 expansion requires a contract transition

**Decision:** TOC/XML/load/project graph work cannot be added as incidental E0-D enhancements.

**Consequence:** activating E2 updates manifest/workstreams/contracts/dependencies/tests explicitly.
