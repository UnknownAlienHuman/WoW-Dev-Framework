# E4-C result envelopes, statuses, and validation payloads

**Status:** normative.

## Envelope principle

One public service call returns one canonical `E4ServiceResultEnvelope`. The envelope preserves owner semantics and adds only exact selector/acquisition/lifecycle/selection metadata.

It does not flatten all operations into a generic success/message pair.

## Canonical status set

```text
Complete
NoChange
Partial
CandidateOnly
ConflictBlocked
Truncated
NotEvaluated
Cancelled
Failed
```

Validation operations additionally include:

```text
Valid
Invalid
NotEvaluated
```

A completed invalid validation can have outer status `Complete` with `validation_state = Invalid` when the validation itself ran completely. Infrastructure or capability failure remains `Failed`/`NotEvaluated`.

## Conservative folding precedence

When multiple owner stages contribute state, fold in this order:

```text
Failed
Cancelled
NotEvaluated
ConflictBlocked
Truncated
Partial
CandidateOnly
NoChange
Complete
```

Exceptions are operation-specific and must be explicit. For example, a fully executed search may return `Complete` with `miss_class = NoCandidatesUnderExecutedLanes`; a fully executed lineage build with only Candidate proposals reports `CandidateOnly` rather than failure.

No warning can override a higher-precedence blocker.

## Required envelope fields

```text
schema and operation family/version
exact public request and normalized request IDs
exact resolved selector/owner/shard/lineage/context generation IDs
operation-specific owner result/artifact IDs
service status and validation state
miss/candidate/ambiguity/review/change/migration/impact classes
proof ceiling/confidence/provenance/evidence summary refs
coverage/conflict/omission/truncation summaries
budget and continuation/retention state
idempotency/response-loss receipt when applicable
resource closure report
privacy/license/consumer/authorization result refs
explicit nonclaims
canonical digest
```

## Operation payload families

```text
SearchIndexOutcome
SearchQueryOutcome
SearchExplanationOutcome
SearchSelectionOutcome
SearchContextOutcome
LineageBuildOutcome
LineageValidationOutcome
LineageReviewOutcome
LineageQueryOutcome
MigrationOutcome
StaticImpactOutcome
ServiceStatusOutcome
ServiceFailureOutcome
ServiceCancelledOutcome
```

The envelope uses a tagged union. Unknown tags or missing required operation fields are invalid.

## Search results

Preserve:

- exact SearchUniverseSet/shards;
- normalized query and lane results;
- ordered candidate IDs and result-set manifest;
- candidate explanations/authority bands;
- miss class;
- owner coverage/conflicts/omissions/budgets;
- continuation.

Do not summarize approximate candidates as exact matches or replacements.

## Selection/context results

`SearchSelectionOutcome` includes the exact receipt and root token. `SearchContextOutcome` contains both the receipt and the untouched E3-C context result reference/payload.

Search and context statuses remain separately visible. Combined status is conservative; a complete selection with partial context is `Partial`.

## Lineage/review results

Preserve producer partition, component, proposal, review, assertion, change, absence, conflict, proof-ceiling, coverage and snapshot records.

A service text field cannot say “renamed,” “removed” or “replaced” unless the exact E4-B owner record has that class and proof state.

## Migration results

Preserve:

- exact candidate/recipe IDs;
- governing assertions and proof ceilings;
- applicability/preconditions/steps/postconditions;
- unsupported/conflicted/NotEvaluated state;
- required future validation;
- explicit `not_applied` and `runtime_not_verified` flags.

`ValidatedRecipe` is not an applied migration.

## Impact results

Preserve direct affected records and reason paths separately. Every path includes exact edge/assertion/source/evidence refs and original confidence.

Required nonclaims:

```text
static-only
runtime-breakage-not-established
severity-not-assigned
performance-cost-not-measured
combat-taint-secret-safety-not-established
fixability-not-established
```

## Failures

```text
ServiceFailureOutcome
    error code/stage/operation
    exact resolved identities available before failure
    owner error refs and recovery class
    durable artifact/idempotency receipt refs if any
    resource closure state
    coverage/partial data only when safe and explicitly classified
```

Default errors exclude source bodies, raw query/review text, private paths, secrets, keys, signatures, raw database/storage handles and stack dumps.

## Cancellation

Cancellation is distinct from failure. A partial owner artifact may be referenced only when the owner contract permits it and the envelope remains `Cancelled`; it cannot be cached or rendered as complete.

## NoChange

`NoChange` is valid only when an explicit owner operation proves that the exact target logical artifact already exists and matches the request. Empty output, no search candidates, no lineage assertions or skipped work is not `NoChange` by convenience.

## CandidateOnly

`CandidateOnly` means the relevant operation completed but produced no conclusion above Candidate for the requested facet. It is not `Partial` merely because candidates are uncertain, though incomplete inputs can additionally force `Partial`/`NotEvaluated` under precedence.

## Canonicalization

- stable field/tag/order profile;
- owner payload is referenced or embedded without semantic rewriting;
- operational timing/retry/terminal fields excluded from semantic digest;
- exact byte serialization frozen before implementation;
- one final JSON document; CLI newline is application framing, not service identity.

## Validation

Validate tag/status compatibility, owner result identity, exact selector closure, proof/confidence preservation, coverage/conflict/omission reconciliation, mandatory nonclaims, continuation retention, closure completion, privacy redaction and canonical bytes.
