# `wow-service` E0-F decisions

**Status:** normative for the E0-F `status`/`check` orchestration slice.

## SERVICE-001 — Service is the only cross-component orchestrator

**Decision:** applications and domain crates do not coordinate reference/project/analyzer/rules directly.

**Consequence:** one layer owns generation coherence, operation ordering, partial degradation, and public result assembly.

## SERVICE-002 — E0 exposes only `status` and `check`

**Decision:** every later public operation remains typed Deferred.

**Consequence:** no empty search/tree/plan/impact/index/runtime results or generic future command bus.

## SERVICE-003 — E0 direct dependencies are minimal

**Decision:** only core/reference/emmy/project/rules are active.

**Consequence:** store/annotations/graph/recognizers/search/CBM/context cannot leak into the executable slice.

## SERVICE-004 — One immutable context per request

**Decision:** service acquires and validates one coherent reference/project/analyzer/rule context before operation execution.

**Consequence:** no mid-request refresh, mixed generation, or partial context switch.

## SERVICE-005 — `CurrentPublished` is an explicit selector, not identity

**Decision:** a request may select the atomically published snapshot for one exact project ID; service immediately resolves and records the exact generation.

**Consequence:** no unscoped latest/current and no canonical output containing only a floating selector.

## SERVICE-006 — Exact generation selectors never fall back

**Decision:** an unavailable/mismatched exact generation fails.

**Consequence:** last-known-good/current-newer snapshots are not silent substitutes.

## SERVICE-007 — Status is observation, not validation

**Decision:** status reports component/configuration/generation/capability state only.

**Consequence:** Ready/installed does not imply clean, tests passed, or runtime-verified.

## SERVICE-008 — Check consumes lower-layer public contracts only

**Decision:** generic findings come from ProjectView/analyzer; WoW outcomes come from RuleExecutor; reference/project identities come from their immutable views.

**Consequence:** service does not reimplement parsing, rule, authority, or fact algorithms.

## SERVICE-009 — Raw findings are immutable input/output

**Decision:** every generic/WoW finding remains in `raw_findings` with original ID/evidence/context.

**Consequence:** presentation folding never destroys auditability.

## SERVICE-010 — Presentation folding is a separate graph

**Decision:** display roots/children are a deterministic projection over raw records and structured causal/blocker hints.

**Consequence:** one can inspect both concise root causes and the complete raw diagnostic stream.

## SERVICE-011 — Message similarity is never causal evidence

**Decision:** folding uses exact finding/root/fact/source/context IDs and relation contracts.

**Consequence:** wording/localization changes cannot alter cause grouping.

## SERVICE-012 — E0 presentation graph is acyclic

**Decision:** cycles are invalid and block result publication.

**Consequence:** deterministic traversal/rendering and no self-explaining diagnostic loops.

## SERVICE-013 — One primary presentation parent per child in E0

**Decision:** a child may have one selected primary parent; other valid causes remain related evidence/secondary relations.

**Consequence:** display tree is deterministic without discarding competing causal evidence.

## SERVICE-014 — Service status is independent of rule rollout

**Decision:** any raw finding yields semantic `findings` when no blockers, even when rollout is advisory.

**Consequence:** CLI/policy may choose exit behavior separately; service does not call advisory findings clean.

## SERVICE-015 — Partial has precedence over findings

**Decision:** findings plus any requested-scope `NotEvaluated`, degradable failure, or truncation yields `partial`.

**Consequence:** users see both found problems and incomplete analysis.

## SERVICE-016 — Failed means no coherent operation result

**Decision:** mandatory context/component/result-contract failure produces `failed`, not partial.

**Consequence:** a structurally invalid envelope or mixed generation never ships as useful partial data.

## SERVICE-017 — Cancelled publishes no result envelope

**Decision:** cancellation before publication yields a cancellation record/result projection, no late check envelope.

**Consequence:** no background continuation or partially canonicalized result.

## SERVICE-018 — Clean requires explicit completion proof

**Decision:** clean requires zero raw findings, zero blockers, complete requested scopes, and complete budget.

**Consequence:** empty arrays are insufficient.

## SERVICE-019 — Generic and rule outcomes remain separately addressable

**Decision:** result contains raw generic findings plus rule execution outcomes/clean/NotEvaluated records.

**Consequence:** service presentation does not collapse provenance or rule coverage.

## SERVICE-020 — Root-cause relations do not change finding identity

**Decision:** presentation graph references existing IDs and has its own identity/digest.

**Consequence:** folding policy changes can be reviewed independently from underlying findings.

## SERVICE-021 — Exact duplicate folding is structural

**Decision:** `exact_duplicate_of` requires equivalent structured finding identity/context/source, not message equality.

**Consequence:** distinct use sites are not merged.

## SERVICE-022 — Component blocker relations are explicit

**Decision:** annotation-library/profile/capability failures may block rules only through structured dependency/blocker records.

**Consequence:** service cannot invent a blocker from coincident timing or wording.

## SERVICE-023 — One canonical envelope per successful coherent operation

**Decision:** status/check responses use versioned canonical envelopes with exact identities and ordering.

**Consequence:** transports cannot add/remove semantic fields or reorder identity-bearing data.

## SERVICE-024 — Volatile telemetry is noncanonical

**Decision:** wall-clock timing, process/thread IDs, temp paths, and human render text stay outside canonical digest.

**Consequence:** repeated equivalent operations serialize byte-identically.

## SERVICE-025 — Deferred operations fail explicitly

**Decision:** lookup/search/tree/skeleton/plan/impact/index/runtime/LSP/MCP return `operation_not_implemented_for_milestone`.

**Consequence:** status can list roadmap state without pretending APIs work.

## SERVICE-026 — Last-known-good identity is never rewritten

**Decision:** service reports retained snapshots exactly and does not satisfy another generation selector with them.

**Consequence:** degradation is honest and reproducible.

## SERVICE-027 — Service does not own source edits

**Decision:** E0 result may contain `plan_only` remediation but service applies no mutations.

**Consequence:** no generation race, hidden filesystem write, or unverified autofix.

## SERVICE-028 — CLI is a projection over service

**Decision:** `apps/wow` owns arguments/formats/stdout/stderr/exit codes only.

**Consequence:** CLI cannot import lower crates to generate a richer/different answer.

## SERVICE-029 — JSON is the canonical E0 transport representation

**Decision:** canonical JSON is the golden/API surface; human text is a noncanonical projection.

**Consequence:** deterministic integration tests are independent of prose formatting.

## SERVICE-030 — CLI exit codes communicate operation status, not hidden release policy

**Decision:** E0 maps clean/findings/partial/failed/cancelled to explicit codes; advisory rollout remains visible in records.

**Consequence:** scripts can distinguish findings from incomplete analysis without service semantic distortion.

## SERVICE-031 — No generic plugin/transport framework in E0

**Decision:** E0 implements one in-process service surface and one CLI projection.

**Consequence:** no premature MCP/LSP/plugin registry/daemon architecture.

## SERVICE-032 — Freeze all prerequisite/result vectors before code

**Decision:** component IDs, findings/outcomes/graph/envelope IDs and checksums remain null while documentation-only and become mandatory before first Rust commit.

**Consequence:** implementation cannot invent or drift expected integration outputs.

## SERVICE-033 — Current KB remains external

**Decision:** live WoW guidance is linked, not embedded in orchestration logic.

**Consequence:** service contracts remain stable and evidence-oriented.
