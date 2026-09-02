# Glossary

**Status:** normative terminology.

## Identity, evidence, and state

**Profile** — exact WoW target identified by flavor, Interface, build, source revision/digest, schema/tool versions, and correction set. `Current` is not a durable profile ID.

**Reference generation** — immutable identity of one built Reference Pack.

**Project generation** — immutable project snapshot published from coherent source/configuration inputs against one profile.

**Capability** — named operation with explicit prerequisites and coverage.

**Coverage partition** — smallest source/analysis area whose completeness is independently reported.

**Negative authority** — permission to state absence; requires exact scope/profile/generation, complete relevant coverage, and no unresolved conflict/truncation.

**Provenance** — origin class of evidence.

**Confidence** — `Proven`, `Derived`, `Possible`, or `Candidate`.

**OutcomeUnknown** — durable state where an effect may have committed but its receipt cannot yet be reconciled. It blocks blind retry.

**Idempotency key** — exact `OperationId + CanonicalRequestDigest` binding.

**Retention receipt** — owner proof that an exact artifact/generation remains available.

## Graph, search, and context

**Entity** — typed graph node such as API symbol, function, package, frame, template, module, registry, state path, restriction facet, or source span.

**Relation** — typed directed edge with producer, evidence, confidence, generation, coverage, and conflicts.

**Parent axis** — one explicit hierarchy/view such as lexical, ownership, load, object, inheritance, registration, lifecycle, state, call, or lineage; there is no universal parent.

**Candidate** — item selected for investigation by exact-relative, fuzzy, text, shape, semantic, external, or search similarity; it is not proof.

**Project Map** — deterministic compact typed projection of one exact project/graph generation.

**L0 skeleton** — bounded container/navigation structure without full bodies.

**L1 skeleton** — bounded exact entity and local-neighborhood structure.

**L2 source** — exact source span/full source under explicit privacy/license policy.

## Recognizers and calibration

**Recognizer** — deterministic declarative pattern over normalized Lua/TOC/XML facts that emits universal roles/relations.

**Core recognizer pack** — immutable reviewed universal pack eligible for an explicitly activated core execution profile.

**Calibration pack** — named shadow-only declarative pack; its name/donor metadata cannot control matcher semantics.

**Calibration candidate source** — exact repository revision proposed for a corpus. A commit pin alone is not admission.

**Admitted calibration corpus** — immutable manifest whose members passed materialization/publication/fact, provenance, license/privacy, label, coverage, and split gates.

**Calibration provenance group** — conservative connected group of forks, copies, vendored/generated code, near-duplicates, mutations, or shared lineage; atomic split unit.

**Calibration split manifest** — immutable assignment of provenance groups to Train/Dev/Test/SealedHoldout/Challenge/Quarantine with visibility/leakage history.

**Calibration candidate artifact** — immutable E5-A evidence bundle containing exact inputs, case/mutation/metric/graph/security/deactivation results, blockers, and nonclaims.

**PromotionEligibleByMetrics** — metric state only; not review authorization, submission, publication, activation, or runtime proof.

## Review and holdout

**Review authorization** — independent security decision that a principal may submit one defined review decision within exact scope; it does not create semantic proof.

**Calibration review record** — immutable E5-B record binding candidate, strict review envelope, use-time authorization, independent validation, decision, audit, and supersession.

**Sealed holdout** — evaluation generation whose hidden membership/labels remain inaccessible until exact candidate/run/evaluator/profile identities freeze and separate access is authorized.

**Holdout authorization** — separate permission to access one exact sealed generation for one frozen candidate/run/disclosure purpose.

**Holdout access audit** — append-only hash-linked record of request, grant/denial, open, evaluation, disclosure, failure, cancellation, revocation, replay, and consumption.

**ContaminationUnknown** — state where nonaccess/noninfluence cannot be proved; it cannot be represented as untouched.

**Promotion submission** — immutable E5-B artifact binding exact candidate evidence, authorized reviews, holdout audit/consumption, blockers, license/privacy, deactivation, target profile, and nonclaims; it is not publication.

## Publication lifecycle

**Submission revalidation report** — E5-C independent exact validation of one PromotionSubmission and all required referenced evidence before artifact build.

**CorePackArtifact** — distinct immutable E5-C production-trust artifact containing exact universal rule/operator/schema/profile bytes, producer namespace, compatibility, lineage, deactivation/closure plans, blockers, and nonclaims. It is not a relabeled calibration candidate.

**CorePackAttestationSet** — exact provenance, build/reproducibility, SBOM, toolchain/dependency, license, notice, and privacy evidence for a CorePackArtifact.

**Detached signature envelope** — signature over exact domain-separated artifact/attestation digests with nonsecret key/trust-root references, authorization and verification state. A valid signature is not semantic/runtime proof.

**Core-pack publication** — immutable internal catalog record binding exact artifact, attestations, signatures, store/profile, validation, retention, and audit. It is separate from activation and public distribution.

**PublishedInactive** — publication state after immutable catalog/object commit but before activation; current/default execution remains unchanged.

**ValidatedInactive** — inactive publication that passed fresh exact read-back and required owner validation. It is still not active.

**Canary cohort plan** — exact bounded population/membership or privacy-preserving commitment, execution profile, observation schemas, denominators, criteria, authorization, budgets, and retention for one publication.

**Canary observation** — registered typed scoped evidence for one exact publication/cohort/project/profile/window with source adapter, coverage, conflicts, privacy, and audit.

**Canary evaluation** — per-signal-first conclusion (`Pass`, `Fail`, `Pause`, `InsufficientEvidence`, `Conflict`, or `Cancelled`) for an exact observation manifest; it is not global runtime proof.

**Rollout plan** — finite ordered activation stages with exact prior state, cohorts, required evidence, authorization, budgets, and stop/pause/rollback criteria.

**CurrentCorePackRecord** — profile-specific guarded current/default selection of one exact publication, updated by compare-and-swap.

**Last-known-good designation** — explicit authorized retained qualification of one exact publication for one execution profile. It is never inferred as previous/newest.

**Rollback receipt** — immutable record of an exact current-record change to an exact retained qualified target plus reindex/partition closure, audit, and retention state. Rollback does not rewrite history.

**Revocation record** — exact scoped declaration that an artifact/publication/signature/profile is no longer eligible under a policy, with evidence, authority, and required actions.

**Partition closure report** — proof that new project/graph generations contain expected target pack partitions, omit stale pack partitions, preserve foreign/historical partitions, and expose coverage changes.

## External semantic candidates

**External candidate provider descriptor** — repository-owned immutable maximum contract for one provider adapter: allowed operations, schemas, state classes, score/locator semantics, privacy/license handling, and limits. Runtime negotiation cannot widen it.

**Provider capability set** — exact intersection of a reviewed descriptor and one runtime transport/session observation.

**StableExternalGeneration** — immutable provider generation/index/corpus identity with sufficient exact receipt for the declared replay/cache claims.

**ObservedMutableGeneration** — one mutable provider-state episode bound to an exact session/observation receipt; later calls are not assumed identical.

**OpaqueExternalState** — provider state without stable or observed identity; explicitly nonreproducible with restricted cache/continuation claims.

**ExternalCandidateQuery** — closed bounded provider-scoped candidate request with no raw SQL, arbitrary MCP/tool call, executable, or model-prompt surface.

**ExternalCandidateResultSet** — immutable provider/state/query-bound candidate set with zero/partial/truncation/failure, coverage/loss/conflict, continuation, and Candidate authority.

**ExternalCandidate** — normalized provider result with `provenance=semantic_candidate`, `confidence=Candidate`, provider-local rank/score, untrusted fields, and no negative authority.

**UnverifiedProviderLocator** — provider-supplied repository/path/URI/revision/symbol/span/digest data not yet mapped to a project/reference owner record.

**External locator mapping request candidate** — E6-A handoff containing exact candidate/locator fields for later E6-B owner mapping; it is not a source handle.

**ZeroCandidatesReported** — provider reported no candidates for one exact request/state under reported coverage; not global absence.

**ZeroCandidatesAfterValidationLoss** — provider returned items but none survived bridge validation; distinct from provider-reported zero.

**Provider-local score** — score/rank meaningful only under one exact provider/descriptor/profile; not framework confidence and not numerically comparable across providers by default.

**External candidate cache entry** — exact descriptor/state/query/profile-bound cached result preserving original staleness, coverage, loss, and Candidate authority.

**Optional degradation** — provider failure disables only the external lane while exact local/reference/project/search/context/diagnostic capabilities remain unaffected.

## E6-B orchestration and mapping

**ExternalProviderConfiguration** — exact reviewed service configuration binding one provider descriptor/adapter, allowed profiles/operations, nonsecret authorization-reference class, session policy, privacy/license policy, and hard limits.

**ExternalProviderUseAuthorization** — nonsecret receipt authorizing use of one exact provider configuration and authorization reference for one operation/query purpose. It does not contain secret material and does not create semantic authority.

**ExternalProviderSessionBinding** — exact session/transport/capability/external-state/reconciliation/cancellation/close receipt returned by a host adapter. It exposes no raw credential, command, process, endpoint, or database handle.

**ExternalCandidateQueryOperation** — durable E6-B operation binding `OperationId + CanonicalRequestDigest`, configuration/session/state/query/profiles, dispatch/result/reconciliation state, and effect receipts.

**ExternalCandidateResultCatalogRecord** — immutable service/store record binding one validated E6-A result set to exact provider/state/query/profile, Candidate authority, privacy/license, retention, and read-back validation.

**ExternalLocatorMappingRecord** — immutable project/reference owner result for one exact locator and owner generation. Status is `ExactMapped`, `MultipleMappings`, `NoMappingWithOwnerAuthority`, `NoMappingPartial`, `Conflict`, `NotEvaluated`, or `Failed`.

**ExactMapped** — one exact owner record matched every required locator identity field under the selected mapping profile. It proves locator identity only, not provider interpretation.

**NoMappingWithOwnerAuthority** — the exact owner proved complete relevant coverage and no matching record. It does not prove the provider concept or broader source is absent.

**ExternalCandidateSelectionReceipt** — immutable caller-supplied `Selected`, `Rejected`, or `Deferred` decision for one exact candidate and mapping. Selection is not verification, acceptance, edit authorization, or promotion.

**Exact mapped root** — stable project/reference owner handle taken from an `ExactMapped` record and used as the unchanged root of an existing E3 context operation.

**ExternalCandidateSidecar** — separate service/application artifact carrying permitted provider Candidate fields plus mapping/selection references. It is not part of `ContextSemanticPack` truth.

**ExternalCandidateContextResult** — combined service record referencing normal exact context artifacts and a separate external Candidate sidecar, with independent lane/coverage/privacy/license/retention/closure state.

**Provider operation reconciliation** — exact adapter/store/owner recovery of a possibly committed provider/query/result/mapping/selection/context effect. Unresolved state remains `OutcomeUnknown` and unsafe to retry.

## Restrictions

**Restriction facet** — open versioned metadata describing Secret, protected, combat, hardware-event, forbidden-object, private-partition, or related constraints.

**Secret Value** — WoW runtime value with restricted accessibility/operations; static nominal types are analysis projections, not runtime wrappers.

**Universe** — separated source domain such as Reference Pack, first-party project, Blizzard UI source, calibration corpus, external candidate, runtime, or history.