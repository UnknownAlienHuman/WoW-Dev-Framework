# E4-B input producers and proof ceilings

**Status:** normative cross-crate producer contract.

## Producer classes

```text
project_stable_identity
project_source_fingerprint
project_structural_change
reference_explicit_transition
reference_deprecation_or_replacement
search_lineage_candidate
review_decision
```

Each class has a versioned schema, exact generation scope, evidence requirements, confidence ceiling, partition key, coverage contract and security limits.

## Project stable identity producer

Owner: `wow-project`.

Eligible inputs are exact owner-controlled identities preserved across before/after publications, such as a stable semantic symbol/entity identifier or explicit project identity transition recorded by the project pipeline.

Required fields:

- exact before/after project and graph generations;
- exact entity refs;
- identity schema/profile/version;
- source/declaration/fact origins;
- producer coverage and conflicts;
- proof that the identity did not arise from name/path/insertion order alone.

Maximum ceiling can be `Proven` for `lineage_successor_of` only under the reviewed profile and complete exact identity evidence.

It does not automatically prove move, rename, replacement or migration.

## Project source fingerprint producer

Owner: `wow-project`.

Produces bounded typed fingerprints/features over exact published source/semantic facts. Examples can include canonical syntax/semantic signatures, declaration neighborhood, source-map-stable feature sets and relation summaries.

Hard rules:

- fingerprint schema/version/profile and source facts are exact;
- no raw model embedding or opaque hash without explainable feature manifest;
- repository/path/name/popularity are not hidden semantic features;
- identical fingerprint is Candidate evidence only;
- dynamic/unknown/unsupported/lossy regions are explicit;
- copied/vendor/generated/repeated code hard negatives are required;
- no source execution.

Maximum lineage ceiling by itself: `Candidate`.

## Project structural change producer

Owner: `wow-project`.

Provides exact before/after differences for already-associated owner records, or bounded proposal evidence such as:

```text
source file/container/package/load position changed
canonical declared name changed
signature/type/source span changed
exact relation set changed
entity present/absent under scoped complete inventory
```

A structural difference does not establish the pair's lineage by itself. It supports typed `ChangeRecord` classification only after an accepted lineage/replacement relation, except scoped presence/absence inputs used for removal/introduction authority.

Maximum ceiling depends on the exact input and relation; presence/absence still requires graph-level coverage closure.

## Reference explicit transition producer

Owner: `wow-reference`.

Provides exact ReferenceView comparison records derived from authoritative source/correction contracts, such as:

```text
explicit rename/move/transition key
exact deprecation target
explicit replacement relation
availability/introduced/removed transition
signature/type/restriction facet change
reviewed correction transition
```

Requirements:

- exact before/after ReferenceProfile and ReferenceGeneration;
- exact source/raw/correction/evidence records;
- correction status and conflict state;
- exact coverage and negative-authority decision where applicable;
- no fuzzy alias/replacement inference.

Can support `Proven`/`Derived` according to the relation registry and exact Reference authority. Scope never extends beyond the bound profiles.

## Reference deprecation/replacement producer

Owner: `wow-reference`.

Specialized explicit transition records for `deprecated_by`, `replaced_by` and migration evidence. It must distinguish:

- deprecation without replacement;
- replacement target explicitly named;
- replacement target unresolved/conflicted;
- candidate recommendation not authoritative;
- source versus reviewed correction origin.

Replacement does not imply same lineage or automatic edit compatibility.

## Search lineage candidate producer

Owner: `wow-search` E4-A supporting seam.

Input consists of exact before/after SearchShard bindings and E4-A `SearchCandidateSignal`/candidate explanations for a typed lineage-candidate query profile.

Permitted evidence:

- exact-name/explicit-alias/member similarity as search signals;
- text/fuzzy/shape matches;
- source fingerprint fields already supplied by owner projections;
- bounded graph-neighborhood candidate paths;
- all rank contributions, caps, conflicts, skipped lanes and coverage.

Hard ceiling: `Candidate` for every lineage/replacement/migration relation, regardless of E4-A authority band or top rank.

Search does not:

- accept/reject/promote lineage;
- emit `Removed`/`Introduced`;
- create Reference transitions;
- call `wow-graph` publication directly;
- query current generations;
- hide competing candidates.

## Review decision producer

Owner: E4-B review application/service boundary, coordinated later by `wow-service` E4-C; validated/stored by `wow-graph`.

A review decision adds explicit provenance and a structured decision over exact proposals/components. It cannot exceed:

```text
min(
    reviewer authority ceiling,
    relation/profile ceiling,
    strongest independently supported input ceiling,
    coverage/conflict ceiling
)
```

A review can reject or defer any proposal. It cannot promote Candidate-only search/fingerprint evidence to `Proven` when the selected profile requires exact owner/Reference proof.

Review notes are bounded untrusted data and never alter proof rules.

## Producer partition keys

Conceptual key:

```text
producer class
producer ID/version
LineageUniverseSetId
before/after generation pair
entity kind or comparison partition
rule/profile ID
```

Updating/disabling one producer replaces only its partition. Accepted assertions/change records depending on removed inputs are recomputed or invalidated; other producers remain intact.

## Evidence composition

Evidence composition is conjunctive and explicit, not a vote. A deterministic rule may combine:

```text
owner stable identity
+ exact before/after field values
+ complete scope coverage
-> Derived rename/move/change assertion
```

But:

```text
same name
+ high search rank
+ similar body
+ similar graph neighborhood
```

remains Candidate without the independent qualifying evidence required by the relation profile.

## Proof ceiling evaluation

For every proposal/assertion, compute and retain:

```text
producer maximum
relation-kind maximum
input-evidence maximum
review authority maximum, if any
coverage/conflict/truncation maximum
effective maximum
requested confidence
effective confidence
blocking reasons
```

Any missing capability or unresolved conflict lowers the effective ceiling or makes the proposal `NotEvaluated` according to the profile.

## Provenance separation

Do not relabel:

- project source as Reference platform authority;
- Reference transition as project implementation continuity;
- search signal as owner identity;
- review note as source evidence;
- implementation source as runtime behavior;
- external/model candidate as reviewed exact fact.

Accepted assertions cite every contributing evidence class separately.

## Input security and limits

Bound:

- producer partitions/records/entities/features/evidence refs;
- fingerprint dimensions and bytes;
- candidate pairs/component size;
- search explanation size;
- review decisions/notes;
- source spans/quotes;
- recursion/derivation depth;
- time/memory/output/cancellation.

Reject executable code, callbacks, plugins, raw SQL, model prompts, arbitrary paths/URLs or source-controlled proof rules.
