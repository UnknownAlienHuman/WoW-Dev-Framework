# `wow-rules` implementation contract

**Status:** E0-E implementation-ready contract; no Rust code yet. Only two closed fixture rules activate in E0.

## Mission

`wow-rules` implements deterministic, capability-declared World of Warcraft diagnostic providers over immutable project/analyzer facts and exact reference facts. It joins independent project and platform evidence, evaluates one narrowly defined rule, and emits evidence-bearing findings or explicit `NotEvaluated` records.

The crate does not parse, index, persist, search, mutate source, or decide which project/reference snapshot is current. It receives one coherent execution context assembled later by `wow-service`.

## E0-E outcome

A future implementation agent must prove two vertical rules:

```text
wow.api.exists@1
    project unresolved member/call fact + exact project span
    + E0-B ReferenceView exact lookup
    + complete authoritative negative coverage
    -> one project-located finding

wow.secret.local_operation@1
    project producer/binding/use/operation/guard/control-flow facts
    + E0-B secret.return facet
    + closed E0 guard-semantics fixture
    + complete required capabilities
    -> finding | evaluated-clean | NotEvaluated
```

No other planned rule family is implemented in E0-E.

## Owned responsibilities

- stable rule/provider IDs and versions;
- rule descriptors and rollout metadata;
- rule capability requirements;
- immutable rule execution context validation;
- applicable-rule selection;
- deterministic provider execution;
- rule-specific evidence joins;
- rule evaluation outcomes: findings, clean, `NotEvaluated`, failed;
- normalized finding construction through `wow-core`;
- root-cause keys and deterministic causal relation hints;
- remediation-tier classification;
- per-rule coverage/evaluation reports;
- fixture corpus and false-blocking measurement inputs;
- exact E0 rule algorithms and non-goals.

## Explicit non-responsibilities

`wow-rules` does not:

- parse Lua, TOC, XML, JSON, or upstream analyzer objects;
- mutate `wow-emmy`, `wow-project`, reference, graph, or source state;
- open files/databases/networks/processes/editors/client data;
- derive project/reference generations;
- select a profile or current snapshot;
- execute analyzed Lua or repository code;
- rank general search results;
- infer aliases, replacements, migrations, or autofixes from similarity;
- fold the final cross-rule/service finding stream;
- suppress generic diagnostics by message text;
- claim runtime/client/combat behavior;
- implement graph-dependent rules in E0;
- return clean success when capabilities are partial, failed, conflicted, stale, or missing.

## Required reading

Before implementation, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`../wow-reference/CONTRACT.json`](../wow-reference/CONTRACT.json)
6. [`../wow-reference/LOOKUP_AND_COVERAGE.md`](../wow-reference/LOOKUP_AND_COVERAGE.md)
7. [`../wow-emmy/FACT_MODEL.md`](../wow-emmy/FACT_MODEL.md)
8. [`../wow-project/CONTRACT.json`](../wow-project/CONTRACT.json)
9. [`AGENTS.md`](AGENTS.md)
10. [`DECISIONS.md`](DECISIONS.md)
11. [`DATA_MODEL.md`](DATA_MODEL.md)
12. [`PROVIDER_EXECUTION.md`](PROVIDER_EXECUTION.md)
13. [`CAPABILITY_AND_COVERAGE.md`](CAPABILITY_AND_COVERAGE.md)
14. [`API_EXISTS_RULE.md`](API_EXISTS_RULE.md)
15. [`SECRET_LOCAL_RULE.md`](SECRET_LOCAL_RULE.md)
16. [`FINDING_AND_REMEDIATION.md`](FINDING_AND_REMEDIATION.md)
17. [`ERROR_MODEL.md`](ERROR_MODEL.md)
18. [`TEST_MATRIX.md`](TEST_MATRIX.md)
19. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
20. [`CONTRACT.json`](CONTRACT.json)
21. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

Normative repository sources:

- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- current Secret/event/hook guidance in the external knowledge base.

## Direct dependencies in E0-E

```text
wow-core
wow-reference
wow-emmy
wow-project
```

The long-term graph permits `wow-graph`, but that edge remains inactive in E0-E. If a rule needs graph facts, it is not an E0 rule.

## Rule descriptor

Every provider declares:

```text
stable RuleId and version
semantic category
technical severity
rollout policy: shadow | advisory | blocking
profile/flavor applicability
accepted source/entity scopes
required capabilities and partitions
required fact/lookup kinds
execution and output budgets
root-cause and remediation policy
fixture/evaluation set
```

A descriptor is configuration/contract data, not permission to run when capabilities are unavailable.

## Rule execution context

E0 context contains exactly:

```text
one ProfileIdentity / ReferenceGenerationId
one immutable ReferenceView
one immutable ProjectSnapshot / ProjectView
one ProjectGenerationId
one accepted AnalyzerSnapshot identity
normalized analyzer fact sets and generic findings for selected files
rule fixture policy for E0-only guard semantics
exact coverage/conflict records
execution budget and cancellation state
```

All context identities must agree. Cross-generation/profile input is rejected before provider execution.

## Rule evaluation outcomes

```text
RuleEvaluationOutcome
    Findings(Finding[])
    EvaluatedClean(CleanEvaluationRecord)
    NotEvaluated(NotEvaluatedRecord)
    Failed(RuleFailure)
    Cancelled
```

`EvaluatedClean` is valid only when every required capability/partition is usable and the rule examined its declared scope. An empty findings list alone is not clean.

## E0 rule descriptors

### `wow.api.exists@1`

```text
severity: error
rollout: advisory during E0 evaluation
remediation: plan_only
scope: direct unresolved member/call references in Main project source
reference lane: exact lookup only
```

Required high-level behavior:

- project unresolved member/call fact identifies `C_E0Fixture.RemovedApi` and exact project span;
- exact reference lookup returns `authoritative_absent` under complete unconflicted coverage;
- emit one finding at project member/reference span;
- include project evidence plus exact reference coverage/authority decision;
- no replacement, alias, fuzzy search, or edit;
- partial/conflict/profile mismatch/library failure -> `NotEvaluated`, not finding/clean.

See [`API_EXISTS_RULE.md`](API_EXISTS_RULE.md).

### `wow.secret.local_operation@1`

```text
severity: error
rollout: advisory during E0 evaluation
remediation: plan_only
scope: one function-local producer -> binding -> direct concatenation use
reference lane: exact producer facet lookup
```

Required high-level behavior:

- exact producer call resolves to `C_E0Fixture.SecretText`;
- reference lookup returns unconflicted fixture `secret.return` on return 1;
- analyzer facts bind the returned value to a local and exact concatenation operation;
- accepted E0 guard semantics and proven dominance for the same value make the case clean;
- no guard, guard after use, or guard on another value -> finding;
- facet/control-flow/guard/reference capability partial/conflict -> `NotEvaluated`;
- copy/conversion does not declassify;
- no runtime or broad interprocedural claim.

See [`SECRET_LOCAL_RULE.md`](SECRET_LOCAL_RULE.md).

## Evidence separation

### API rule

```text
project evidence
    unresolved member/call source facts and project SourceHandle

reference authority inputs
    exact query result, coverage IDs, conflict IDs, generation context

rule derivation evidence
    provider version and deterministic join inputs
```

No absent symbol source handle is fabricated.

### Secret rule

```text
project evidence
    producer call, binding, use, operation, guard/control-flow source facts

reference evidence
    exact secret.return facet and its raw/source/coverage evidence

rule derivation evidence
    provider version, fixture guard policy, exact value/operation relation
```

Project and reference evidence retain independent provenance.

## Capability gating

Before a provider runs:

1. validate profile/reference/project/analyzer generation coherence;
2. select exact required capabilities/partitions;
3. inspect partial/failed/unknown/conflict/truncation blockers;
4. build `NotEvaluated` with exact missing/blocking inputs when any requirement is unavailable;
5. run the provider only over immutable facts/lookups;
6. report examined scope and output budget.

No provider implements a local weaker coverage boolean.

## Root-cause semantics

Providers emit deterministic structured root-cause keys and optional causal relation hints. Final folding remains `wow-service` responsibility.

Examples:

```text
api absence finding can be the root for a same-span generic unresolved-member symptom
reference profile unavailable blocks the API rule
restriction facet conflict blocks the Secret rule
annotation library failure blocks resolution-dependent rules
control-flow capability failure blocks guard evaluation
```

Message-text similarity is never causal proof.

## Remediation

E0 findings are `plan_only`.

- API absence: confirm selected profile and locate a proven current contract; no replacement proposed.
- Secret local operation: restructure evaluation/use around an accepted access contract or native sink after project/runtime review; no automatic guard insertion.

No E0 provider emits an edit. See [`FINDING_AND_REMEDIATION.md`](FINDING_AND_REMEDIATION.md).

## E0-E hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No graph dependency/rules.
- No additional planned rule family.
- No IO/network/process/editor/client access.
- No source mutation or autofix application.
- No alias/fuzzy/semantic/replacement lane.
- No static permanent spell whitelist.
- No conversion/copy/serialization/`pcall` declassification.
- No rule clean result under partial/conflicted/missing capability.
- No generic diagnostic suppression/folding in this crate.
- No runtime/client/combat validation claim.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- provider registry/descriptors and E0 guard fixture policy;
- API exact-found/authoritative-absent/partial/conflict/profile/library cases;
- Secret unsafe/guarded/after-use/different-value/copy/conversion/partial/conflict cases;
- provider selection, clean, findings, `NotEvaluated`, failure, cancellation, and deterministic ordering;
- pending prerequisite IDs and byte/checksum freeze.

Actual profile/reference/project/analyzer/fact/evidence/finding IDs and SHA-256 values are frozen after E0-A through E0-D implementation exists and before the first `wow-rules` Rust commit.

## Definition of done

E0-E implementation is complete only when:

```text
both rule descriptors validate and are the only active E0 providers
all context generations/profile identities match
wow.api.exists emits exactly one finding only for authoritative exact absence
partial/conflict/profile/library states produce NotEvaluated
wow.secret.local_operation distinguishes unsafe, dominating guard, after-use guard, and different-value guard
copy/conversion never declassifies
all findings preserve separate project/reference/derivation evidence
clean outcomes prove evaluated scope and complete capabilities
root-cause keys are deterministic and message-independent
all E0 remediation is plan_only and no edit is emitted
randomized provider/fact/coverage order yields byte-identical outputs
all TEST_MATRIX cases pass
```

Until then, this directory remains an implementation-ready rule contract, not a functioning diagnostic engine.
