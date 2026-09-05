# `wow-service` implementation contract

**Status:** E0-F implementation-ready contract; no Rust code yet. Only `status` and `check` activate in E0.

## Mission

`wow-service` is the only production layer allowed to coordinate multiple framework crates into one user-facing operation. It acquires one coherent immutable reference/project/analyzer/rule context, executes the selected use case, preserves exact capability and `NotEvaluated` state, constructs deterministic presentation/root-cause views without deleting raw findings, and returns one transport-independent result envelope.

E0-F proves two operations:

```text
status
    -> exact configured component/profile/generation/capability state
    -> no implication that analysis or tests passed

check
    -> one coherent reference + project + analyzer + rule context
    -> generic Emmy findings
    -> two E0 WoW rules
    -> structured clean/findings/NotEvaluated/failure outcomes
    -> deterministic root-cause presentation graph
    -> one canonical result envelope
```

A thin `apps/wow` CLI serializes these service results. It does not bypass service or reconstruct domain logic.

## Owned responsibilities

- service configuration and component registry;
- explicit project/profile/generation selection policy;
- coherent immutable context acquisition/lease;
- component identity and capability validation;
- `status` use case;
- `check` use case;
- generic finding collection from the selected project/analyzer snapshot;
- E0 rule registry/execution invocation;
- rule `Findings`, `EvaluatedClean`, `NotEvaluated`, `Failed`, and `Cancelled` aggregation;
- deterministic root-cause/causal presentation graph;
- raw-finding preservation;
- service-level semantic result status;
- canonical result-envelope construction and validation;
- operation budgets, cancellation, and partial-degradation policy;
- explicit deferred-operation reporting;
- transport-neutral request/response contracts;
- last-known-good component status without generation substitution;
- application-facing typed errors and health state;
- deterministic serialization handoff to `apps/wow`.

## Explicit non-responsibilities

`wow-service` does not:

- parse Lua, TOC, XML, or source text;
- implement reference ingestion, analyzer facts, project publication, or rule algorithms;
- perform source discovery, indexing, search ranking, lineage, graph traversal, or skeleton generation in E0;
- expose raw database, analyzer, actor, or mutable project handles;
- mutate source, editor configuration, component state, or the WoW client;
- execute analyzed Lua, repository hooks, build scripts, tests, or generators;
- silently select another profile/generation after mismatch;
- upgrade candidate/partial evidence;
- infer replacement/autofix/runtime behavior;
- discard raw generic/WoW findings during presentation folding;
- use message text as causal/dedup identity;
- treat component installation/readiness as a passing check;
- return empty/default success for deferred operations;
- own CLI parsing, stdout/stderr, or exit-code projection.

## Required reading

Before implementation, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`../wow-reference/CONTRACT.json`](../wow-reference/CONTRACT.json)
6. [`../wow-emmy/CONTRACT.json`](../wow-emmy/CONTRACT.json)
7. [`../wow-project/CONTRACT.json`](../wow-project/CONTRACT.json)
8. [`../wow-rules/CONTRACT.json`](../wow-rules/CONTRACT.json)
9. [`AGENTS.md`](AGENTS.md)
10. [`DECISIONS.md`](DECISIONS.md)
11. [`DATA_MODEL.md`](DATA_MODEL.md)
12. [`CONTEXT_ACQUISITION.md`](CONTEXT_ACQUISITION.md)
13. [`STATUS_OPERATION.md`](STATUS_OPERATION.md)
14. [`CHECK_OPERATION.md`](CHECK_OPERATION.md)
15. [`ROOT_CAUSE_FOLDING.md`](ROOT_CAUSE_FOLDING.md)
16. [`RESULT_ENVELOPE.md`](RESULT_ENVELOPE.md)
17. [`ERROR_MODEL.md`](ERROR_MODEL.md)
18. [`TEST_MATRIX.md`](TEST_MATRIX.md)
19. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
20. [`CONTRACT.json`](CONTRACT.json)
21. [`../../apps/wow/README.md`](../../apps/wow/README.md)
22. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

Normative repository sources:

- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/AGENT_WORKFLOW.md`](../../docs/AGENT_WORKFLOW.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## Direct dependencies in E0-F

```text
wow-core
wow-reference
wow-emmy
wow-project
wow-rules
```

All later production crates remain inactive:

```text
wow-store
wow-annotations
wow-graph
wow-recognizers
wow-search
wow-cbm
wow-context
```

The long-term maximum dependency graph does not authorize loading them in E0-F.

## E0 service configuration

```text
ServiceConfiguration
    service schema/version
    configured fixture project ID
    configured fixture ProfileIdentity / ReferenceGenerationId
    reference-view provider identity
    project-snapshot provider identity
    analyzer pin/probe/config identity
    rule-registry identity
    generation selection policy
    operation budgets
    cancellation policy
    canonical output schema/version
    deferred-operation registry
```

No field is inferred from the local WoW installation, editor, current Git branch, floating upstream branch, or environment credentials.

## Generation selection

E0 requests use an explicit selector:

```text
Exact(ProjectGenerationId)
CurrentPublished(ProjectId)
```

`CurrentPublished` is an operation selector scoped to one project registry, not a durable identity. The service atomically acquires one immutable snapshot and immediately records the exact selected `ProjectGenerationId` in the context/result.

Rules:

- no unscoped `latest`;
- no silent retry/switch after acquisition;
- exact selector mismatch fails;
- last-known-good cannot satisfy a request for another target generation;
- canonical result identity uses the exact selected generation, never the selector token.

Deterministic golden fixtures use `Exact` after IDs are frozen.

## Coherent service context

```text
ServiceContextLease
    service configuration identity
    selected ProfileIdentity
    ReferenceGenerationId / ReferenceView identity
    ProjectGenerationId / ProjectSnapshot / ProjectView identity
    AnalyzerSnapshot and accepted pin/config identity
    E0 RuleRegistry identity
    core schema/tool versions
    capability/coverage/conflict registries
    operation budget/cancellation state
```

All identities must agree before an operation runs. The lease is immutable for the request. E0 may implement it synchronously without an async runtime.

## `status`

`status` reports exact configured/available state:

```text
service/tool/schema versions
configured project/profile/reference identities
current published project generation and snapshot
accepted analyzer pin/probe/config/snapshot
active rule registry and rule rollout
component health
capability and coverage summaries
last-known-good identities and failed target identities separately
deferred operation/capability registry
operation budgets
```

It does not:

- run diagnostics;
- claim tests passed;
- convert Ready into clean;
- hide degraded/failed partitions;
- select a different profile/project;
- report unimplemented operations as available.

See [`STATUS_OPERATION.md`](STATUS_OPERATION.md).

## `check`

E0 sequence:

```text
validate CheckRequest and service configuration
-> acquire one coherent ServiceContextLease
-> validate reference/project/analyzer/rule identities and capabilities
-> collect generic findings from ProjectView for exact scope
-> invoke E0 RuleRegistry/Executor for exact scope
-> retain every rule outcome and NotEvaluated record
-> validate findings/evidence/source/generation
-> construct structured causal presentation graph
-> preserve raw findings unchanged
-> derive service semantic status
-> build/validate/canonicalize ResultEnvelope
-> hand transport-neutral result to apps/wow
```

No component is re-read against another generation during the request.

See [`CHECK_OPERATION.md`](CHECK_OPERATION.md).

## Service semantic status

```text
clean
    coherent complete requested scope; no raw findings; no NotEvaluated/failure/truncation

findings
    coherent complete requested scope; one or more raw findings; no blocking unavailable scopes

partial
    coherent useful result exists, but one or more requested rule/diagnostic/capability scopes are NotEvaluated, failed-degradable, or truncated; findings may also be present

failed
    request/context/mandatory component/internal result contract failed; no coherent check result

cancelled
    operation cancelled before result publication
```

Status is semantic and independent of CLI exit-code policy.

Precedence:

```text
failed
cancelled
partial
findings
clean
```

A result with findings plus `NotEvaluated` is `partial`, with findings retained.

## Raw findings and presentation graph

The result stores:

```text
raw_findings[]
    every generic and WoW finding unchanged

presentation_graph
    display_root_ids[]
    causal/blocked/duplicate relations[]
    child ordering
```

Folding never deletes raw findings or alters finding identities.

Allowed E0 relations derive only from structured provider/component evidence:

```text
causes_or_explains
blocked_by
exact_duplicate_of
```

Examples:

- authoritative `wow.api.exists` root can explain the exact same-source generic unresolved-member symptom when `wow-rules` supplies a proven causal hint;
- annotation-library failure blocks API/Secret rules and becomes the presented root for their `NotEvaluated` records;
- unrelated generic/Secret/API findings remain independent roots.

Message similarity is prohibited. See [`ROOT_CAUSE_FOLDING.md`](ROOT_CAUSE_FOLDING.md).

## Result envelope

One canonical envelope contains:

```text
operation and request identity
service semantic status
exact GenerationContext and component snapshot identities
selected scope
component health/capability summary
raw findings
presentation graph
rule outcomes and clean records
NotEvaluated records
warnings/failures when coherent partial result exists
budget/truncation/cancellation state
deferred operations/capabilities
schema/tool/producer versions
canonical ordering and digest
```

No timestamp, temp path, process/thread ID, memory address, rendered-text ordering, or local credential enters canonical identity.

See [`RESULT_ENVELOPE.md`](RESULT_ENVELOPE.md).

## E0 baseline check fixture

The full closed project scope is expected to contain, after prerequisite freeze:

```text
accepted generic Emmy fixture diagnostic: 1
wow.api.exists findings: 1
wow.secret.local_operation findings:
    unsafe_concat
    guard_after_use
    different_value_guard
wow.secret.local_operation clean evaluations:
    guarded_concat
```

A same-source generic unresolved-member symptom may be present only if the accepted upstream diagnostic mapping emits it; that optional family must be frozen by E0-C and folded only through an exact causal hint. The service contract does not invent it.

Expected full-scope status without blockers: `findings`.

## Deferred operations

E0 reports typed `operation_not_implemented_for_milestone` for:

```text
lookup
search
tree
skeleton
plan
patch_impact
index_repo
runtime_review
LSP
MCP
release/pack publication
```

`status` lists them as Deferred. They never return empty/default success.

## Last-known-good behavior

Service may report or allow an explicitly requested old published snapshot, but:

- it retains original generation/reference/analyzer identity;
- it is never substituted for a requested failed target;
- status separates current published, failed candidate/target, and last-known-good;
- check result records the exact snapshot actually acquired;
- no mixed old/new context.

## Cancellation and budgets

- validate cheap request/context/budget preconditions first;
- propagate cancellation to rule execution and bounded component reads;
- no result envelope published after cancellation;
- no background continuation;
- budget/truncation cannot become clean;
- partial result is explicit and includes completed/blocked scopes;
- E0 defaults to whole-request publication only after coherent result validation.

## E0-F hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No later crate activation.
- No parser/index/rule/search/domain algorithm in service.
- No source/editor/client mutation or analyzed-code execution.
- No hidden snapshot/profile retry.
- No raw finding deletion during folding.
- No message-text causal grouping.
- No `status = pass` from component readiness.
- No empty success for deferred operations.
- No transport/CLI parsing inside service.
- No timestamp/temp path in canonical output.
- No runtime/client validation claim.
- No CI.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- status result;
- clean check scope;
- complete findings check;
- partial check with blockers and retained findings;
- context failure;
- cancellation;
- root-cause presentation graph;
- canonical result and checksum freeze.

Actual prerequisite/component/snapshot/finding/outcome/graph/envelope IDs and SHA-256 values freeze after E0-A through E0-E implementations exist and before the first `wow-service` Rust commit.

## Definition of done

E0-F implementation is complete only when:

```text
status reports exact component/profile/generation/capability/deferred state without claiming checks passed
check acquires one coherent immutable context and never switches it
one full-scope check preserves generic + API + Secret raw findings and guarded clean outcome
all rule NotEvaluated/failure/cancellation states survive aggregation
root-cause presentation uses structured hints and preserves raw findings
service status precedence classifies clean/findings/partial/failed/cancelled exactly
one canonical ResultEnvelope validates and serializes byte-identically
last-known-good is never substituted/relabelled
all deferred operations fail explicitly
apps/wow consumes only service contracts and maps exact CLI exit behavior
all TEST_MATRIX cases pass
```

Until then, this directory remains an implementation-ready orchestration contract, not a running service.
