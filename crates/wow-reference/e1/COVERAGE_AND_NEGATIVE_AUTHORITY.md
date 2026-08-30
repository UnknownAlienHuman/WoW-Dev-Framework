# Coverage, conflicts, and negative authority

**Status:** normative E1-B reference completeness and exact absence contract.

A ReferenceView may know many true facts while remaining unable to assert that another fact is absent. Coverage, conflict, profile, generation, truncation, and runtime gaps remain independent.

## 1. Capability families

Initial reference capabilities, versioned/narrowed as needed:

```text
snapshot.identity
source.files
apidoc.registrations
apidoc.systems
apidoc.callables
apidoc.signatures
apidoc.tables_structures
apidoc.events
apidoc.enums
apidoc.cvars
apidoc.widgets_script_objects
apidoc.predicates
restriction.facets
restriction.raw_metadata
deprecation.explicit
transition.explicit
raw.metadata
raw.unknown_fields
corrections
reference.cross_links
reference.persistent_store
reference.exact_lookup
reference.raw_lookup
reference.source_handles
```

Do not create a single `reference.complete` flag as authority.

## 2. Partition dimensions

Coverage partition can combine:

```text
profile/reference generation
source snapshot/partition/file
system/namespace
entity kind/entity/member
raw field family/path
signature/event/table member family
restriction/predicate facet family
correction set/application
parser/evaluator/normalizer stage
store write/read/validation operation
```

Partition must be narrow enough to keep unaffected facts usable and broad enough to prove declared input completeness.

## 3. Coverage statuses

Use core statuses:

```text
Complete
Partial
Unknown
Failed
NotApplicable
NotEvaluated
```

Examples:

- file parsed/evaluated fully: Complete for exact declared evaluator capability;
- one registration has unsupported helper: Partial for dependent system/entity/field capability;
- profile lacks optional CVar source by design: NotApplicable for that selected profile contract;
- parser fatal file error: Failed for file and dependent partitions;
- rule/query skipped because restriction raw unknown facet: NotEvaluated;
- no manifest completeness record: Unknown.

## 4. Stage dependencies

Coverage dependency graph conceptually:

```text
snapshot/file identity
-> parser
-> evaluator registration/raw values
-> field registry/normalizer
-> correction applications
-> cross-reference/conflict resolution
-> persistent store write/validation/read
-> ReferenceView query capability
```

A downstream capability cannot be stronger than required upstream dependencies. Store success does not upgrade partial ingestion.

## 5. Partition completeness criteria

Each `InputPartitionManifest` declares:

```text
required file/registration set/count/digests
allowed optional/ignored/unexpected inputs
parser/evaluator/normalizer success criteria
unknown/unsupported tolerance by capability
correction requirements
cross-reference/conflict resolution requirements
persistent write/read validation requirements
```

“Processed all files found” is not completeness unless the expected declared set is itself complete/verified.

## 6. Unknown fields

Unknown field handling must answer:

- raw value preserved?
- parent entity/source known?
- field may affect identity/signature/restriction/applicability/predicate/deprecation?
- dependent capability list?
- can unaffected fields remain Complete?
- release eligibility impact?

Default for an unknown restriction/signature/applicability field is conservative dependent downgrade, not ignore.

## 7. Unsupported constructs

Unsupported record links exact dependents. Propagation examples:

```text
unsupported unrelated local outside registrations
    -> file diagnostic; capability impact per parser/evaluator proof

unsupported value inside one optional documentation prose field
    -> raw/docs field Partial; signature may remain Complete if independent

unsupported helper producing a whole registration table
    -> registration/system/entity kinds Partial/Failed

fatal parse of required generated APIDoc file
    -> file and all dependent declared systems/capabilities Failed/Partial
```

Policy is explicit and tested; never optimism by default.

## 8. Conflicts

Conflict can exist under Complete ingestion:

```text
duplicate different source values
source partitions disagree
correction contradicts current source
ambiguous type/entity reference
explicit transition target unresolved
normalizer invariant collision
```

Coverage records can remain Complete for ingestion while `ReferenceConflictRecord` blocks authority for affected facts/queries.

## 9. Truncation and budgets

Any truncated input/raw/fact/correction/store/query output records:

```text
stage
partition/scope
processed/omitted counts/bytes/work units
reason/budget ID
continuation/detail handle if supported
```

Truncation affecting declared completeness blocks negative authority and release eligibility for relevant scope. A bounded list query may truncate presentation without invalidating underlying store coverage, but result cannot imply full listing.

## 10. Staleness

Reference generation is immutable, so it is not stale relative to itself. “Stale” applies when caller asks for another target/current build/profile or when hotfix/runtime data freshness is required.

- exact query against selected old profile can be authoritative for that profile;
- caller expecting current live but selects old profile is request/configuration issue, not source mutation;
- runtime/hotfix-sensitive facets can require a freshness/runtime evidence contract and remain static NotEvaluated for current behavior.

## 11. Negative authority inputs

```text
NegativeAuthorityRequest
    exact ReferenceView/ProfileId/ReferenceGenerationId
    normalized exact entity/query key and kind
    relevant capability families/partitions
    selected scope/system/namespace
    requested absence claim kind
```

Absence claim kinds are narrow:

```text
entity-not-present
member-not-present-on-exact-entity
field-not-present-in-exact-normalized-contract
deprecation-or-transition-not-recorded-in-exact-selected-input
```

Do not answer broader “safe/current replacement/never exists” claims through this operation.

## 12. Authority decision

Authoritative `yes` only when all hold:

1. exact view/profile/reference generation valid;
2. query normalized and entity kind/scope known;
3. all declared relevant input partitions/files/registrations verified;
4. parser/evaluator/normalizer capabilities Complete;
5. required corrections Applied/NotApplicable with no expired/rejected/conflict blocker;
6. exact cross-reference/identity index Complete;
7. persistent store write/read/schema/integrity capabilities Complete;
8. no relevant unknown field/unsupported construct/conflict;
9. no relevant truncation/budget/cancellation;
10. no runtime/hotfix-only uncertainty for the requested claim;
11. query returned no exact fact after validated exact lookup.

Otherwise authoritative `no` with exact blockers.

## 13. Decision result

```text
NegativeAuthorityDecision
    decision ID
    exact request/query/view/profile/generation
    relevant capabilities/partitions
    coverage records/summaries
    conflict/unknown/unsupported/correction/truncation/runtime blockers
    exact lookup result ID
    authoritative yes/no
    reason codes
    producer/version/digest
```

`authoritative=no` is not itself a finding that the entity exists; it means absence cannot be asserted.

## 14. Query result state matrix

### Found

Exact fact found; return fact/evidence/coverage. Negative authority not needed.

### AbsentAuthoritative

No exact fact and authority decision yes.

### NotFoundPartial

No exact fact, one or more relevant coverage/unknown/unsupported/truncation blocker.

### Conflict

Competing evidence/identity/correction/ref resolution prevents conclusion.

### NotEvaluated

Required capability/profile/store/parser/runtime contract unavailable or deliberately not run.

### InvalidRequest

Ambiguous/malformed/wrong view/profile/generation/kind key.

No generic null/empty result.

## 15. Aggregate/list queries

List/namespace operations report:

```text
selected exact scope
underlying coverage state
returned count
known total count only when exact/bounded complete
truncation/continuation state
```

An empty list is authoritative only under the same relevant coverage conditions and exact scope.

## 16. Release eligibility

Release profile capability declaration includes exact partition statuses. A candidate can release only declared capabilities that satisfy gates; a profile with partial optional capabilities may be release-eligible if manifest clearly declares them and no mandatory gate depends on them. Do not label the entire pack “complete” without capability map.

## 17. Store interaction

Persistent rows for facts, raw observations, coverage, conflicts, corrections, manifests, and capabilities must all be written/read/validated. Missing coverage/conflict rows are a store validation failure, not permission to assume complete.

ReferenceView validates store generation/schema/manifests before decisions.

## 18. Determinism

Equivalent inputs/policies produce equivalent:

```text
coverage dependency graph/records/summaries
conflict/unknown/unsupported blockers
negative-authority decisions/reasons
release capability manifest
```

Independent of diagnostic prose, row order, worker order, temp path/time.

## 19. Required operations

```text
build_reference_capability_registry
build_reference_coverage_partitions
record_stage_coverage
propagate_coverage_dependencies
record_unknown_unsupported_conflict_blockers
combine_reference_capability_coverage
validate_reference_coverage_closure
build_negative_authority_request
resolve_relevant_negative_authority_partitions
evaluate_reference_negative_authority
build_reference_release_capability_manifest
validate_reference_release_eligibility
```

## 20. Required tests

- complete positive and authoritative negative;
- partial/failed/unknown/not-applicable/not-evaluated matrices;
- unsupported dependent-only vs system-wide propagation;
- complete ingestion plus unresolved conflict blocks authority;
- unknown restriction/signature/applicability field blocks relevant claims;
- expired correction blocks authority;
- store write/read/integrity missing records blocks authority;
- bounded list truncation and empty list cases;
- old exact profile authority vs current-target mismatch;
- runtime/hotfix-sensitive static uncertainty;
- unrelated partial partition does not block exact independent query;
- deterministic reasons/partition ordering.

## 21. Hard stops

- no global complete boolean as authority;
- no empty-result absence by default;
- no conflict overridden by complete ingestion;
- no unknown/unsupported ignored;
- no store-row absence authority;
- no runtime safety/replacement claim through absence;
- no truncation clean result;
- no cross-profile fallback;
- no `NotEvaluated` as pass.
