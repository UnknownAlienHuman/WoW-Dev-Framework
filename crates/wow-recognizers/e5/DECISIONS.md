# E5-A calibration decisions

**Status:** normative.

## CAL-001 — Candidate revision and admitted corpus are separate

A repository/commit pin is provenance only. Admission additionally requires exact materialization, fact publication, license/privacy, upstream-lineage, label and split closure.

## CAL-002 — Normalized facts are the only matcher input

Raw source may support label review but never becomes a second recognizer parser or runtime input.

## CAL-003 — Labels are independent artifacts

Expected labels cannot be copied from the candidate pack, current recognizer output, search rank, README prose or model output.

## CAL-004 — Universal semantics only

Named packs may name donors in metadata; rules and graph outputs cannot encode donor repository/addon/owner/path identity.

## CAL-005 — Same owner does not prove independent shape

Repositories under one current owner may have distinct origins, but independence is unknown until upstream/fork/copy lineage is reviewed.

## CAL-006 — Provenance groups define split atomicity

Forks, copies, vendored libraries, shared generated templates and near-duplicate implementations stay in one split group.

## CAL-007 — Holdout is sealed before tuning

Holdout membership/digest is frozen; labels/results are unavailable to pack authors until candidate-pack bytes are frozen.

## CAL-008 — Rerunning after holdout-driven edits creates a new evaluation generation

The old holdout result remains historical and cannot be described as untouched validation.

## CAL-009 — Unknown is not negative

Unknown, Possible, NotEvaluated, Partial, Conflict and Truncated labels/outcomes remain distinct.

## CAL-010 — Label conflicts are retained

Reviewer disagreement creates an explicit conflict/review state. No silent majority or last-write-wins resolution.

## CAL-011 — One example is not one independent observation

Multiple facts/calls/files/mutations from the same provenance group cannot inflate independent-generalization counts.

## CAL-012 — Mutations share provenance with the source example

Derived mutation cases remain in the same split group unless a separate challenge-set contract proves no tuning leakage.

## CAL-013 — Repository/name/path invariance is mandatory

Renaming repository, owner, addon metadata, directories and irrelevant local identifiers must not alter universal outputs except exact source identities.

## CAL-014 — Decisive semantic sensitivity is mandatory

Changing a required public convention literal, receiver, structural edge, producer, scope, resolution or coverage state must change/downgrade output exactly as declared.

## CAL-015 — Name/path invariance does not erase semantic literals

A literal is allowed only when the universal convention contract requires it. Its use must be documented and mutation-sensitive.

## CAL-016 — E5-A packs are shadow-only

Calibration candidates cannot satisfy default core coverage or publish default graph assertions.

## CAL-017 — Metrics eligibility is not promotion

Passing frozen E5-A gates may produce `PromotionEligibleByMetrics`; E5-B review/authorization and E5-C immutable rollout remain required.

## CAL-018 — No aggregate hides a hard failure

A mandatory false positive, authority upgrade, graph-invalid proposal, leakage, nondeterminism, security failure or named semantic condition blocks the candidate regardless of weighted average.

## CAL-019 — Per-case results are canonical

Aggregate reports derive from immutable case results; aggregate-only evaluation is invalid.

## CAL-020 — Corpus weighting is explicit and nonexculpatory

Weights can describe sampling/summary importance but cannot turn a failed mandatory example into pass.

## CAL-021 — Graph validation is independent

Recognizer success does not validate graph kinds, endpoints, semantic keys, attributes, evidence or confidence. Every proposal is independently checked by `wow-graph` fixtures/ports.

## CAL-022 — Search/context/lineage results are not labels

Higher-layer retrieval or history candidates cannot become calibration truth without independent exact review evidence.

## CAL-023 — No model in the correctness or labeling path

LLM/embedding output cannot choose labels, clauses, thresholds, splits, promotion or expected results.

## CAL-024 — Exact outputs remain Derived or Possible

Calibration metadata, repeated donors or reviewer confidence cannot make recognizer-produced graph proposals `Proven`.

## CAL-025 — Negative clauses retain E2 coverage requirements

Corpus knowledge that no match exists cannot bypass complete closed-scope fact coverage at runtime.

## CAL-026 — Pack identity includes schema, rules and evaluation bindings

Changing clauses, captures, outputs, confidence, ambiguity, coverage, budgets or bound evaluation profiles creates a new pack/version identity.

## CAL-027 — Corpus and pack identities do not form a cycle

Corpus/labels/splits exist independently; pack may bind corpus IDs; evaluation binds both; corpus identity never includes pack output.

## CAL-028 — Deactivation is partition-local

Removing a candidate pack deletes only its producer partitions/coverage and never changes core graph semantics or other packs.

## CAL-029 — Raw source redistribution is separately authorized

Public repository access does not automatically permit committed source fixtures or derived redistributable artifacts.

## CAL-030 — Performance cannot trade away correctness

A faster candidate that violates authority, precision, coverage, mutation, leakage or determinism gates is rejected.

## CAL-031 — Exact donor names are nonsemantic audit fields

Donor IDs can appear in reports/manifests for reproducibility but are excluded from match clause and graph-output inputs.

## CAL-032 — Generalization scope is explicit

A pack validated only on one provenance ecosystem must state that limited scope and cannot claim ecosystem-wide generalization.

## CAL-033 — No automatic core mutation

E5-A never edits `core-pack.json`, changes active core rule versions or adds default rollout state.

## CAL-034 — No current/floating corpus

Corpus members bind exact repository commits and exact owner publication/fact identities.

## CAL-035 — KB routes current platform interpretation, not recognizer labels

Patch-sensitive API/runtime guidance remains in exact Reference/runtime evidence and the external KB; it is not hard-coded as corpus truth by name.
