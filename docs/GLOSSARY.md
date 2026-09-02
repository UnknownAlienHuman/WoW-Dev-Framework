# Glossary

**Status:** normative terminology.

## Identity, evidence, and state

**Profile** — exact WoW target identified by flavor, Interface, build, source revision/digest, schema/tool versions, and correction set. `Current` is not a durable profile ID.

**Reference generation** — immutable identity of one built Reference Pack.

**Project generation** — immutable project snapshot published from coherent source/configuration inputs against one profile.

**External generation** — exact revision/index identity of an external source or candidate system.

**Capability** — named operation whose availability depends on explicit inputs and coverage.

**Coverage partition** — smallest source/analysis area whose completeness is reported independently.

**Negative authority** — permission to state absence. Requires exact scope/profile/generation, complete relevant coverage, and no unresolved conflict/truncation.

**Provenance** — origin class of evidence: platform source, project source, runtime probe, correction, oracle, external implementation, semantic candidate, or historical record.

**Confidence** — `Proven`, `Derived`, `Possible`, or `Candidate`.

**Stable source handle** — compact exact source identity including owner/revision/profile/path/span/digest and optional symbol/entity key.

**OutcomeUnknown** — durable state where an effect may have committed but its receipt cannot yet be reconciled. It blocks blind retry and is not failure, cancellation, no-change, or success.

**Idempotency key** — exact `OperationId + CanonicalRequestDigest` binding used to reconcile/resume effects without duplication.

**Retention receipt** — owner proof that an exact artifact/generation remains available for a result, continuation, review, audit, or submission.

## Graph, search, and context

**Entity** — typed graph node such as API symbol, function, package, frame, template, module, registry, state path, restriction facet, or source span.

**Relation** — typed directed edge with producer, evidence, confidence, generation, coverage, and conflicts.

**Parent axis** — one explicit hierarchy/view such as lexical, ownership, load, object, inheritance, registration, lifecycle, state, call, or lineage. There is no universal parent.

**Candidate** — item selected for investigation by exact-relative, fuzzy, text, shape, semantic, external, or search similarity. It is not a proven relation or replacement.

**Project Map** — deterministic compact typed projection of one exact project/graph generation.

**L0 skeleton** — bounded container/navigation structure without full implementation bodies.

**L1 skeleton** — bounded exact entity and local-neighborhood structure.

**L2 source** — exact source span/full source under an explicit privacy/license policy.

## Recognizers and calibration

**Recognizer** — deterministic declarative pattern over normalized Lua/TOC/XML facts that emits universal roles/relations.

**Core recognizer pack** — immutable reviewed universal pack eligible for an explicitly activated core execution profile.

**Calibration pack** — named declarative pack derived from corpus conventions. Its name is audit metadata; rules cannot branch on repository/addon/owner/path/popularity/split/label/reviewer/model identity.

**Calibration candidate source** — exact repository revision proposed for a corpus. A commit pin alone is not admission.

**Admitted calibration corpus** — immutable manifest whose members passed exact materialization/publication/fact, provenance, license/privacy, label, coverage, and split gates.

**Calibration expected label set** — immutable independent expected outputs/uncertainty/cardinality statement, never copied from candidate output.

**Calibration provenance group** — conservative connected group of forks, copies, vendored/generated code, near-duplicates, mutations, or shared authoring lineage; atomic unit for split independence.

**Calibration split manifest** — immutable whole-provenance-group assignment to `Train`, `Dev`, `Test`, `SealedHoldout`, `Challenge`, or `Quarantine`, with visibility/leakage history.

**Calibration pack candidate** — exact `trust_class=calibration`, `rollout_state=shadow_only` pack limited to universal `Derived`/`Possible` proposals.

**Calibration candidate artifact** — immutable E5-A evidence bundle containing exact inputs, run/case/mutation/metric/graph/security/deactivation results, blockers, and nonclaims.

**Calibration deactivation plan** — proof that disabling a candidate removes only its owned shadow partitions and reports exact coverage loss.

**PromotionEligibleByMetrics** — E5-A metric state only. It is not review authorization, submission, publication, activation, or runtime proof.

## Review and holdout

**Calibration review decision envelope** — strict exact-candidate decision input with structured decision, principal/role/scope references, authorization evidence, expiry/revocation/replay state, and digest.

**Review authorization** — independent security decision that a principal may submit a defined review decision within exact scope. It does not create semantic or graph proof.

**Calibration review record** — immutable E5-B record binding candidate, envelope, use-time authorization, independent candidate validation, decision, audit, and supersession links.

**Sealed holdout** — evaluation generation whose membership/labels remain inaccessible to candidate authors until exact candidate/run/evaluator/profile identities freeze and separate access is authorized.

**Holdout authorization** — independent permission to access one exact sealed generation for one frozen candidate/run/disclosure purpose. Reviewer authorization does not imply it.

**Holdout access grant** — immutable exact authorization record plus nonsecret vault-scope handle, use/expiry/revocation/replay policy, and permitted visibility.

**Holdout access audit** — append-only hash-linked record of requests, grants/denials, open, evaluation, disclosure, failure, cancellation, revocation, replay, and consumption.

**Holdout disclosure class** — `AggregateGateOnly`, `PerCaseClassificationWithoutHiddenInputs`, `BoundedReviewedEvidence`, or `FullReviewedDisclosure`.

**Holdout consumption record** — exact determination of whether disclosed/evaluated holdout evidence can influence a candidate lineage.

**ContaminationUnknown** — state where nonaccess/noninfluence cannot be proved. It cannot be represented as untouched.

## Promotion and publication

**Promotion submission** — immutable E5-B artifact binding exact candidate evidence, authorized reviews, holdout audit/consumption, blockers, license/privacy, deactivation, target profile, and nonclaims. It requests E5-C consideration but is not publication.

**CorePackArtifact** — future E5-C distinct immutable reviewed core-pack artifact created only after independent submission/candidate/pack revalidation.

**Core-pack publication** — future E5-C immutable catalog effect placing a validated core pack in `PublishedInactive` or another explicit publication state. It is distinct from activation.

**Activation** — future guarded selection of an exact published pack for a defined execution profile/current record.

**Canary** — future bounded exact activation cohort/profile used to gather explicit rollout evidence; it is not global runtime proof.

**Last-known-good** — exact retained previously validated publication explicitly designated under a policy. It is never inferred as previous/newest or relabeled as a failed target.

## Restrictions

**Restriction facet** — open versioned metadata describing Secret, protected, combat, hardware-event, forbidden-object, private-partition, or related constraints.

**Secret Value** — WoW runtime value with restricted accessibility/operations. Static nominal types are analysis projections, not runtime wrappers.

**Root-cause folding** — deterministic grouping of downstream diagnostics under a causal failure while preserving raw findings.

**Universe** — separated source domain such as Reference Pack, first-party project, dependency, Blizzard UI source, calibration corpus, external candidate, runtime, or history.
