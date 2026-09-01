# E5-A public operation contracts

**Status:** normative operation behavior. Concrete Rust APIs may differ only with same-change contract and fixture updates.

E5-A operations are library operations over exact immutable artifacts. They do not clone repositories, read floating branches, execute source, publish graph state, authorize promotion, mutate the E2 core pack, or expose a service/CLI surface.

## Common outcome

```text
CalibrationOperationOutcome
    operation and exact request digest
    pack/corpus/split/run/artifact IDs when applicable
    status
    validation/errors/blockers
    coverage/conflicts/omissions
    resource/cancellation state
    canonical result digest
```

Common statuses:

```text
Complete
NoChange
Partial
CandidateOnly
ConflictBlocked
NotEvaluated
Cancelled
Failed
```

No empty/default success exists.

## `validate_calibration_candidate_source`

Input: one `CalibrationCandidateSource`, exact materialization/publication manifests available to the caller, and admission/security/license profiles.

Checks:

- exact immutable repository revision and tree/content identity;
- bounded declared roots and complete source-inventory accounting;
- exact project/analyzer/graph/fact publication binding;
- provenance/upstream/fork/copy grouping evidence;
- license, notice, privacy, and redistribution decisions per artifact class;
- absence of floating current/latest inputs and executed-source paths;
- canonical identity and digest closure.

Output is a validation report and explicit admission blockers. Validation alone does not admit the source.

## `validate_calibration_corpus_manifest`

Input: exact corpus manifest, candidate-source reports, example/label/provenance manifests, and profile bundle.

Validates identity DAG closure, exact member resolution, scope/generalization declarations, coverage/conflicts/omissions, license/privacy compatibility, and absence of pack-output cycles.

## `admit_calibration_corpus`

Input: a valid corpus candidate plus all mandatory admission reports.

Behavior:

- evaluates each member independently;
- retains Quarantine/Rejected members and reasons outside admitted membership;
- creates a new immutable admitted corpus identity;
- never mutates an earlier corpus generation or silently inherits labels across revisions.

A Git commit pin without fact, provenance, label, and license closure remains `CandidateCorpusInput`.

## `build_calibration_fact_snapshot`

Input: exact admitted examples and exact `RecognizerFactBundle` references supplied by owner publications.

Behavior:

- validates fact schema/profile/generation/scope and source/evidence handles;
- projects only matcher-eligible normalized facts;
- excludes repository/addon/owner/path/popularity/label/split/reviewer metadata from matcher fields;
- emits one immutable fact-snapshot manifest with complete/partial/conflict state.

It never reparses Lua/TOC/XML or reads raw source as fallback.

## `validate_expected_label_set`

Input: exact example, expected universal label set, graph registry/profile, evidence, coverage, and reviewer records.

Checks output kind/relation/key/attributes, confidence ceiling, decisive evidence, Negative closed-scope authority, Possible/Unknown/NotEvaluated/Conflict distinctions, reviewer independence, visibility policy, and canonical bytes.

It does not resolve reviewer authorization for promotion; durable authorization is E5-B.

## `validate_calibration_split_manifest`

Input: admitted corpus, conservative provenance-group graph, split manifest, and visibility/sealing profile.

Checks:

- connected provenance components do not cross Train/Dev/Test/SealedHoldout;
- mutation children remain with their source group unless a reviewed challenge profile says otherwise;
- holdout member/label manifests are sealed before candidate construction;
- consumed test/holdout generations are recorded;
- Unknown grouping blocks unsupported independence claims;
- split identity is explicit, deterministic, and immutable.

## `validate_calibration_pack_candidate`

Input: exact candidate pack bytes plus E2-B pack/fact/graph profiles and E5-A corpus/split/evaluation bindings.

Behavior:

1. delegates bounded declarative schema validation to the E2-B pack validator;
2. validates `trust_class = calibration` and `rollout_state = shadow_only`;
3. audits all data flow from named/audit metadata into clauses, captures, outputs, confidence, ordering, coverage, and budgets;
4. validates universal graph outputs and justified exact convention literals;
5. verifies rule-evidence, near-miss, mutation, and generalization-scope closure.

No core/default rollout fields are accepted.

## `run_calibration_pack_shadow`

Input: valid candidate pack, exact visible corpus split(s), exact fact snapshots, E2-B matcher implementation/profile, graph validation port/profile, and finite run budgets.

Behavior:

- compiles and executes only through the E2-B matcher;
- emits exact pack-owned shadow output partitions;
- retains all competing matches, ambiguity, coverage, conflicts, partial/truncated/cancelled state;
- independently validates every graph proposal;
- records immutable per-case results before aggregates.

It never publishes into the default project graph or uses hidden holdout labels.

## `run_calibration_mutation_suite`

Input: exact mutation suite, source examples, candidate pack, fact snapshots, and expected semantic-delta records.

Runs invariance, sensitivity, leakage, named-condition, adversarial-resource, cancellation, and determinism mutations. A hard-gate failure is explicit and cannot be averaged away.

## `evaluate_calibration_pack`

Input: exact case-result manifest, split/provenance/label profiles, graph/mutation/security/determinism reports, and frozen thresholds.

Produces per-case, per-rule, per-role/relation, per-split, per-provenance-group, and per-shape metrics with explicit denominators. Unknown/NotEvaluated/Conflict/Partial/Truncated are reported separately.

The operation may return `ShadowValidated` or `PromotionEligibleByMetrics`; neither is promotion or activation.

## `compare_calibration_runs`

Input: two exact compatible runs and a comparison profile.

Compares pack/corpus/split/implementation/profile identities, per-case semantic deltas, graph acceptance, hard-gate state, metrics, resources, and contamination history.

Incompatible identities yield a structured non-comparable result; the operation never normalizes away changed labels, splits, thresholds, or consumed holdouts.

## `explain_calibration_case`

Input: exact run and case-result ID.

Returns expected labels, observed matches/proposals, decisive facts, captures, evidence, graph validation, mutation relationship, coverage/conflicts, classification, blockers, and nonclaims. Raw restricted source is returned only through an independently authorized source-handle path outside this crate.

## `build_calibration_candidate_artifact`

Input: exact frozen pack, corpus, split, run, case, metric, anti-overfitting, graph-validation, license/provenance, and deactivation artifacts.

Builds an immutable candidate artifact only after all required identities and hard gates close. A non-promotion shadow artifact may retain explicit missing quantitative thresholds; a promotion-eligible artifact may not.

The artifact is an E5-B review input, not a core pack.

## `validate_calibration_deactivation_plan`

Input: exact pack/rule/input-generation partitions, shadow outputs, retained references, and expected coverage downgrade.

Proves that disabling, rejecting, quarantining, or superseding the candidate removes only its owned shadow partitions and stale references, leaves core/foreign partitions byte-identical, and reports the precise coverage loss.

## Common guarantees

- exact immutable identity and no symbolic current/latest;
- deterministic canonical semantic outputs for equivalent logical inputs;
- typed bounded errors and explicit recovery class;
- no hidden source, filesystem, network, process, editor, client, model, search, or service access;
- no source execution, graph publication, promotion authorization, runtime claim, or background work;
- no confidence, provenance, coverage, negative-authority, or generalization upgrade beyond exact evidence;
- cancellation publishes no complete run/candidate/deactivation artifact.
