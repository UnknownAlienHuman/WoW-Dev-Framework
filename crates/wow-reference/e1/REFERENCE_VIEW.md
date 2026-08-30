# Exact read-only `ReferenceView`

**Status:** normative E1-B exact query, result, context, budget, and negative-authority contract.

`ReferenceView` is the only supported domain read boundary over a published E1 ReferenceStore. It exposes exact reference facts and their evidence/coverage without raw SQLite, fuzzy search, cross-profile fallback, or source mutation.

## 1. View identity

```text
ReferenceView
    view_id
    exact ProfileIdentity / ReferenceGenerationId
    ReferenceStore StoreId / StoreGenerationId / StoreManifest ID
    schema/read operation catalog/version
    capability/coverage/conflict/correction manifest IDs
    source/object manifest IDs
    runtime/open validation report ID
    budgets/cancellation policy
    producer/tool versions
```

Opening validates all identities and read-only store state. A view never switches if an active pointer changes.

## 2. Open request

```text
OpenReferenceViewRequest
    exact ProfileId
    exact ReferenceGenerationId or exact store generation selected by a validated mapping
    expected source/correction/schema/parser identities: optional strict constraints
    requested validation level
    budgets/cancellation
```

No implicit current/latest/last-used/fallback.

## 3. Open result

```text
OpenReferenceViewResult
    Opened(view)
    ProfileOrGenerationMismatch
    StoreUnavailableOrInvalid
    CapabilityUnavailable
    Cancelled
```

Last-known-good can be opened only through its own exact ID/request. Never relabel it as the requested failed generation.

## 4. Exact entity key

```text
ReferenceExactKey
    entity kind
    canonical system/namespace
    canonical name/member name
    owner/receiver entity key where required
    exact signature/type/ordinal discriminator where required
```

Validation is kind-specific. A string alone is insufficient when multiple entity kinds/scopes can share a name.

## 5. Core operations

### `get_profile_and_generation`

Return exact profile/source/parser/evaluator/normalizer/correction/schema/store/manifest identities and eligibility/capability overview.

### `lookup_exact_entity`

Exact kind/scope/key lookup; no fallback.

### `lookup_api_callable`

Return callable fact, ordered parameters/returns, restrictions, predicates, deprecation/transition, raw/evidence/source/coverage refs.

### `lookup_event`

Return exact event and ordered payload records with restrictions/evidence/coverage.

### `lookup_table_or_structure`

Return exact structure/table and bounded ordered fields/type refs.

### `lookup_enum_or_cvar`

Return exact selected record/member values/metadata as supported by profile.

### `lookup_widget_or_method`

Return exact widget/script object/method; no inferred inheritance beyond persisted explicit E1 facts.

### `lookup_restriction_facets`

Return known open-facet records plus raw unknown/unsupported/conflict blockers for exact target.

### `lookup_deprecation_or_explicit_transition`

Return only exact source/correction-supported records; no similarity-based replacement.

### `read_raw_metadata`

Return bounded exact raw observation/value/unknown/unsupported/correction relations for exact handles.

### `resolve_reference_source_handle`

Return validated stable source handle and optionally bounded source detail only when allowed/object available.

### `list_exact_scope_entities_bounded`

List exact selected profile/system/namespace/kind/owner scope under count/byte budget; no fuzzy ranking.

### `negative_authority_decision`

Evaluate exact absence authority under `COVERAGE_AND_NEGATIVE_AUTHORITY.md`.

## 6. Lookup request

```text
ReferenceLookupRequest
    request_id
    view_id/profile/reference generation
    operation kind
    exact entity/member key
    requested detail mask
    include raw/evidence/source/coverage/corrections/conflicts flags
    count/depth/byte/work budgets
    cancellation token/state
```

Detail masks are typed. Unknown fields cannot be silently omitted if requested raw metadata is within budget; truncation explicit.

## 7. Lookup result family

```text
ReferenceLookupResult
    Found
    AbsentAuthoritative
    NotFoundPartial
    Conflict
    NotEvaluated
    InvalidRequest
    Cancelled
```

### `Found`

```text
exact entity/fact/member data
evidence/provenance/source handles
coverage/capability summaries and decisive records
restriction/predicate/deprecation/correction/conflict relations
raw metadata handles or bounded values as requested
view/profile/generation/store/schema identity
truncation/budget state
```

Finding an entity does not imply every requested field/capability complete; per-field partial state remains.

### `AbsentAuthoritative`

Contains exact `NegativeAuthorityDecision` with relevant complete partitions and lookup proof.

### `NotFoundPartial`

No exact record but absence not authoritative; includes blockers.

### `Conflict`

Exact competing observations/entities/corrections/links prevent one result; returns bounded conflict/evidence handles.

### `NotEvaluated`

Required capability/partition/store/raw/runtime contract unavailable or intentionally deferred.

### `InvalidRequest`

Malformed/ambiguous/wrong kind/profile/generation/view key.

## 8. Exact duplicate/cardinality handling

Schema/normalization should ensure one exact normalized entity key where contract says unique. Query encountering multiple nonidentical rows for a unique key is store/domain conflict/validation failure, not first row.

Operations that legitimately return ordered members/list sets declare cardinality/order.

No SQL result order without explicit canonical/domain order.

## 9. Evidence and raw source

Every material fact can carry:

```text
raw observation IDs
source handles
producer/version
provenance/confidence
correction applications
coverage/conflict records
```

Source handle is identity/evidence, not filesystem permission. Raw full source is not returned by default. Bounded exact source span resolution follows licensing/object/security/budget policy.

## 10. Restriction result semantics

Restriction lookup distinguishes:

```text
known normalized facet
raw known source payload
unknown field/facet
unsupported payload
conditional/predicate applicability
static source scope
runtime/hotfix gap
```

No `safe=false/true` blanket output. Downstream rules evaluate exact facet capabilities/context; runtime-only cases remain NotEvaluated/require probe.

## 11. Deprecation/transition semantics

Results distinguish:

```text
deprecated with exact source record
explicit alias
explicit replacement/moved-to
historical only for selected profile/input
no explicit transition recorded with authority state
```

“No explicit replacement recorded” is not “there is no replacement” beyond exact declared input/coverage claim, and never yields fuzzy candidate.

## 12. Raw metadata reads

Request exact:

```text
entity/raw observation ID
field path/prefix
max depth/nodes/bytes
include source/evaluator/correction relation
```

Result preserves canonical value distinctions and reports truncation/unsupported. Do not read whole raw store/build by default.

## 13. List operations

Exact filters only:

```text
entity kind
system/namespace
owner/receiver
profile/reference generation
```

Optional canonical prefix listing may be offered only as enumeration, not relevance/search, and must identify it as prefix filter. Results sorted canonically and bounded.

Return total only if exact/cheap/complete under operation contract; otherwise known returned count + truncation/continuation handle.

## 14. Cache behavior

ReferenceView is immutable and safe for caches keyed by exact view/generation/query/detail/budget contract. Cache:

- cannot omit evidence/coverage required by caller;
- includes schema/operation/tool versions;
- no cross-profile/generation reuse;
- no active pointer/current key;
- invalidation by closing/releasing view/retention, not mutation;
- cached partial/conflict stays scoped exactly.

## 15. Budgets and cancellation

Per operation:

```text
rows/facts/members/raw nodes/evidence/conflicts/source bytes
query work/statement/result bytes
object reads/decode bytes
output records/bytes
```

Budget exceed:

- exact single-entity core identity may return with explicit partial detail only if contract permits;
- absence authority cannot survive relevant truncation;
- list/raw result truncated explicitly;
- no silent field/evidence removal.

Cancellation returns no misleading partial clean/absent result and no background query.

## 16. Error and privacy

Structured errors include request/view/profile/generation/entity/capability/partition/store IDs and codes. Exclude raw SQL, private paths, excessive source/raw values, tokens/private URLs, runtime Secret-capable data.

## 17. Determinism

Equivalent view/query/detail/budget yields equivalent:

```text
result variant
entity/fact/member/raw/evidence/coverage/conflict order
negative-authority decision/reasons
result ID/digest
```

Independent of SQLite row/page order, connection, thread scheduling, temp root, diagnostic prose, active pointer changes.

## 18. Required operations

```text
open_reference_view
validate_reference_view
close_reference_view
validate_reference_lookup_request
normalize_reference_exact_key
lookup_exact_entity
lookup_api_callable
lookup_event
lookup_table_or_structure
lookup_enum_or_cvar
lookup_widget_or_method
lookup_restriction_facets
lookup_deprecation_or_explicit_transition
read_raw_metadata
resolve_reference_source_handle
list_exact_scope_entities_bounded
evaluate_reference_negative_authority
validate_reference_lookup_result
canonicalize_reference_lookup_result
```

## 19. Required tests

- each exact entity/fact operation positive;
- authoritative negative, partial miss, conflict, NotEvaluated, invalid request;
- same name different kind/system/owner/signature;
- duplicate unique row conflict;
- profile/reference/store/schema mismatch;
- active pointer changes while view remains exact;
- raw unknown/unsupported/correction read;
- restriction known + unknown/runtime gap;
- explicit vs inferred transition;
- per-field partial under Found entity;
- bounded list/raw/source detail truncation;
- row/input/thread/order determinism;
- cache exact context/detail/budget key;
- no SQL/fuzzy/external/source mutation/privacy leak.

## 20. Hard stops

- no implicit current/fallback;
- no raw SQL/store handle;
- no string-only ambiguous key;
- no first-row conflict resolution;
- no empty/null generic result;
- no fuzzy/FTS/semantic/replacement lane;
- no absence under partial/conflict/truncation/runtime gap;
- no blanket restriction safety;
- no cross-profile cache/read;
- no silent detail truncation;
- no view generation switch.
