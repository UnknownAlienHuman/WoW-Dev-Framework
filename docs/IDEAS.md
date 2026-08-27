# Candidate ideas

**Status: non-normative**

These proposals are compatible with the accepted architecture but are not implementation commitments. Each idea must earn an ADR through a small experiment, measurable benefit, or unique correctness responsibility.

## Near-term design hardening

### I-001 — Capability handshake as the first public operation

**Idea:** make `wow_status` return a machine-readable capability matrix before any lookup/check workflow.

**Why:** most dangerous false conclusions begin with an implicit profile, stale generation, or incomplete partition.

**Experiment:** E0 client calls `status`, caches its generation token, and rejects a diagnostic response whose token differs. Add complete/partial/failed fixture variants.

**Accept when:** it prevents mixed-generation or false-negative fixtures without materially complicating the service API.

**Reject when:** equivalent safety is already guaranteed by every result envelope with no extra round trip.

### I-002 — Evidence capsule export

**Idea:** allow any finding/search hit/impact item to export a compact reproducibility capsule containing profile, generations, source handles, producer versions, capability state, and relevant normalized facts.

**Why:** bug reports and agent handoffs often lose the exact evidence that produced a conclusion.

**Experiment:** export/import one E0 diagnostic capsule and reproduce the same normalized finding without the original workspace.

**Accept when:** capsules materially reduce reproduction setup and remain small, deterministic, and privacy-safe.

**Reject when:** they become hidden mini-databases or duplicate whole source files.

### I-003 — Unknown-field quarantine lane

**Idea:** when Blizzard introduces an unrecognized APIDocumentation or restriction field, preserve it in a quarantine table and generate a schema-gap report with affected capabilities.

**Why:** silent field loss is more dangerous than an explicit partial profile.

**Experiment:** mutate a fixture with a new nested field and verify raw round-trip, capability downgrade, and triage output.

**Accept when:** the builder remains usable while preventing authoritative dependent answers.

**Reject when:** quarantine semantics make known unaffected partitions unusable.

### I-004 — Root-cause diagnostic graph

**Idea:** represent diagnostic causality explicitly, not only as message deduplication.

**Example:** failed annotation load → invalid library state → many unknown globals. The default stream shows the failed annotation load; raw descendants remain inspectable.

**Experiment:** inject three known root failures and measure diagnostic-count reduction without hiding independent errors.

**Accept when:** agent remediation improves and independent findings remain visible.

**Reject when:** grouping relies on message text or model judgment rather than deterministic causes.

### I-005 — Autofix proof tiers

**Idea:** classify remediation as:

```text
exact_edit       mechanically proven and directly applicable
validated_recipe structured transformation requiring post-check
plan_only        evidence-backed steps, no automatic edit
candidate_only   investigation options, insufficient proof
```

**Why:** a boolean “has fix” obscures the difference between exact API migration and similarity-based advice.

**Experiment:** apply tiers to E0/API migration fixtures and verify no candidate can produce an edit.

**Accept when:** the policy is simple enough to enforce in schemas and tests.

**Reject when:** tiers duplicate confidence without adding application safety.

### I-006 — Cross-provider Reference Pack attestation

**Idea:** materialize the same logical Blizzard snapshot through two acquisition paths when possible and compare canonical pack digests.

**Why:** this detects mirror packaging drift and proves that provider is provenance rather than authority.

**Experiment:** run one small fixture through a mirror-shaped input and a local-export-shaped input.

**Accept when:** logical output converges and differences are actionable.

**Reject when:** provider-specific normalization dominates cost without improving trust.

## Patch and runtime intelligence

### I-007 — Patch delta risk classifier

**Idea:** classify Reference Pack deltas into risk domains:

```text
signature/return change
restriction-facet change
package/load move
template/inheritance change
event payload change
explicit removal/replacement
documentation-only change
unknown schema change
```

Then intersect those classes with project facts to select tests and runtime scenarios.

**Experiment:** label historical deltas and compare the generated study set with a manually curated migration plan.

**Accept when:** it reduces irrelevant impact items while retaining known breakages.

**Reject when:** a single opaque risk score replaces explicit evidence.

### I-008 — Structured runtime probe importer

**Idea:** define a minimal JSON/SQLite import contract for client probes that records build, context, probe version, raw digest, and observation.

**Why:** runtime-only Secret/protected behavior should be reproducible and joinable without turning anecdotal logs into source authority.

**Experiment:** import one synthetic probe, associate it with a finding, and ensure it remains scenario-scoped.

**Accept when:** privacy and provenance are enforceable and static outputs remain valid without probes.

**Reject when:** the format encourages storing full combat/chat logs or user data.

### I-009 — Hotfix freshness lease

**Idea:** allow profile metadata to declare data domains whose runtime truth may age faster than the Lua source surface. Queries involving those domains require a freshness lease or explicit stale warning.

**Why:** some restriction data can change independently of the API files used to build the pack.

**Experiment:** mark a fixture facet as hotfix-sensitive and test fresh/stale/runtime-confirmed states.

**Accept when:** it prevents overconfident static claims without making all profiles permanently stale.

**Reject when:** freshness cannot be defined per domain and becomes an arbitrary timer.

## Evaluation and maintainability

### I-010 — Query replay corpus

**Idea:** store normalized query intent, profile, expected evidence class, expected top entities, and required source-read budget. Do not store model prose as the oracle.

**Why:** task evaluations need stable semantics across models and prompt changes.

**Experiment:** replay 10 architecture tasks through CLI and MCP and compare normalized outputs.

**Accept when:** results are transport-independent and failures are diagnosable.

**Reject when:** the corpus overfits exact wording instead of task intent.

### I-011 — Context-budget telemetry

**Idea:** record source reads, bytes, skeleton levels, graph nodes, and external candidate count for evaluation runs.

**Why:** “small context” is otherwise an aspiration without enforcement.

**Experiment:** compare baseline manual research with framework-assisted E3/E4 tasks.

**Accept when:** telemetry identifies regressions and correlates with accepted task outcomes.

**Reject when:** collection leaks source content or slows normal queries materially.

### I-012 — Recognizer mutation laboratory

**Idea:** automatically rename repositories, move files, alter factory names, add structurally similar negatives, and remove named packs.

**Why:** this detects hidden repository-name/path overfitting and fragile call-pattern recognizers.

**Experiment:** run mutations against initial Ace3/oUF calibration rules.

**Accept when:** failures expose real non-universal assumptions.

**Reject when:** synthetic mutations are unrealistic and dominate maintenance without predictive value.

### I-013 — Differential triage bundles

**Idea:** when Ketho, Numy, Emmy, and our extractor disagree, emit a single triage bundle containing normalized facts and exact source handles.

**Why:** differential tests are only valuable when a maintainer can resolve the mismatch quickly.

**Experiment:** create three intentional disagreements and measure manual triage steps.

**Accept when:** bundles remain compact and distinguish projection loss from source disagreement.

**Reject when:** they duplicate full oracle outputs.

### I-014 — Public schema compatibility simulator

**Idea:** before changing a public pack/result schema, replay stored older consumers and migration fixtures against the candidate schema.

**Why:** contract breakage should be detected before release packaging.

**Experiment:** define one v0→v1 synthetic migration with additive, deprecated, and breaking changes.

**Accept when:** it produces a clear compatibility report and enforces version rules.

**Reject when:** there are no real consumers yet; defer until a first public schema exists.

## Security and resilience

### I-015 — Malicious corpus qualification suite

**Idea:** require every parser/indexer milestone to pass a bounded suite of traversal, symlink, XML expansion, huge-table, malformed encoding, source prompt-injection, and candidate-authority spoofing fixtures.

**Why:** external repositories and generated UI inputs are untrusted.

**Experiment:** add the smallest fixture for each threat class and enforce time/memory/output bounds.

**Accept when:** the suite catches regressions without executing fixture content.

**Reject when:** a fixture depends on platform-specific unsafe behavior that cannot be reproduced reliably.

### I-016 — Generation leases and cancellation

**Idea:** long-running graph/search/impact requests hold an immutable generation lease and support cooperative cancellation.

**Why:** editor updates should not produce mixed-generation answers or retain obsolete snapshots indefinitely.

**Experiment:** update files during a delayed query and verify coherent old-generation output followed by garbage collection.

**Accept when:** the model is simpler than copying state and prevents cross-generation bugs.

**Reject when:** E0/E1 workloads do not justify it; defer until concurrent service operation exists.

## Later experiments

### I-017 — Patch-impact test synthesis

Generate a proposed test matrix from changed API/facet/template/load relations and project usage. Output remains a plan until a human or project harness supplies executable scenarios.

### I-018 — Source-handle content negotiation

Allow clients to request L0, L1, exact span, surrounding declaration, or full file with an explicit byte budget. Measure whether this improves multi-agent coordination.

### I-019 — DerivedFacts interoperability package

Prototype a generic versioned facts pack only after the local node/edge semantics are stable. The goal is interoperability, not replacing the owning WoW store prematurely.

### I-020 — Local explanation audit

For every default rule and ranking signal, provide a deterministic “why this result” trace suitable for tests. Avoid model-generated explanations in the correctness path.

## Promotion process

To promote an idea:

1. create a focused issue with the smallest decisive experiment;
2. identify affected accepted decisions and public contracts;
3. implement behind an experimental flag or fixture-only path;
4. measure correctness or agent-task benefit;
5. document failure modes and operational cost;
6. accept, revise, defer, or reject through an ADR update.
