# Native Reference callable Search

This component indexes the exact global and namespace callables admitted by the
existing [source-only Reference reader](SOURCE_REFERENCE.md). It consumes the
sealed `NativeCallableSignature` name, namespace and ordered field read ports.
Search never decodes an opaque Reference payload or extracts a name from a key.

## Build and exact handle

```text
wow search reference build --config <ABSOLUTE-SOURCE-REFERENCE-JSON>
  --expect-config-sha256 <SHA256-OF-EXACT-CONFIG-BYTES>
  --search-store <ABSOLUTE-STORE-FILE> --operation-id <ID>
  --allow-local-metadata-only <SEARCH-BOUNDS> <STORE-BOUNDS>
  --max-output-bytes <LIMIT>
```

The finite Search/Store flags are listed by `wow search --help` and
[the existing Search commands](SEARCH_BUILD_QUERY.md). The input configuration
uses the existing explicit digest-pinned files or manifested TOC route. Profile,
revision, flavor, build, Interface, source acquisition, raw metadata, corrections,
coverage and currentness remain with their original owners. Build verifies the
exact configuration digest. Its native source reader executes no source code.

Build atomically stores a genuine ReferenceStore view object, the immutable
Search shard, independent source/shard catalog roots and completed response in
the existing SearchStore SID v2 transaction. The source root is keyed by shard,
so retiring another shard sharing that ReferenceView cannot release it. Physical
SQLite/FTS, read-only images, validation, journaling and GC stay with `wow-store`.
No new Store layout or FTS backend is introduced. Exact completed build replay
needs neither the original config file nor source files.

`payload.handle` is the closed
`wow-service/native-reference-search-handle/1` object. Preserve it whole: it binds
the exact logical shard, object, ReferenceView object, Reference generation/view
digest, Search profile and all Search bounds. Each shard also binds the original
ProfileIdentity and actual ReferenceStore instance/configuration. It is a
separate `ReferencePlatformContract` universe; equal strings in a Project shard
remain separate identities. No current/latest selector is accepted.

## Query

```text
wow search reference query --search-store <SAME-STORE>
  --handle-json <EXACT-HANDLE-JSON> --query-json <CLOSED-QUERY-JSON>
  --operation-id <ID> --lease-expires-after <EPOCH>
  --allow-local-metadata-only <STORE-BOUNDS> --max-output-bytes <LIMIT>
```

For example:

```json
{
  "schema": "wow-search/native-reference-query/1",
  "selectors": {
    "qualified_name": "C_Example.Read",
    "short_name": "Read",
    "namespace_member": {"namespace": "C_Example", "member": "Read"},
    "prefix": "Re",
    "shape": {"argument_count": 1}
  },
  "enabled_lanes": ["exact_canonical_name", "namespace_member", "identifier_prefix", "structured_shape"],
  "required_lanes": ["exact_canonical_name"],
  "max_scanned_documents": 1024,
  "max_candidates": 32,
  "max_signals": 128,
  "max_result_bytes": 1048576
}
```

All selectors are optional, but at least one must be present and its lane must
be enabled. Other selectors are `entity_key` and `literal_text`. Unknown fields,
duplicate lanes, required-but-disabled lanes, zero bounds, control-bearing text,
unbounded text, prefixes shorter than two UTF-8 bytes and empty shape filters
reject. Explicit-alias and seeded-graph lanes remain unavailable in this
component. Explicit [identifier similarity](SEARCH_IDENTIFIER_SIMILARITY.md) is
available only with its own selector and named query/ranking profiles; an old
request with no such selector retains its unavailable-lane behavior. A required
unavailable lane cannot be hidden by a positive name hit.

Exact comparisons preserve case. Qualified names are the frozen projection of
the explicit namespace plus `.` plus name, or the original name for a global
owner. Short names never collapse distinct namespaces. Namespace/member compares
both original fields. Prefix tests the qualified and short name fields; each
signal identifies the actual matched field and its prefix byte range. Literal
FTS indexes canonical identifiers only, compiles text through the existing safe
native literal lane, and remains Candidate. No raw MATCH/SQL, stemming, inferred
aliases, documentation mentions or source bodies become name authority.

Shape filters support declared original argument/return field counts and an
`arguments` array of `{position, original_type, nilability?}`. Positions are zero
based and unique. Nilability is `unspecified`, `allows_nil` or `disallows_nil`;
missing metadata is distinct from explicit false. Type text and inner-type text
remain original unresolved source identities. Nilable does not imply optional;
declared field count is not resolved variadic call arity. Defaults, variadic
classification, corrected/resolved type equivalence and receiver inheritance
are not queried by this profile. An available upstream type-resolution profile
is retained as metadata. Shape signals are Candidate.

### Original field-shape queries

Select `"schema": "wow-search/native-reference-field-shape-query/1"` to filter
ordered callable return fields as well as arguments, event payloads, structure
fields and callback arguments/returns. This uses the same retained shard handle,
query command, federation and captured-page lifecycle; no index rebuild is needed.
For example, the following `selectors` searches a callable's first return:

```json
{
  "shape": {
    "return_count": 1,
    "returns": [{
      "position": 0,
      "original_type": "table",
      "inner_type": {"kind": "equals", "original_type": "number"},
      "nilability": "allows_nil"
    }]
  }
}
```

The remaining request envelope and finite budgets are the same as above. Enable
`structured_shape`. Each field filter requires its original outer `original_type`;
optional `inner_type` is either `{"kind":"unspecified"}` for metadata the owner
did not state, or `{"kind":"equals","original_type":"..."}` for exact recorded
text. Omitting `inner_type` leaves that field unconstrained. Case is preserved;
missing metadata never matches an explicit inner type. These filters also apply
inside the existing event `event_payload` and declaration `fields`, `arguments`
and `returns` arrays. Callable, event and declaration selectors cannot be mixed.
Positions remain zero based and unique within each array, with at most 128 filters
per array. Out-of-range positions match nothing; no type equivalence, widening,
default, variadic, runtime or absence conclusion is inferred.

The new schema selects `wow-search/native-reference-field-shape/1`; its exact
request and retained shard bind the query identity. Shape signals remain band-7
Candidate evidence with original field, record, source and value digests. Legacy
`wow-search/native-reference-query/1` rejects callable `returns` and `inner_type`
extension keys and retains its original query/profile/result bytes. Existing
declaration-return filters in that legacy schema remain supported.

The result is an immutable
`wow-search/native-reference-result-set/1` manifest with complete whole-candidate
name/namespace/field/source projections and every retained signal. Each signal
preserves the actual owner field, original record/source identity, original and
normalized value digests, transformation and integer rank contribution. Field
members retain original source byte ranges, including reused local tables whose
member ranges may lie outside the callable table range. Owner profile, view,
source origins, partition coverage/counts, conflicts and projection issues remain
visible. Original source-acquisition failure receipts retain relative path,
source digest, error code and optional original byte range; failed files remain
omissions and cannot authorize absence. Opaque payloads and unrelated field
documents are not query output.

Bands are identity `0`, qualified name `1`, short name `2`, namespace/member `4`,
prefix Candidate `5`, and literal/shape Candidate `7`. Both qualified and short
name signals survive when both selectors match. Ranking uses band, best canonical
lane rank and reverse integer reciprocal-rank sum, ending in the exact entity key.
Contributions are `1000000 / (60 + rank)` with integer division. FTS row order,
physical layout and BM25 do not determine semantic order. Scan/candidate/signal
caps remain explicit truncation; result-byte refusal cannot silently strip fields.

Reopen validates the exact native Store artifact, physical rows, source/view/
generation/profile/store bindings, original Reference records/digests/source IDs,
coverage/conflicts and source-origin hashes through the owners. Query needs no
original source files. Resources close before response publication. Completion
reasserts both exact catalog roots, so concurrent retirement cannot recreate them.
An exact retry replays recorded bytes; a changed request under that operation ID
fails. Missing/retired/corrupt/mismatched owners never trigger a rebuild or fallback.

An exact hit proves the stored string relation. Missing canonical names remain
partial. An authoritative exact-entity miss additionally requires the original
Reference owner's negative decision, complete relevant enumeration/query coverage,
no truncation/conflict and no partial acquisition. An exact conflicted key is
`conflict_blocked`; empty text/shape/prefix remains nonauthoritative. No runtime,
Secret/combat/taint safety or caller-intent authority follows from these records.

## Documentation inventory

An explicit `search reference build --include-documentation` selects
`native_reference_documentation_inventory`. Its source configuration must select
`native_documents: "source_declarations_v1"`. The original native callable,
event, structure, callback, enumeration, constant and ScriptObject records remain
the exact entities. This component adds their source `Documentation` fields;
it does not index full source files, arbitrary comments or opaque record payloads.

The exact handle uses `wow-service/native-reference-documentation-search-handle/1`
with `include_documentation: true`. Build/query/retire, Reference pages, federation
and captured federation pages use their existing operations and retention roots.
No implicit build or current/profile fallback is added. Preserve the whole handle.

Use `wow-search/native-reference-documentation-query/1` with the existing request
envelope and finite lane/candidate/scan/result bounds. In this component,
`literal_text` with `literal_text` lane searches **documentation only** through
the existing safe native literal FTS5 query. Names are available through the
existing exact-name, namespace/member, prefix and similarity selectors, and
the original field-shape selectors are supported. Documentation queries require
this component; the other query schemas cannot silently select its text index.
Old components and requests keep their original identifier-only FTS bytes and
behavior.

Each returned document retains the original ordered strings, original native
record/source identity and one of `recorded`, `not_stated` or `unsupported`.
An explicit empty string array is recorded; an absent field is not stated.
Non-array, gapped or non-string event/declaration documentation remains unsupported
without erasing the original entity. Original callable normalization refusals
remain in the native owner's issues/omissions. Whole-corpus documentation counts
remain visible in `owner.documentation_coverage`, even when no candidate matches.

The index joins original lines with spaces and replaces whitespace/control
characters with spaces under `original-documentation-whitespace-to-space/1`.
Original strings are retained without that transformation. A single physical
blank represents an empty token stream for the existing nonempty row contract;
it is never source metadata and cannot match a valid text query. Bounds apply to
the full encoded field; over-cap input refuses rather than truncating evidence.

Text signals are `literal_reference_documentation_candidate`, band 7, with the
actual callable/event/declaration documentation port, original and indexed value
digests and normalization profile. Their authority class is
`ReferenceContractDocumentation`; mentions never become exact names, aliases,
replacement or runtime facts. Empty text results carry no absence authority.
Documentation is untrusted data and remains subject to the explicit local
metadata permission and the retained `license_state: not_evaluated`.

## Full native Reference inventory

`search reference build --reference-inventory` selects
`native_reference_inventory`. It includes the original source declarations and
documentation plus the already produced `reference.native.apidoc.restriction`
partition. The configuration must select `source_declarations_v1` and genuine
manifested TOC input; an explicit pinned-file list cannot supply this capability.
This flag and `--include-documentation` are mutually exclusive.

Preserve the exact `wow-service/native-reference-inventory-search-handle/1`
handle, with both include flags true. Query it using
`wow-search/native-reference-inventory-query/1`; other components' query schemas
cannot silently select this inventory. Existing operations, native Store,
federation, original Reference replay pages and captured federation pages remain
the consumers. `literal_text` retains documentation-only FTS semantics.

For example, enable the `structured_shape` lane with:

```json
{
  "shape": {
    "restriction": {
      "kind": "secret_return",
      "scope": "first_return_unconditional_source",
      "facet_id": "secret.return",
      "state": "restricted",
      "subject_binding": "exact"
    }
  }
}
```

Restriction shape fields are optional, but at least one is required. The finite
`kind` values are `secret_return` and `access_predicate`; scopes are
`first_return_unconditional_source` and `first_argument_true_immediate_caller`.
An optional `subject_entity_key` is the original API owner key.
`subject_binding` accepts `exact`, `unresolved` or `conflicted`. Restriction,
callable, event and declaration shape selectors cannot be mixed.

Each facet candidate retains its original owner partition/key, digest, typed
facet/state/evidence, exact field origin and recorded subject name/key/scope.
Its owner-produced `reference-restriction:...` identity is separate from the API
key: both partitions may use the same original record key. A genuine API target
digest is present only when that exact record exists. Missing/conflicted targets
remain unresolved, and nonidentity signals cap at band 10; resolved subject shape
matches remain Candidate band 7. Exact lookup of the scoped facet entity ID is
identity band 0. A facet's identifier does not become a canonical API name.

These are the existing positive static source observations only. First-return
scope does not expand to all returns; the predicate is bound to the immediate
caller and a true result. `allowed`/`restricted` describe that original facet
scope, never blanket runtime safety. False, omitted, conditional or unsupported
Secret metadata does not create a negative facet. Restriction coverage remains
Partial even when the native API callable partition is Complete. Derived facet
records have no documentation field; their subject's documentation remains on
its own API record. Older profiles, handles, queries and result bytes are retained.

## Configured compatibility aliases

The native source-reference configuration accepts one optional pinned input:

```json
{
  "compatibility_aliases": {
    "path": "aliases.json",
    "content_digest": "sha256:<actual-raw-file-digest>",
    "byte_length": 1234
  }
}
```

Use actual digest/length and `native_documents: "source_declarations_v1"`.
The resource is a strict `wow-reference/configured-compatibility-alias-input/1`:

```json
{
  "schema": "wow-reference/configured-compatibility-alias-input/1",
  "profile_id": "<exact-base-profile-id>",
  "profile_digest": "sha256:<canonical-validated-ProfileIdentity-digest>",
  "base_generation": "<exact-base-reference-generation>",
  "base_view_digest": "sha256:<exact-base-view-digest>",
  "declarations": [{
    "value": "C_Example.LegacyRead",
    "target": {
      "partition_id": "reference.native.apidoc.api",
      "record_key": "function:C_Example.Read",
      "record_digest": "sha256:<actual-owner-record-digest>"
    }
  }]
}
```

Acquire the base without the optional resource. Use its actual source report and
owner/Search read results for profile/view/generation/record identities; key text
or a display name cannot supply a record digest. The profile digest is SHA-256
over the Core canonical validated `ProfileIdentity`. Input is limited to 64 KiB,
128 declarations and 4096-byte bounded identifier/value fields; the complete
encoded owner header must also fit the existing 64-KiB record payload limit.
Oversize input refuses; it is never truncated. Unknown/duplicate/null fields and
caller-selected kinds/policies/confidence are not accepted.

`--reference-inventory` automatically selects an alias-capable inventory when the
native owner has this resource. The original manifested restriction requirement
still applies. `--include-documentation` supports the same alias resource without
requiring the restriction capability. The returned exact handle uses
`wow-service/native-reference-compatibility-search-handle/1`; preserve it whole.
Omitted resource keeps the original profile/handle/query bytes and unavailable
alias lane. A default noninventory build does not implicitly select this lane.

Query an alias-capable handle using `wow-search/native-reference-alias-query/1`
with `selectors: {"explicit_alias":"C_Example.LegacyRead"}`, enabled/required lane
`exact_explicit_alias` and the normal finite request envelope. Other native name,
documentation, similarity and shape selectors remain explicit independent lanes;
restriction shape requires its actual shard capability. Old query schemas reject
the new selector, including explicit null.

Resolved aliases retrieve the original exact target document once, with band-3
`exact_configured_owner_compatibility_alias` evidence. Distinct alias record/source
evidence survives alongside name/identity signals. Alias evidence identifies the
owner record/digest, literal value, original source IDs/indices, exact target and
configured-declaration authority class. Same strings do not merge universes.

Missing targets, changed digests, target conflicts and multiple exclusive mappings
remain visible without exact target signals. Alias chains/target partitions are
refused. Matching is case-sensitive, with no inferred deprecation/replacement,
runtime behavior, platform fact or caller intent. Alias coverage remains Partial;
an exact miss is not global alias absence. The original resource is retained by
Reference; Search never parses an opaque owner payload or hydrates native AST
caches from its projection. Existing native Reference/federated pages, offline
reopen, roots, retire/GC and historical response replay remain the route. See
[the owner seam](../../docs/EXPLICIT_COMPATIBILITY_ALIAS_SEAM.md).

## Retire

```text
wow search reference retire --search-store <SAME-STORE>
  --handle-json <EXACT-HANDLE-JSON> --operation-id <ID>
  --lease-expires-after <EPOCH> --allow-local-metadata-only
  <STORE-BOUNDS> --max-output-bytes <LIMIT>
```

Retirement validates the exact owner/shard and atomically removes that shard's
two roots plus records its completed response. Other shard/generation roots
remain independent. Existing explicit `search index gc` collects unrooted objects;
completed response replay remains historical and recreates no roots.

The default query returns one bounded immutable result manifest. Explicit
`--page-json` selects [Reference pages and continuation](SEARCH_REFERENCE_PAGINATION.md)
with independent result, shard and original ReferenceView retention.
Federation, other Reference/project/Blizzard document
kinds, explicit aliases, graph lanes, full corpus/fault/platform
qualification and current Tier X/runtime acceptance remain in Issue #169.
