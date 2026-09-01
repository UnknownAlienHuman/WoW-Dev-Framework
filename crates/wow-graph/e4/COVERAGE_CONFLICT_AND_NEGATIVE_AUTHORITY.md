# E4-B coverage, conflicts, omissions, and negative authority

**Status:** normative.

## Authority preservation

E4-B keeps independent authority classes:

```text
Reference Pack public contract
Blizzard UI implementation source
user project source/use
owner stable-ID/transition record
graph assertion and deterministic derivation
E4-A retrieval candidate signal
runtime observation
community/external/model candidate
```

No aggregation, ranking, review preference, or rendering silently upgrades one class into another.

## Coverage dimensions

Track at least:

```text
generation endpoint materialization/admission
owner store/read snapshot acquisition
entity/relation inventory
source decoding and coordinate mapping
ReferenceView fact/alias/transition/negative authority
project analyzer and recognizer facts
Blizzard UI structural facts
stable-ID and explicit transition records
structural fingerprint extraction
E4-A candidate generation/import
candidate component enumeration
matching/ambiguity solving
review decisions
lineage assertion validation/publication
change facet comparison
removal/introduction evaluation
migration evidence/applicability/recipe generation
static-impact lane/traversal/path enumeration
store read-back/integrity
pagination/continuation
runtime validation
privacy/license/source detail
```

A complete store write or matching pass cannot repair incomplete owner/source/reference coverage.

## Coverage record scope

Each record states:

- exact generation pair/endpoints;
- capability and narrow partition;
- producer/profile/version;
- expected/observed/omitted/failed/truncated units;
- state and reason;
- evidence/source/conflict refs;
- whether negative authority is permitted for that exact scope;
- canonical digest.

Broad “lineage complete” flags are forbidden.

## Proposal proof and coverage

A proposal can reach its lane ceiling only when every required decisive capability is complete for its exact relation schema. Missing optional evidence can lower confidence or remain an omission; missing decisive evidence produces Candidate/Possible/NotEvaluated.

Examples:

- stable owner ID equality without verified namespace/profile identity -> NotEvaluated;
- structural continuity with one source file unparsed -> Possible/NotEvaluated;
- explicit transition record with conflicting endpoint kind -> conflict;
- E4-A candidate under partial target shard -> Candidate with partial coverage;
- removal while a decisive package/source partition is missing -> no authoritative removal.

## Conflict classes

```text
GenerationDirectionConflict
EndpointUniverseOrProfileConflict
StableIdentityConflict
ExplicitTransitionConflict
RelationKindOrCardinalityConflict
MultipleEqualLineageSolutions
SplitMergeInterpretationConflict
OwnerFactConflict
ReferenceSourceAuthorityConflict
CoverageAssertionConflict
ReviewDecisionConflict
ChangeFacetConflict
MigrationTargetOrApplicabilityConflict
StaticImpactPathConflict
StorePublicationOrReadBackConflict
PrivacyLicenseConflict
```

Conflicts link exact competing records and affected capabilities/facets. They are not reduced to one message or preferred source.

## Conflict behavior

- incompatible explicit transitions block automatic acceptance;
- stable-ID conflict cannot be overridden by fuzzy/search score;
- equal valid matching solutions remain ambiguity/conflict;
- project source and ReferenceView disagreement remains separate authority evidence;
- change facets depending on a conflicted lineage relation are conflicted/NotEvaluated;
- migration recipes depending on a conflicted target are CandidateOnly/NotEvaluated;
- static-impact paths crossing a conflicted edge are ConflictBlocked;
- unrelated capabilities/partitions can remain available.

## Omission records

Every known excluded/deferred item that can affect interpretation has a typed record:

```text
ProfileExcluded
IncompatibleUniverseOrKind
ProofCeilingInsufficient
CandidateFiltered
DuplicateEvidenceRetainedElsewhere
OptionalOwnerUnavailable
InputPartialOrFailed
ConflictBlocked
PrivacyOrLicenseDenied
BudgetPruned
ComponentNotFullyEnumerated
ContinuationDeferred
RuntimeEvidenceRequired
DeferredFeatureOrProvider
CancelledBeforeEvaluation
```

An omitted candidate is not a rejected lineage assertion unless rejection criteria were evaluated.

## Ambiguity versus conflict

```text
Ambiguity
    more than one semantically valid solution because decisive evidence is insufficient

Conflict
    records/constraints cannot all be true under the active contract
```

Both remain explicit. Lexical order cannot convert ambiguity into a unique solution.

## Negative authority classes

### Entity absent in one generation

Requires exact owner inventory/read coverage for the entity kind/scope and owner-supported negative semantics.

### No lineage target/source

Requires:

- all allowed high-ceiling evidence lanes complete;
- candidate component fully enumerated within profile;
- no unresolved competing proposal/ambiguity/conflict;
- exact opposite-generation inventory complete;
- no profile/build mismatch;
- no truncation/cancellation.

### Removed/introduced

Requires entity absence plus no accepted continuity and complete relation-specific gates. Unpaired does not equal removed/introduced.

### No replacement

Requires complete explicit transition/replacement record coverage for the capability/scope. Failure to find a similar target is not proof that no replacement exists.

### No static impact

Requires complete target graph/project/reference coverage for every active impact lane, full bounded traversal of the declared finite scope, no conflict/candidate blocker, and profile-authorized negative semantics.

An empty path set normally yields:

```text
NoImpactPathFoundUnderExecutedLanes
```

not global unaffected truth.

## Candidate-only outcomes

A result can validly contain only candidates. It must state:

- exact candidate producers/lanes;
- proof ceilings;
- missing discriminators;
- owner/search coverage;
- ambiguity groups;
- which assertions/change/migration/impact conclusions are unavailable;
- exact review/additional evidence routes.

Candidate-only is not failure and not proof.

## Partial and truncated outcomes

Keep separate:

```text
PartialInput
PartialLaneCoverage
PartialMatchingComponent
TruncatedCandidateEnumeration
TruncatedChangeFacetComparison
TruncatedImpactTraversal
ContinuationAvailable
Cancelled
NotEvaluated
Failed
```

A partial artifact never becomes complete after rendering or service wrapping.

## Counts and summaries

Every summary states exact scope, selected/known/unknown/conflicted/truncated counts, and underlying manifest. Examples:

```text
accepted lineage assertions
candidate proposals
ambiguous components
removed-with-authority
unpaired-under-partial-coverage
static direct/derived/possible/conflict-blocked/not-evaluated paths
```

Do not combine these into a misleading “matched” or “affected” total.

## Runtime boundary

Static source/reference/project completeness does not establish:

- actual client execution;
- event delivery/payload accessibility;
- Secret Value state;
- taint/combat/protected legality;
- performance;
- user-visible failure;
- successful migration.

Runtime evidence requirements remain separate omissions/validation steps.

## Validation

- no authority/confidence upgrade;
- all required coverage records scope correctly;
- negative decisions satisfy exact gates;
- conflicts/ambiguities preserved;
- omissions reconcile with candidate/endpoint/path manifests;
- partial/truncated/cancelled state propagates;
- unrelated capabilities remain usable where safe;
- deterministic summaries under shuffled input.
