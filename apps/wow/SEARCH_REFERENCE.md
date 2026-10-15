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
reject. Alias/similarity/seeded-graph lanes remain explicitly unavailable in this
component. A required unavailable lane cannot be hidden by a positive name hit.

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

This Reference query currently returns one bounded immutable result manifest.
Reference result paging/federation, other Reference/project/Blizzard document
kinds, explicit aliases, similarity/graph lanes, full corpus/fault/platform
qualification and current Tier X/runtime acceptance remain in Issue #169.
