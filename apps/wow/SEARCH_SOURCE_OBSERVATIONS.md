# Recorded source observations

The `project_source_observations_v1` component projects all local-binding and
direct-member-reference facts from one genuinely acquired exact Project pair.
It supports ordinary projects and Blizzard source publications, preserving their
original source class. It does not enumerate global function declarations.

Use the existing `search index build` and `search index retire` commands with
`--source-observations` instead of any file or Graph selection. Preserve the
returned exact shard, object, profile, generation, build operation and source-hold
receipt. Existing Store initialization, operation replay, source retention,
retirement and GC apply. No existing physical Store schema changes.

Query the captured immutable data with:

```text
wow search observations --search-store <STORE> --generation <EXACT>
  --search-shard-id <EXACT> --object-id <EXACT> --profile-digest <EXACT>
  --query-json <JSON> --operation-id <ID> --lease-expires-after <EPOCH>
  --allow-local-metadata-only <original Search/Store/output bounds>
```

The source files and original ProjectStore are unnecessary for this query.
Retirement still requires its original exact source-hold receipt and SourceStore.
The active Search catalog is checked before querying and fenced at completion;
an absent/retired shard cannot be implicitly rebuilt. Same-operation replay
returns the original recorded response, including after retirement.

A closed query selects exactly one lane and explicit allowed kinds:

```json
{
  "schema": "wow-search/source-observation-query/1",
  "query": {"class": "recorded_name", "name": "value"},
  "kinds": ["local_binding", "member_reference"],
  "max_scanned_documents": 128,
  "max_candidates": 16,
  "max_result_bytes": 1048576
}
```

Other `query` variants are `exact_fact` with `fact_id`, `recorded_member` with
`receiver` and `member`, `prefix` with `prefix`, `literal_name` with `text`, and
`identifier_similarity` with `identifier`, `ascii_fold`, `max_distance`,
`max_prefiltered_documents` and `max_distance_cells`. Unknown fields and variants
fail. Names are case sensitive. Literal input uses the existing safe FTS literal
profile over recorded names only. It accepts no raw SQL, regex or FTS expression.

Similarity uses the shared frozen Unicode scalar Levenshtein kernel with an
explicit optional ASCII fold, at most 128 query scalars, 132 candidate scalars and
distance 4. Cells and prefiltered documents are charged cumulatively across the
selected corpus. Excluded long identifiers, exhausted budgets, omitted candidates
and unscanned records remain explicit. Distance zero stays Candidate. Ranking is
deterministic by authority band, distance and exact original fact ID; duplicate
names remain separate occurrences.

Every candidate retains the exact original fact, file, content digest and native
source handle, while the result carries report/snapshot identities, per-file
coverage and Project/Graph coverage/conflicts. These are Search data projections,
never deserializable native owner authority. A recorded receiver is source syntax;
it does not prove an API namespace, alias, callable target or runtime binding.
No hit or miss proves availability, absence or complete source coverage.

This component currently has no pagination, federation, context selection or
frontend registration. Those remain separate #169/W22 scopes. Synthetic native
functional proof does not qualify vendor/current source, Ketho parity, a client
runtime, the full corpus or launch acceptance.

## Explicit named function declarations

Use `--source-declarations` instead of `--source-observations` on build/retire,
and include it on `search observations` queries of that exact component. This
selects `project_source_declarations_v1` with its own profile digest and
`wow-search/source-observation-declaration-query/1` request. The request accepts
the additional `function_declaration` kind. Original omitted selection, handles,
profiles and result bytes are preserved; the two query schemas cannot be mixed.

This mode includes the original local/member corpus and all supported named
function bindings from the genuinely acquired native function report. The
`ProjectView::function_binding_metadata` source custody join supplies the name,
original function/report IDs, source handle and parent scope in both ordinary and
direct-platform layouts; it does not attest accepted Graph membership.
The native adapter reads ordinary local/global
function declarations and static dot/colon paths from already held AST tokens.
It preserves exact token/name/declaration spans, source case and colon spelling;
trivia between tokens is excluded from the recorded composed name. This is a
syntax name, not a canonical WoW namespace or a runtime declaration/target claim.

Exact-name/prefix/similarity queries consider the qualified and short names;
receiver/member lookup uses the recorded receiver path and short name. Similarity
charges cells across both fields with one prefilter charge per document and keeps
the best distance without duplicating the occurrence. Original literal FTS indexes
the qualified recorded name. Long fields and exhausted work remain explicit.

Chunks, anonymous/assigned/table/argument/returned closures, extended/dynamic or
missing syntax and over-cap names are separate recorded coverage states. An
assignment to a closure or a local alias does not create a function binding name.
The adapter bounds each name before copying at 128 components/4096 bytes and
checks cancellation during traversal. The private native name cache is skipped
by all original fact/report/Project metadata serializers, so no legacy replay,
Graph fingerprint or report identity changes. Search captures its own data DTO;
serialized Search metadata cannot reconstruct native authority.

Global variable declarations, explicit aliases and cross-generation relationships
remain separate owner work. This selection does not claim complete Lua symbol
inventory, current WoW correctness, runtime loading or platform availability.
