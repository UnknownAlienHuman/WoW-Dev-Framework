# Retained ordinary Project Candidate snapshots

Issue #178 implements one selected W21 path. `wow lineage` supports `build`,
`compare`, `retire` and `abandon`; run `wow lineage --help` for transport syntax.
Every command takes an explicit ordinary ProjectStore root, a separate absolute
lineage Store file and bounded strict JSON. Applications call Service once.

`build` and `abandon` consume `wow-service/lineage-build/1`:

```json
{
  "schema": "wow-service/lineage-build/1",
  "operation_id": "original-build-id",
  "before_generation": "project-store-generation:sha256:<64 hex>",
  "after_generation": "project-store-generation:sha256:<64 hex>",
  "limits": {
    "max_entities": 1024, "max_bucket_members": 128, "max_pairs": 16384,
    "max_component_nodes": 128, "max_component_edges": 16384,
    "max_work": 128000000, "max_metadata_bytes": 16777216
  },
  "max_source_bytes": 16777216,
  "store_limits": {
    "max_object_bytes": 16777216, "max_batch_objects": 16,
    "max_catalog_mutations": 16, "max_manifest_records": 10000,
    "max_gc_deletes": 100
  },
  "max_output_bytes": 16777216
}
```

These are explicit example bounds, not corpus-fit qualification. A Store's
configuration and limits must remain identical on subsequent opens. Build output
contains the exact operation digest and snapshot ID. `compare` and `retire` take
`wow-service/lineage-read/1` with `build_operation_id`, `build_request_digest`,
`snapshot_id`, original `store_limits` and a finite `max_output_bytes`.
`retire` and `abandon` also require a separate `--operation-id`. Abandon additionally requires its own finite `--max-output-bytes`, independent of the original failed build response bound.

Project reuses its physical-function source-span primitive over the genuine
borrowed native generation pair, original FunctionCallReport, accepted graph
assertions, source handles and evidence. Comments and whitespace are significant.
The feature manifest retains kind, fixed profile, content digest and byte length;
no source bodies are exported. Generation-local entity identities stay distinct.

Graph groups exact kind/fingerprint buckets and retains every equal cross-role
pair. Its disjoint complete bipartite components preserve all OneToOne,
OneToMany, ManyToOne, ManyToMany, BeforeOnly and AfterOnly shapes. Missing or
conflicted features remain explicit; bounded overflowing buckets are skipped in
full with `truncated` coverage. Work and metadata accounting continue from input
validation through candidate construction and snapshot identity.

The separate identity-aware `wow-graph.lineage-project-span-store.v1` uses the
existing wow-store SQLite/object/catalog owners. Service records operation and
exact source-hold intent before dispatch, installs independent before/after
Policy roots, commits inactive data, checks close/reopen, validates against native
expected data, then atomically seals catalog roots with the result journal. A
fresh compare reopens the original exact owners and returns validated stored
bytes. Same operation/digest replay returns the original receipt; changed requests
fail. Retirement removes snapshot roots before releasing source roots. Abandon
fences an unfinished build, removes its inactive target only if no sealed roots
exist, and releases exact holds; retry completes interrupted release. Old object
and journal bytes remain immutable.

All results are Candidate-only with zero accepted assertions, promotion
`not_evaluated` and no negative authority. Build/compare exit 2; completed
retire/abandon exit 0. This subset does not implement authorized review,
stable-identity/Reference/Search/structural-change producers, accepted continuity,
rename/move/copy/split/merge conclusions, removal/introduction, migration or impact.
Full E4-B/E4-C, corpus, platform fault/durability, current Ketho/client/runtime and
launch acceptance remain separate under #125, #118 and the existing qualification
Issues. This path does not change Current or generation-local graph bytes.

## Exact native read/query profile (#179)

`wow lineage status`, `validate` and `explain` consume one strict
`wow-service/lineage-query/1` request through the same explicit `--store-root`,
`--lineage-store` and `--request-json <ABSOLUTE-FILE|->` options. `compare` also
accepts this query schema for exact entity comparison. Its original read/1 input
continues to return the exact whole saved Candidate snapshot as inspection.
The CLI verb must match the closed `query.operation`; a status request under
compare, display name or rank is refused before a service call.

The query request fields are:

```text
schema = wow-service/lineage-query/1
operation_id = new exact query operation ID
build_operation_id + build_request_digest = original build receipt guards
snapshot_id = exact original saved snapshot
store_limits = original physical Store configuration
query = one closed operation below
limits = finite max_work, max_metadata_bytes, max_output_bytes,
         max_components, max_entities, max_proposals
lease_expires_after = explicit positive Store logical epoch, at most i64::MAX
```

Closed operations:

```json
{"operation":"status"}
{"operation":"validate"}
{"operation":"compare","before":{"role":"before","node_id":"<exact saved node ID>"},"after":{"role":"after","node_id":"<exact saved node ID>"}}
{"operation":"explain","target":{"kind":"proposal","id":"<exact saved proposal ID>"}}
{"operation":"explain","target":{"kind":"component","id":"<exact saved component ID>"}}
```

The original build guard fixes the before/after generations and profiles; a
query cannot refresh or override them. Compare requires both exact role-bound
entities. It returns their whole relevant components and every alternative,
including the case where the requested pair was not proposed. Unknown entities
or wrong roles fail. Explain returns the whole exact component closure, original
native entity/ownership receipts and parent SourceFile node, joined typed source
handles and evidence, original producer partitions, coverage, conflicts and
before/after deferred metadata. No prefix of a component is called a complete
explanation: insufficient item/work/byte bounds fail.

Each fresh observation registers its exact operation/digest, holds both genuine
native owner reads, and acquires real Store leases for the snapshot and validation
objects. Graph checks reverse lease release on every success or error. Service
persists the captured result only after the read view and native pair close, then
checks both outer Store closures before delivery. The read budget is explicit and
separate from the original native producer/requalification budget. Semantic result
identity excludes observation counters; the Store result object binds full bytes.

Retry with the same operation and request returns the original recorded native
observation, including after snapshot retirement; it is not a fresh validity
claim. Use a new operation ID for a fresh read. Changed requests conflict. A fresh
read after retirement fails. Query execution changes no snapshot, source hold,
Current pointer, review or accepted assertion. Logical lease expiry is not a
wall-clock or power-loss durability claim.

Successful query delivery exits 2: results remain CandidateOnly or Truncated.
`selected_validation` covers only the actual ordinary Project source-span
Candidate contract; `full_e4_b_validation` remains `not_evaluated`. Queries do not
create future assertion/review/change/migration/impact results as empty successes.
