# `wow.api.exists@1`

**Status:** normative E0-E rule algorithm.

## 1. Purpose

Report a direct Main-project API member/call use that is proven absent from the selected exact reference profile.

The rule does not search for replacements, validate signatures, classify deprecation, infer runtime safety, or report unresolved ordinary Lua symbols.

## 2. Descriptor

```text
rule_id: wow.api.exists
version: 1
semantic_category: wow.api.missing
technical_severity: error
rollout_policy: advisory
remediation_tiers: plan_only
source_scope: Main project direct member/reference use
supported_profile: fixture-retail-120100-e0-v1 (E0 fixture only)
```

## 3. Required project/analyzer input

```text
ProjectSnapshot / ProjectView identity
ProjectGenerationId
Main ProjectFileRecord
exact project SourceHandle
ReferenceFact:
    reference_kind = member
    receiver_spelling = C_E0Fixture
    member_spelling = RemovedApi (fixture case)
    resolution_status = unresolved
    exact member/full-reference span
optional CallFact tied to the same ReferenceFact
source-coordinate capability Complete
reference/call fact capabilities Complete for selected use
```

The E0 rule applies only to an exact direct member reference. Ambiguous/dynamic/computed member uses are `NotEvaluated` or nonapplicable.

## 4. Exact query construction

Construct the canonical exact entity key from the normalized project fact and fixture namespace grammar:

```text
function:C_E0Fixture.RemovedApi
```

Rules:

- no case correction;
- no alias/prefix/fuzzy/FTS/semantic lookup;
- no guessing function versus field/method kind;
- no namespace fallback;
- no external repository lookup;
- invalid/noncanonical fact/query -> context/input failure or `NotEvaluated`, not alternate search.

## 5. Required reference lookup

Call E0-B `ReferenceView.lookup_symbol_exact` with:

```text
selected profile/reference generation
canonical EntityKey
expected entity kind = function
```

Expected outcomes:

```text
found
authoritative_absent
absent_without_authority
conflict
profile_mismatch
capability_unavailable
```

## 6. Decision table

| Project fact | Exact reference outcome | Rule outcome |
|---|---|---|
| unresolved direct member/call | `authoritative_absent` | one finding |
| unresolved direct member/call | `found` | `EvaluatedClean` for API existence only |
| unresolved direct member/call | `absent_without_authority` | `NotEvaluated` |
| unresolved direct member/call | `conflict` | `NotEvaluated` |
| unresolved direct member/call | `capability_unavailable` | `NotEvaluated` |
| unresolved direct member/call | `profile_mismatch` | context failure |
| resolved direct member/call | exact found or resolution already proves fixture declaration | clean/nonapplicable according to selected scope; no absence finding |
| ambiguous/dynamic/computed member | any | `NotEvaluated` or nonapplicable; no guessed query |
| ordinary unresolved local/global not in WoW API namespace | any | nonapplicable; no WoW finding |

## 7. Authoritative absence requirements

A finding requires all:

```text
exact canonical query
selected profile/reference generation coherent
exact reference capability usable
exact system/entity-kind partition Complete
negative-authority decision = authoritative
no relevant conflict
no truncation/stale input
project reference fact/source current
rule scope fully evaluated
```

An empty reference result alone is insufficient.

## 8. Finding primary source

Preferred primary span:

1. exact unresolved member-name span (`RemovedApi`);
2. otherwise exact full member-reference span (`C_E0Fixture.RemovedApi`);
3. never a whole file/line when exact span exists.

The call span may be related source evidence.

Primary source is project evidence/location, not platform evidence.

## 9. Evidence inputs

### Project evidence

- Main project SourceHandle/content/project generation;
- unresolved member ReferenceFact;
- optional direct CallFact;
- analyzer producer/version/snapshot;
- project/analyzer coverage IDs.

### Reference authority inputs

- exact lookup request/result ID;
- selected profile/reference generation;
- exact coverage record IDs;
- negative-authority decision and reasons;
- conflict IDs (empty for finding branch);
- reference producer/version.

No source handle is fabricated for the absent entity.

### Rule derivation

- `wow.api.exists@1` descriptor/provider version;
- canonical entity key;
- decisive fact/query/authority IDs;
- rule execution context/fixture policy.

## 10. Finding arguments

```text
rule_id: wow.api.exists
rule_version: 1
missing_entity_key
receiver_spelling
member_spelling
use_kind: member_reference | direct_member_call
selected_profile_id
reference_generation_id
authority_status: authoritative_absent
```

Rendered message example is non-normative:

```text
`C_E0Fixture.RemovedApi` is not present in the selected reference profile.
```

Message text is not identity.

## 11. Finding identity

Canonical fingerprint includes:

```text
rule ID/version
GenerationContext ID
primary project SourceHandle/span/content digest
canonical missing EntityKey
project ReferenceFact ID
reference exact lookup/authority decision ID
use kind
provider version
```

Excludes message prose, timestamps, discovery order, temp paths, and generic finding IDs unless used only in a separate causal hint.

## 12. Deduplication

- equivalent duplicate observations at the same canonical use/source/fact -> one finding;
- distinct source spans -> distinct findings;
- multiple calls sharing one member expression according to AST/fact identity follow exact per-use scope fixture policy;
- do not collapse all uses of one missing API into a single repository-level finding in E0.

## 13. Generic analyzer symptom relation

The analyzer may also emit a generic unresolved/unknown-member diagnostic.

The rule may emit a causal hint only when:

```text
same ProjectGenerationId / AnalyzerSnapshot
same Main file/content
same member/reference fact or exact span
compatible generic semantic category
API absence is authoritative
```

Conceptual relation:

```text
wow.api.exists finding
    causes_or_explains
same-source generic unresolved-member symptom
```

`wow-rules` does not suppress/fold/reorder the generic finding. `wow-service` owns that.

## 14. Clean outcome

`found` yields `EvaluatedClean` only for this narrow question:

```text
the exact referenced API entity exists in the selected profile
```

It does not imply:

- correct arguments/returns;
- nondeprecated status;
- Secret/protected safety;
- load/reachability correctness;
- runtime availability in every context.

Clean record includes scope/fact/query/coverage/budget IDs.

## 15. NotEvaluated cases

Required blockers include:

- partial/failed/unknown exact reference partition;
- `absent_without_authority`;
- reference conflict;
- profile/reference/project/analyzer generation mismatch (context error where appropriate);
- annotation library failure/no exact project reference fact;
- ambiguous/dynamic/computed member;
- invalid source span/digest;
- unsupported entity kind/query grammar;
- budget/truncation preventing complete scope evaluation;
- retained stale project snapshot substituted for requested target.

No API finding or clean record accompanies that scope.

## 16. Remediation

Tier: `plan_only`.

Structured plan:

1. confirm the selected profile/reference generation is the intended target;
2. locate the current exact API/Blizzard extension contract using authoritative reference/search tooling when later milestones exist;
3. inspect project intent and call site;
4. implement a profile-valid change only after an explicit replacement/current contract is proven;
5. rerun generic + WoW diagnostics and project tests.

Prohibited:

- suggest similarly named API;
- automatic delete/comment-out;
- replacement from fuzzy/semantic/external code;
- exact edit in E0.

## 17. Required operations

```text
is_api_exists_scope_applicable
build_api_exists_exact_query
evaluate_api_exists_capabilities
classify_api_exists_lookup_outcome
build_api_exists_finding
build_api_exists_clean_record
build_api_exists_not_evaluated
build_api_generic_causal_hint
validate_api_exists_outcome
```

## 18. Fixture cases

```text
api.known-found
api.removed-authoritative-absent
api.removed-partial
api.removed-conflict
api.profile-mismatch
api.library-failure
api.dynamic-or-computed-member
api.ordinary-unresolved-symbol-nonapplicable
api.no-fuzzy-fallback
api.duplicate-same-span
api.distinct-use-spans
api.generic-causal-hint
api.generic-no-unproven-causal-hint
api.budget-truncation
```

## 19. Hard stops

- no finding without authoritative exact absence;
- no analyzer unresolved -> absence upgrade;
- no alias/fuzzy/replacement lane;
- no absent-entity source handle;
- no generic symptom suppression;
- no clean under partial/conflicted reference coverage;
- no source mutation/edit;
- no runtime claim;
- no whole-file span when exact member span exists.
