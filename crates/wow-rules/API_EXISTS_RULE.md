# `wow.api.exists@1`

**Status:** normative and executable rule algorithm with closed E0 fixture, native
release-profile policies and an exact XML-inline source lane.

## 1. Purpose

Report a direct Main-project API member/call use that is proven absent from the
selected exact reference profile. The source may be a physical Main Lua file or an
admitted inline Lua unit whose static member/call facts map exactly back to a captured
XML document.

The rule does not search for replacements, validate signatures, classify deprecation,
infer runtime safety, or report unresolved ordinary Lua symbols.

## 2. Descriptor

```text
rule_id: wow.api.exists
version: 1
semantic_category: wow.api.missing
technical_severity: error
rollout_policy: advisory
remediation_tiers: plan_only
source_scope: physical Main or exact-static XML inline direct member/reference use
supported_profile: closed E0 fixture or an exact profile-bound native release policy
```

## 3. Required project/analyzer input

```text
ProjectSnapshot / ProjectView identity
ProjectGenerationId
one admitted source lane:
    physical Main ProjectFileRecord and exact SourceHandle
    or captured XML document + XmlLuaUnitAnalysis with exact_xml_script_site authority
ReferenceFact / XmlLuaMemberReference:
    reference_kind = member
    receiver_spelling = exact static namespace expression
    member_spelling = exact static member name
    resolution_status = resolved | unresolved | possible
    exact member/full-reference source mapping
optional CallFact tied uniquely to the same ReferenceFact
source-coordinate capability Complete
reference/call fact capabilities Complete for the selected physical file or XML unit
project.analyzer.facts.available Complete for the exact selected partition
```

The rule applies only to an exact direct static member reference. The E0 policy restricts the
receiver to `C_E0Fixture`. The native release policy admits a receiver only when the exact callable
partition already contains that namespace (or a conflict in it). XML units additionally require the
versioned semantic context to admit `exact_xml_script_site`; this does **not** admit an implicit
callback receiver, inherited template receiver, lifecycle state or runtime dispatch. Ambiguous,
dynamic or computed member uses are `NotEvaluated` or nonapplicable.

## 4. Exact query construction

Construct the canonical exact entity key from the normalized project fact:

```text
function:<receiver>.<member>

# E0 example
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
| unresolved direct member/call in an admitted physical/XML lane | `authoritative_absent` | one finding |
| direct member/call in an admitted physical/XML lane | `found` | `EvaluatedClean` for API existence only |
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

### Native release-profile authority

The profile-bound native policy uses `reference.native.apidoc.api`. Exact presence
is usable under Partial coverage. Absence becomes authoritative only when all of
the following are retained in the same generation:

```text
complete exact source-manifest member inventory
selected generated-API TOC set equals the complete generated_api member set
all selected documents admitted
no in-domain normalization, payload-limit, or record-construction loss
callable partition coverage = Complete
exact query has no key-scoped conflict
```

Environment-excluded systems and ScriptObject methods are outside this
global/namespace callable partition and remain explicit omissions. The authority
is relative to the exact pinned corpus/profile/environment. It does not attest
remote Git membership, currentness, runtime availability, signatures, hotfixes,
protected-state behavior, or client acceptance. Explicit file lists and imported
prebuilt artifacts remain Partial and cannot produce absence findings.

## 8. Finding primary source

Preferred primary span:

1. exact unresolved physical member-name span (`RemovedApi`); or
2. first exact XML member-name source piece for an admitted inline unit;
3. otherwise exact full physical member-reference span (`C_E0Fixture.RemovedApi`);
4. never a whole file/line when exact mapped evidence exists.

Additional discontinuous XML pieces, the full reference and the optional call span are related
project evidence. Every XML span is validated against the captured document digest and byte length.

Primary source is project evidence/location, not platform evidence.

## 9. Evidence inputs

### Project evidence

- physical Main SourceHandle or generation-bound captured XML SourceHandle(s);
- captured source content digest, byte length and project generation;
- member ReferenceFact or XML member-reference fact;
- optional uniquely linked direct CallFact;
- XML unit/context/mapping identity when the source is inline XML Lua;
- analyzer producer/version/snapshot;
- project/analyzer coverage IDs for the exact file or XML unit.

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
- rule execution context and exact fixture or production policy identity.

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
`C_Spell.RemovedApi` is not present in the selected exact reference profile.
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
- incomplete/failed XML semantic-unit capabilities or syntax/documentation errors;
- unsupported entity kind/query grammar;
- budget/truncation preventing complete scope evaluation;
- retained stale project snapshot substituted for requested target.

No API finding or clean record accompanies that scope.

Invalid source spans/digests, non-exact XML mappings, a semantic context without
`exact_xml_script_site`, or a duplicate/open call-reference graph are context/input
failures. They reject rule execution rather than being downgraded to missing evidence.

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
- no implicit XML callback receiver, inherited receiver, lifecycle or runtime-dispatch claim;
- no runtime claim;
- no whole-file span when exact member span or exact XML pieces exist.
