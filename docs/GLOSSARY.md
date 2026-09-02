# Glossary

**Status:** normative terminology.

## Core identity and evidence

**Profile** — exact target binding flavor/build/source/tool/schema versions; `Current` is not a durable profile ID.

**Capability** — named operation with explicit prerequisites and coverage.

**Negative authority** — permission to state absence; requires exact scope/generation, complete relevant coverage, and no blocker.

**Provenance** — origin class of evidence.

**Confidence** — `Proven`, `Derived`, `Possible`, or `Candidate`.

**OutcomeUnknown** — durable state where an effect may have committed but cannot yet be reconciled; blind retry is forbidden.

**Idempotency key** — exact `OperationId + CanonicalRequestDigest`.

**Retention receipt** — owner proof that an exact artifact/generation remains available.

## Graph/search/context

**Entity** — typed graph node.

**Relation** — typed directed edge with producer/evidence/confidence/generation/coverage/conflicts.

**Candidate** — item selected for investigation; it is not proof.

**Project Map** — compact deterministic projection of one project/graph generation.

**L0/L1/L2** — container structure / exact entity neighborhood / exact source under policy.

## Calibration/publication

**Calibration candidate artifact** — immutable E5-A candidate evidence bundle.

**Promotion submission** — immutable E5-B request for E5-C consideration; not publication.

**CorePackArtifact** — distinct immutable E5-C production-trust artifact; not a relabelled candidate.

**Detached signature envelope** — signature over exact digests plus nonsecret key/profile/authorization/verification references; not semantic proof.

**PublishedInactive** — immutable catalog publication that has not changed current/default.

**Canary evaluation** — exact scoped per-signal conclusion; not global runtime proof.

**Last-known-good designation** — explicit authorized retained qualification; never inferred as previous/newest.

**Rollback receipt** — immutable exact pointer/reindex/closure record; rollback does not rewrite history.

## E6 external candidates

**External candidate provider descriptor** — repository-owned immutable maximum contract for one provider adapter: allowed operations/schemas/state/score/locator/privacy/limits. Runtime negotiation cannot widen it.

**Provider capability set** — exact intersection of reviewed descriptor and one runtime transport/session observation.

**StableExternalGeneration** — immutable provider generation/index/corpus identity with sufficient exact receipt for the declared replay/cache claims.

**ObservedMutableGeneration** — one mutable provider state episode bound to an exact session/observation receipt; later calls are not assumed identical.

**OpaqueExternalState** — provider state without stable/observed identity; explicitly nonreproducible with restricted cache/continuation claims.

**ExternalCandidateQuery** — closed bounded provider-scoped candidate request with no raw SQL/MCP/tool/script/model surface.

**ExternalCandidateResultSet** — immutable provider/state/query-bound candidates, zero/partial/truncation/failure, coverage/loss/conflict, continuation, and Candidate authority.

**ExternalCandidate** — normalized provider result with `provenance=semantic_candidate`, `confidence=Candidate`, provider-local rank/score, untrusted fields, and no negative authority.

**UnverifiedProviderLocator** — provider-supplied repository/path/URI/revision/symbol/span/digest data not yet mapped to a project/reference owner record.

**External locator mapping request candidate** — E6-A handoff containing exact candidate/locator fields for future E6-B owner mapping; it is not a source handle.

**ZeroCandidatesReported** — provider reported no candidates for one exact request/state under reported coverage; not global absence.

**ZeroCandidatesAfterValidationLoss** — provider returned items but none survived bridge validation; distinct from provider-reported zero.

**Provider-local score** — score/rank meaningful only under one exact provider/descriptor/profile; not framework confidence and not comparable across providers by default.

**External candidate cache entry** — exact descriptor/state/query/profile-bound cached result preserving original staleness, coverage, loss, and Candidate authority.

**Optional degradation** — provider failure disables only the external lane while exact local/reference/project/search/context/diagnostic capabilities remain unaffected.

## Restrictions

**Restriction facet** — versioned metadata for Secret/protected/combat/hardware-event/forbidden/private constraints.

**Secret Value** — WoW runtime value with restricted accessibility/operations; static nominal types are not runtime wrappers.

**Universe** — separated domain such as Reference Pack, first-party project, Blizzard UI source, calibration corpus, external candidate, runtime, or history.