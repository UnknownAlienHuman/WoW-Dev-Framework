# Retained Blizzard UI Source Candidate lineage

Issue #180 adds an explicit Source profile to the existing `wow lineage`
commands. It compares two exact persisted generations in one selected native
Source namespace. Input is local retained data; these commands do not acquire or
refresh a Gethe checkout, resolve Current, execute source, or claim client/API
compatibility.

Use the same explicit `--store-root`, `--lineage-store` and `--request-json`
transport described in [LINEAGE_CANDIDATES.md](LINEAGE_CANDIDATES.md), with:

| Command | Exact request schema |
| --- | --- |
| build / abandon | `wow-service/source-lineage-build/1` |
| legacy whole-snapshot compare / retire | `wow-service/source-lineage-read/1` |
| status / validate / entity compare / explain | `wow-service/source-lineage-query/1` |

Source requests require a `source_selection` object with the existing
`wow-service/platform-project-selection/1` shape:

```json
{
  "schema": "wow-service/platform-project-selection/1",
  "owner_project_id": "<original owner ProjectId>",
  "source_profile_id": "<original Source ProfileId>",
  "catalogs": {
    "xml_schema_classification": false,
    "physical_function_declarations": "absent",
    "physical_function_signatures": "absent",
    "composed_candidate": true
  }
}
```

Catalog choices must match both original stored generations. The selection is
data-only intent. Service opens the exact namespace, reconstructs the actual
sealed native candidate and original Project/Graph owners, and validates their
association. Decoding this object grants no native or review authority. Missing,
null, unsupported, wrong-namespace or mismatched selection is refused. Ordinary
request schemas require the selection to be absent.

Build fields and finite bounds otherwise match the ordinary request. Read/query
guards identify the original Source build operation/digest and exact Source
snapshot, with the original Store limits and selection. Query operations retain
the same closed role-bound entity and proposal/component grammar. Abandon has a
separate operation ID and independent `--max-output-bytes`.

Source uses its own fixed Store profile
`wow-graph.lineage-source-span-store.v1`, input profile
`wow-graph/lineage/blizzard-ui-source-span-candidates/1`, input schema
`wow-graph/lineage-source-span-input/1`, and snapshot schema
`wow-graph/lineage-source-candidate-snapshot/1`. An ordinary Store is not upgraded
or reused as a Source Store. Original ordinary profile/read/record bytes remain
unchanged.

The native Source pair borrows both original live read custodies and genuine
`PlatformIndexSourceParts`. The existing Project owner checks Project Arc
identity, native prefix membership, foundation, registry, source context and
namespace. Before/after graph universes intentionally differ with source snapshot
identity and are preserved independently. No entity IDs are merged or rebound to
the other revision.

The fixed Source comparison policy requires the same logical namespace/owner,
Source evidence class, product/channel/flavor/edition, analyzer pin, native recipe,
registry and project generation schema. Exact source/target/build/Interface and
configuration differences remain recorded under the Candidate byte-span policy;
they do not prove compatibility, build chronology, move, rename or continuity.
Origin, materializer, source/profile/content/admission digests, license state,
configured roots, root inventory, omission counts and original unevaluated
capabilities remain visible. Mirror labels and asserted materializer/license
metadata are not client, Git membership or redistribution authority.

Project reuses the original native function report, native assertion-address
map, accepted ownership/source/evidence, and existing exact-span primitive.
Comments and whitespace matter; names, paths and rank do not become fingerprint
keys. Graph retains all equal candidates and full ambiguity components under
finite budgets. No source bodies are exported.

Publication and lifecycle use existing typed Store operations, independent
before/after original-generation Policy roots, inactive commit, checked
close/reopen/native validation/seal, read leases, exact retained response replay,
retirement and failed-build abandonment. Fresh reads refuse retired snapshots;
replay returns the original captured observation. Every success waits for checked
resource closure.

Build, compare, status, selected validate and explain remain Candidate-only and
exit 2. Accepted assertions stay zero, negative authority stays false and full
E4-B validation remains `not_evaluated`. Full corpus, Source/Reference transition
semantics, authorized review, accepted lineage, migration/impact, platform fault,
power-loss, fresh Ketho/WoW API MCP, named-client runtime and launch qualification
remain separate obligations.
