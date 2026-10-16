# ProjectEntity shards and exact-seed graph retrieval

This separate component projects every materialized GraphNode from one genuine
coherent retained Project/Graph pair. It preserves original node ID, universe,
generation, kind, opaque owner key and evidence IDs. An over-cap complete
projection refuses publication; it never silently indexes a prefix of the owner.
Opaque owner keys are identity metadata, never inferred names/roles or bridges.

## Build and custody

Use [durable Search build](SEARCH_BUILD_QUERY.md) with `--graph-entities` instead
of either file selector:

```text
wow search index build --graph-entities --store-root <ORIGINAL-PROJECT-STORE>
  --generation <EXACT-STORE-GENERATION> --search-store <SEARCH-STORE-FILE>
  --operation-id <ID> --allow-local-metadata-only
  <SEARCH-BOUNDS> <STORE-BOUNDS> --max-output-bytes <LIMIT>
```

The component is `project_graph_entities_v1`; it conflicts with `--file-id` and
`--file-ids` and needs no fabricated representative file. Preserve the complete
build response: operation ID, Search handle/profile, original Source epoch,
generation hold ID and digest. Existing per-physical-Store holds, Prepared/
unknown reconciliation, atomic catalog/response completion and retirement apply.
Single/FileSet profiles, hold identities and response bytes remain unchanged
when this component is absent. Native Store owns SQLite/FTS, validation, leases
and GC; no second graph, traversal, search backend or persistence layout exists.

## Exact retained query

### Query-wide original entity-kind scope

Use `schema: "wow-search/project-entity-filtered-query/1"` with the same query
class and bounds, plus required `filters: {"entity_kinds": ["lua_source_function"]}`.
The closed list is nonempty, unique, at most 512 IDs, normalized in exact
case-sensitive order. Each requested ID must exist in the original digest-bound
Graph registry; an unsupported kind fails the operation instead of producing an
empty success. A registered kind with no records remains a supported empty query.
The old schema rejects this field, including explicit null.

All four query classes restrict returned candidates by `GraphNode.kind()`.
Exact owner keys remain opaque metadata; filtering never produces absence
authority. Linear scans retain their visited-document charges. Excluded hits
do not consume the caller's candidate cap. Related traversal uses its original
seeds, edges, confidence and bounds, retaining original intermediate witnesses;
only returned targets are scoped. It does not prune traversal by kind or
upgrade Candidate authority. Original requested evidence/explanations remain.

The new query interpretation is `wow-search/project-entity-owner-kind-scope/1`;
shards, handles, source holds and old unfiltered request/result bytes stay intact.
Standalone and federated captured continuation bind the original normalized
request and recheck the candidate kind against exact retained Search documents.
They neither rerun Graph/the original query nor permit a cursor filter override.

### Commands and unfiltered request

```text
wow search entities --store-root <SAME-ORIGINAL-PROJECT-STORE>
  --generation <EXACT> --search-store <SAME-SEARCH-STORE>
  --search-shard-id <EXACT> --object-id <EXACT> --profile-digest <EXACT>
  --build-operation-id <ORIGINAL> --source-store-epoch-id <EXACT>
  --source-generation-hold-id <EXACT> --source-generation-hold-digest <EXACT>
  --query-json <CLOSED-QUERY> --operation-id <ID> --lease-expires-after <EPOCH>
  --allow-local-metadata-only <ORIGINAL-SEARCH/STORE-BOUNDS>
  --max-output-bytes <LIMIT>
```

Query checks actual physical SearchStore identity and original hold/epoch, reads
the exact ProjectStore generation and reconstructs its real coherent owners.
The full stored entity projection/registry/snapshot must match. Original project
files need not exist. Wrong source epoch/receipt, missing hold or retired/corrupt
shard fails; no repair, rebuild, public-report surrogate or Current fallback.

```json
{
  "schema": "wow-search/project-entity-query/1",
  "query": {
    "class": "related",
    "seeds": ["<EXACT-GRAPH-NODE-ID>"],
    "expansion_profile": "structure",
    "direction": "outgoing",
    "confidence": "include_possible",
    "limits": {
      "max_depth": 4, "max_nodes": 64, "max_edges": 128,
      "max_scanned_edges": 100000, "max_expansions": 100000,
      "max_output_bytes": 1048576
    }
  },
  "max_scanned_documents": 128,
  "max_candidates": 64,
  "explanation_limits": {
    "max_scanned_assertions": 100000, "max_supports": 128,
    "max_output_bytes": 1048576, "max_derivation_depth": 16
  },
  "max_total_support_scans": 1048576,
  "max_result_bytes": 4194304
}
```

Other classes: `exact_entity` with `entity_id`, `exact_owner_key` with original
`owner_key`, and `literal_owner_key` with bounded literal `text`. Key equality
is not a canonical name. Literal FTS is Candidate and treats text as data.
Unknown fields, malformed IDs, controls, invalid limits, duplicate/empty/over-eight
seeds and unsupported profiles reject. Exact identity is band `0`, recorded key
band `1`, literal metadata Candidate band `7`; no exact miss grants absence.

## Graph semantics and evidence

Only exact caller-provided nodes in this Graph snapshot may seed expansion.
Candidate seeds and cross-universe bridges are unavailable. Repository-owned
relation profiles are closed:

| Profile | Original relation whitelist |
| --- | --- |
| `structure` | contains, defines, owns, loads, loads_before, depends_on, optional_depends_on |
| `code` | calls, uses_api |
| `state` | reads_state, writes_state |
| `callbacks` | native/custom event register/handle/bridge, CVar callback, script/hook relations |

Direction/confidence are Graph owner enums; Candidate edges require explicit
`include_candidate`. Existing GraphSubgraphQuery executes bounded traversal.
Search reconstructs **one canonical owner discovery witness per target**, not
all paths. Roots do not become related self candidates. Original graph cycle,
missing/partial coverage and depth/node/edge/scan/expansion/output stops remain
visible. Graph no-hit is nonauthoritative.

Each path retains seed/target, ordered original edges and separate traversal
endpoints/direction. Incoming traversal creates no inverse or transitive edge.
Weakest confidence preserves per-edge ceilings, including Possible. Graph
retrieval is Candidate band `9`: unit edge cost/depth, weakest confidence, exact
target ID. Candidate caps retain that canonical closest order and report truncation.

Selected node/edge explanations execute through the original owner under
per-query and cumulative accepted-assertion scan bounds. The closed
`wow-search/graph-support-metadata/1` projection retains producer/batch/report/
proposal addresses, accepted input-generation records, source/evidence/coverage
handles, confidence, derivation/conflict rows, boundaries and truncation.
Arbitrary semantic-key properties are explicitly not delivered by this metadata
profile; exact original owner addresses support separately permitted detail.
Missing owner assessment/resolution is never invented or promoted to success.

Shard lease, held source read and SourceStore close before success. Completion
CASes the exact shard catalog so retirement cannot resurrect it. Exact completed
query replay is historical and needs no source read. Optional
[whole-candidate result pages](SEARCH_ENTITY_PAGINATION.md) retain the exact Search
capture and shard, then continue without recomputing native Graph owners.
Metadata indexing is local with explicit permission;
license/current/runtime acceptance is not implied.

## Retire and remaining acceptance

Use ordinary exact `search index retire` receipt/bounds with `--graph-entities`
and no file selector. The complete component and original hold are reconciled
before its retention is removed. Wrong component/profile/source receipt cannot
release another store's hold. Existing `search index gc` collects released shards.
Other source holds stay independent.

Graph mechanics proof remains separate from current WoW/Tier X/runtime,
all-path enumeration, other documents/aliases/federation and full
E4/launch acceptance.
