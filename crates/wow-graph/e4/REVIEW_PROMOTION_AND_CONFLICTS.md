# E4-B review, promotion, and conflict model

**Status:** normative.

## Purpose

Some lineage components cannot be resolved by deterministic owner evidence alone. Review is an explicit, auditable producer input; it is not an unrestricted authority override.

## Review target

A `LineageReviewDecision` targets exactly one of:

```text
LineageProposal
LineageCandidateComponent
proposed assignment set within one component
existing LineageAssertion
GenerationAbsenceDecision candidate
MigrationCandidate or MigrationRecipe
LineageConflictRecord
```

It binds the exact `LineageUniverseSet`, profile set, producer versions and target digests. A review against another generation/profile is invalid.

## Decision states

```text
Accept
Reject
Defer
Conflict
Supersede
```

`Accept` authorizes only the exact target relation/assignment and requested/effective proof ceiling. It does not silently accept neighboring candidates or all future generations.

`Reject` does not prove the inverse relation or absence. `Defer` preserves unresolved status. `Conflict` records incompatible review/evidence. `Supersede` links a new decision to the exact prior decision without deleting history.

## Reviewer authority classes

A reviewed profile may define bounded classes such as:

```text
CorpusAnnotation
ProjectOwnerReview
FrameworkMaintainerReview
ReferenceCorrectionReview
ReleaseValidationReview
```

Each class has:

- eligible relation/change/migration targets;
- maximum proof ceiling;
- required independent input evidence;
- required second review or consensus where applicable;
- privacy/audit/retention requirements;
- expiration/recheck conditions;
- prohibited transitions.

A class label is not self-asserted by arbitrary input. The service/application identity and authorization mechanism are E4-C/E7 concerns; `wow-graph` validates a typed attested decision against the configured profile.

## Effective promotion ceiling

```text
effective ceiling = minimum(
    relation-kind ceiling,
    producer/input-evidence ceiling,
    reviewer-class ceiling,
    comparison compatibility ceiling,
    coverage/conflict/truncation ceiling
)
```

Review cannot:

- promote Candidate-only search/fingerprint evidence to `Proven` when exact owner/reference evidence is required;
- convert source implementation evidence into Reference platform authority;
- convert static structure into runtime proof;
- hide partial coverage or unresolved component alternatives;
- create a replacement/migration relation not allowed by the relation registry;
- declare removal/introduction without closed negative authority;
- relabel a prior generation/entity/assertion.

## Automatic deterministic promotion

Manual review is not required when a reviewed deterministic rule has exact sufficient inputs and no ambiguity/conflict, for example owner-stable identity continuity.

Automatic promotion records:

```text
rule ID/version
exact input proposal/evidence IDs
proof-ceiling calculation
coverage/conflict checks
accepted assertion ID
```

It is still a producer decision and is independently validated.

## Review notes

Optional notes are bounded untrusted data:

- no schema/profile/proof instruction effect;
- no source code execution;
- no Markdown/tool/prompt authority;
- no credentials/private source by default;
- separate digest/privacy/retention policy;
- not part of the relation proof unless an exact structured evidence record is separately attached.

## Accepted assertion construction

An accepted `LineageAssertion` includes:

- exact source/target generation entity refs;
- relation kind/version and comparison scope;
- every supporting proposal/evidence/producer partition;
- proof-ceiling arithmetic;
- review decision/rule records;
- coverage/conflicts and remaining limitations;
- publication generation and canonical digest.

Acceptance never mutates the proposal.

## Rejection and retention

Rejected/deferred proposals remain stored while required by:

- active assertion explanation;
- ambiguity/conflict history;
- review audit;
- evaluation corpus;
- migration/impact result;
- continuation;
- explicit retention/debug policy.

A later proposal/review uses a new ID and links predecessors. No in-place status rewrite under the same digest.

## Conflict classes

```text
CompetingExclusiveLineage
StableIdentityCollision
OneToOneMultiplicityViolation
SplitMergeAssignmentConflict
CopyMoveConflict
LineageReplacementSemanticConflict
RemovalContinuityConflict
IntroductionContinuityConflict
ChangeFacetConflict
ReferenceTransitionConflict
ReviewDecisionConflict
ProducerSchemaOrVersionConflict
CrossUniverseOrGenerationConflict
CoverageVersusConclusionConflict
ProofCeilingViolation
MigrationPreconditionConflict
ImpactPathEvidenceConflict
```

Every conflict names exact records and affected capabilities.

## Conflict behavior

- retain all sides;
- prevent last-write/majority/popularity resolution;
- cap or block accepted assertion/change/removal/migration/impact outputs according to profile;
- expose typed resolution requirements;
- preserve independent unaffected relations/capabilities;
- produce deterministic conflict IDs/order.

A conflict in one change facet does not automatically invalidate unrelated exact facets unless the relation/profile says the root identity itself is unresolved.

## Stable identity collisions

If an owner-stable identity key maps to multiple before or after entities:

- emit a `StableIdentityCollision`;
- do not accept all as same lineage;
- require owner correction/review under the exact profile;
- block removal/introduction decisions in the affected scope;
- preserve the conflicting owner evidence.

## Review disagreement

Multiple reviews do not vote by count. The profile defines whether:

- one class supersedes another;
- two independent accepts are required;
- any disagreement creates conflict;
- a decision expires after input/profile change;
- release-validation review is required for a migration recipe.

Every applied rule is explicit and deterministic.

## Removal/introduction review

Review cannot compensate for incomplete source/index coverage by asserting “not found.” It may attach exact external evidence only if the relation/negative-authority profile recognizes that evidence class and all scope requirements are satisfied. Otherwise the result remains unmatched/NotEvaluated.

## Migration review

A reviewed migration recipe must validate:

- exact source/target contract identity;
- replacement/deprecation/lineage evidence;
- applicability constraints;
- transformation semantics;
- forbidden cases;
- required static/client/runtime validation;
- version/profile scope;
- coverage/conflicts;
- advisory tier.

Review does not execute or prove the edit.

## Determinism and security

Review application order cannot change the final accepted set. Sort by exact target, authority class, decision precedence/profile and decision ID; conflicting decisions remain conflicts.

Reject:

- missing/tampered attestation/profile;
- target digest mismatch;
- unknown reviewer class;
- proof request above ceiling;
- source/search text used as review authorization;
- private data leak;
- executable note/payload;
- unbounded batch/note/evidence size;
- review against nonretained inputs.
