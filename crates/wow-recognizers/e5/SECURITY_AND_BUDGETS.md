# E5-A security, privacy, holdout, and resource budgets

**Status:** normative.

E5-A extends the E2-B matcher threat model to untrusted repositories, corpus metadata, labels, reviewers, split manifests, holdout artifacts, metrics, and candidate promotion evidence.

## Threat model

Inputs may attempt to:

- execute repository source, hooks, filters, workflows, generators, tests, package managers, installers, or addons;
- smuggle repository/addon/owner/path/popularity/label/split/reviewer/model fields into matcher semantics;
- expose sealed-holdout membership, labels, results, or evaluator secrets before candidate freeze;
- use copied/forked/vendor/generated examples to fake independent generalization;
- rewrite labels/splits/thresholds after failures without a new identity;
- convert Unknown/Conflict/NotEvaluated/Partial into favorable negatives;
- hide mandatory false positives through weighting or aggregate-only reports;
- leak raw private source, host paths, credentials, reviewer identity, or restricted notes;
- create pathological corpus size, joins, mutations, reports, graph proposals, or comparisons;
- exploit nondeterministic ordering, first/last match, cache, worker count, or storage layout;
- retain stale candidate partitions after disable/supersede.

## Execution boundary

`wow-recognizers` has no filesystem, network, Git, process, environment, editor, WoW client, database, model, or source-control API. Repository materialization and project/analyzer/graph publication occur through external owners before E5-A operations receive exact immutable artifacts.

Never execute:

```text
Lua/XML or source snippets
repository-local scripts or binaries
Git hooks/filters/submodules with code execution
GitHub Actions or other workflows
build/test/release tools
package managers or generators
installers/addons/WoW client
review notes or prompt-like text
```

## Metadata isolation

Audit/provenance fields are accepted only in nonsemantic manifests. Static validation traces data flow and rejects any route from:

```text
repository/provider/owner/addon/display/path/URL/branch/tag
stars/downloads/popularity
corpus/split/example/label/reviewer IDs or notes
expected outputs and evaluation results
search/model/embedding/community suggestions
```

into clause truth, captures, output keys/attributes, confidence, coverage, ordering, tie-breaking, budgets, or rollout.

## Holdout confidentiality

Sealed holdout artifacts use separate exact identities for:

- opaque membership manifest/digest;
- encrypted or access-controlled label/result material owned by later orchestration;
- candidate pack bytes and run request frozen before unsealing;
- evaluator access/audit records;
- consumed/contaminated generation history.

E5-A defines the semantic contract but stores no encryption key, credential, token, or reviewer secret. Missing E5-B authorization/unsealing infrastructure is `NotEvaluated`, never simulated as pass.

## License and privacy

Policies are evaluated per artifact class: raw source, excerpts, normalized facts, source handles, labels, mutations, pack rules, reports, graph proposals, fixtures, and publication candidates.

- Public visibility is not sufficient redistribution authorization.
- Private/local sources require explicit local-only profiles.
- Default errors/reports contain IDs, counts, bounded safe arguments, and digests, not raw source or private paths.
- Reviewer notes remain bounded untrusted data and are excluded from matcher input.
- Credentials, private keys, access tokens, SavedVariables contents, runtime payloads, and secrets are forbidden in fixtures.

## Resource profiles

Finite maxima exist for:

```text
candidate sources and admitted corpus members
source roots/inventory entries/publication refs
provenance graph nodes/edges/components
examples/labels/reviews/evidence refs
split groups and leakage candidates
packs/rules/clauses/captures/joins/outputs
fact snapshots and facts per partition
matches/ambiguity groups/proposals
mutations and before/after fact bytes
case results, metric dimensions, comparisons and explanations
report/artifact bytes
CPU/wall/memory/checkpoints/cancellation latency
```

Unlimited, negative, overflowing, host-derived, or candidate-selected limits are invalid. System maxima and profile compatibility are checked before work begins.

## Fanout and amplification

- provenance/near-duplicate closure is bounded and reports truncation;
- many-to-many joins require explicit E2-B caps;
- mutation expansion has per-family and total caps;
- graph proposals, evidence, metrics dimensions, and comparison matrices are capped;
- canonical early termination uses semantic order and reports Partial/Truncated;
- duplicate evidence cannot increase confidence, independent-group counts, or aggregate weight;
- minimum explanation/decision fields cannot be dropped merely to fit an output cap.

## Label and metric integrity

- expected labels are immutable run inputs;
- label/split/threshold changes create new identities;
- per-case results are canonical before aggregates;
- mandatory failures remain unweighted blockers;
- Unknown/Conflict/NotEvaluated/Partial/Truncated exclusions and denominators are explicit;
- a corpus member cannot count as multiple independent observations through files, calls, mutations, or copies;
- test/holdout result use is recorded as consumed/contaminated.

## Cancellation

Cancellation is checked during validation, provenance closure, split analysis, compilation, matching, graph validation, mutation generation/execution, case classification, aggregation, comparison, explanation, candidate-artifact construction, and deactivation validation.

Cancellation:

- starts no background work;
- emits no complete run, metric, candidate, or deactivation artifact;
- retains exact partial stage/budget state where policy permits;
- does not activate a previous result as current;
- does not reset cumulative budgets on explicit continuation by a future owner.

## Determinism as integrity

Equivalent logical inputs under shuffled facts/examples/evidence, 1/2/N workers, cold/warm caches, different temp roots, and independent materialization histories reaching identical logical publications produce identical semantic bytes. Host timing and peak-memory measurements are noncanonical benchmark fields.

Nondeterminism is a correctness/security failure because it can conceal first-match behavior, leakage, amplification, or mutable-label state.

## Security corpus

Required adversarial cases include:

- executable/include/template/plugin/regex/expression payloads;
- repository/path/prompt/label/reviewer metadata injection;
- holdout access before candidate freeze;
- fork/copy/vendor/generated leakage and unknown provenance;
- label/split/threshold rewrites after observation;
- Unknown-to-Negative and partial-coverage negative coercion;
- aggregate weighting that hides a mandatory FP;
- huge/deep corpus/pack/provenance/mutation/report inputs;
- duplicate-evidence and independent-count amplification;
- cross-generation/profile/universe identity collision;
- raw source/private path/token leakage;
- cancellation at every phase;
- worker/order/cache/storage nondeterminism;
- stale partition retention after disable.
