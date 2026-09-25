# Graph-build bundle reads and resolved explanations

```text
wow graph build --config project.json --project my-addon --format json > build.json
wow graph neighbors --bundle build.json --request neighbors.json --format json
wow graph explain --bundle build.json --request explain.json --format json
```

Use a fresh output path for shell redirection. A produced build receipt has exit
2 (Partial), not a construction failure. Preserve it rather than rerunning source
analysis to retrieve its evidence. The new read route accepts graph-build **v7**;
older receipts remain readable as their bare `snapshot` through `--snapshot`.
There is no implicit schema migration, latest-generation selection or fallback.

All six read commands (`entity`, `neighbors`, `subgraph`, `axis`, `explain`, `path`)
accept exactly one of `--snapshot` and `--bundle`. They use the same exact query
IDs and the same graph owners. Without the separate `--source-root` route below, no script, compiler, source read,
store write or network access occurs. Paths inside a bundle are data, never files
to open implicitly.
Path continuation still requires the same exact graph/query and an explicit
cursor; bundle loading does not follow another page automatically.

## Admission and authority

The service bounds the complete bundle at 32 MiB, 2,000,000 JSON values/keys and
64 nesting levels. Decoded duplicate keys are rejected throughout the document.
Inline-source strings may use the bundle byte ceiling; object keys remain at
16 KiB. The embedded snapshot must separately satisfy the old 16 MiB, 1,000,000
value/key and 16 KiB string profile. Null/floating-point input is not accepted.

The complete canonical receipt (excluding only `result_digest`), build request
and canonical snapshot must match their digests. Graph independently rebuilds
and validates every partition and proposal report. Project admits the source-file
manifest, its source proposals and whole-file evidence, all handle paths/content
checksums/generations/origins and span bounds. Graph binds evidence producers to their context versions. Core validates
source/evidence IDs and the closed evidence derivation DAG, including cycles
and confidence/provenance restrictions. Malformed retained
records fail the read, even when unrelated to the requested subject.

This validates **retained metadata**, not source bytes or the authenticity of an
origin. A checksum is not a signature. The original analyzer, recognizer and
other build sidecars remain integrity-bound data; they are not independently
replayed or certified by this import. Core evidence coverage references stay
references, distinct from the graph coverage records in the explanation.

The new outer schema is `wow-service/graph-bundle-read-result/1`, with transport
bundle digest, original result digest, canonical snapshot digest and admitted
catalog digest. `input_boundaries` retains construction limitations. A query
cannot upgrade the bundle's Partial state to Complete. Failures expose typed
stage/code, not parser messages or source text. The previous `--snapshot` request,
result schema, canonical bytes and behavior are unchanged when the new optional
field below is absent. Graph-build v7 bytes and identity recipes are unchanged.

## Explained evidence

`explain --bundle` returns `payload.explanation` (the existing complete graph
explanation) plus `payload.evidence_resolution`. It follows evidence attached to
the selected node/edge, both endpoints, and every returned producer support,
including proposal-only source handles. Exact `EvidenceRecord` derivation inputs
are expanded breadth-first. Shared evidence and source handles are emitted once.
No producer, provenance, confidence, claim scope or coverage reference is rewritten.

The resolution includes the full generation context, evidence records and original
`SourceHandle` records: relative file, exact byte range, content checksum, revision
and origin. Use the IDs in the original supports to join these tables. Missing
graph references are `not_retained`; opaque legacy evidence IDs are
`unsupported_identifier`, never proof of absence. Missing inputs *inside* a
retained evidence record are invalid catalog data, not a successful partial DAG.

Optional top-level `evidence_limits` is valid only for an explain request using
`--bundle`; it is rejected for other operations or bare snapshots. Defaults:

```json
{
  "schema": "wow-service/graph-read-request/1",
  "query": {
    "operation": "explain",
    "parameters": {
      "snapshot_id": "<exact graph snapshot ID>",
      "subject": {"kind": "relation", "id": "<exact edge ID>"},
      "limits": {
        "max_scanned_assertions": 500000,
        "max_supports": 128,
        "max_output_bytes": 1048576
      }
    }
  },
  "evidence_limits": {
    "max_evidence_records": 1024,
    "max_source_handles": 4096,
    "max_derivation_depth": 32,
    "max_work": 100000
  }
}
```

The limits permit at most 16,384 evidence records, 65,536 source handles, depth
256 and 1,000,000 work steps; depth zero resolves roots without expanding their
inputs. Each seed, lookup and inspected dependency/source reference consumes work.
Catalog admission has separate hard caps: 131,072 records of each type, 1,000,000
references and 32 MiB. It validates all retained data before query selection.

The original `max_output_bytes` bounds the **entire combined owner payload**,
including explanation, context and resolved evidence, not just each part. Records
are returned whole or omitted with explicit truncation. The minimum fixed payload
must fit, otherwise the read fails with BudgetExceeded. Known-frontier counters
are not totals for an unexplored derivation closure. No retry or budget increase
is implicit. Cancellation discards the payload; individual core validation and
serializer calls are bounded but not forcibly interruptible.

`closure_complete` covers only the selected evidence/source closure.
`supports_complete` separately reports contributor enumeration. A complete
closure removes only the unresolved-evidence boundary; it does not invent full
inference-rule records, conflict assessment, source authentication, runtime or
coherent ProjectStore publication. `source_bytes_verified` and
`coverage_references_resolved` remain false. Default output stays Partial, or
Truncated when either graph-support or evidence limits are reached.

Owners: `wow-project::graph::RetainedProjectGraphEvidence`,
`wow-graph::GraphEvidenceCatalog`, `GraphExplainQuery::execute_with_evidence`,
and `wow-service::graph::execute_graph_bundle_read`. No new dependencies or
acceptance/fixture claims are introduced.


## Explicit source verification and excerpts

```text
wow graph explain --bundle build.json --request explain.json --source-root <Main-root> --format json
```

This opt-in route reads only source handles returned by the bounded evidence
resolution, below the explicitly selected **Main project source root**. For TOC
input this is the addon's root, not necessarily the configuration directory.
The bundle is still fully admitted first. No source selector in the JSON can
open a host root; `--source-root` is private local configuration and is never
serialized or included in an identity. Other graph operations and `--snapshot`
reject this option. Omitting it preserves the previous no-source-I/O behavior
and exact result encodings.

The project owner binds the retained file manifest to the exact admitted evidence
catalog. It preflights all selected handles and case collisions, then groups by
canonical relative path and handle ID. It reads each selected file once through
the existing directory-capability loader. Descendant directories and files never
follow symlinks; device names, alternate streams, traversal, nonregular files and
paths outside the Lua/XML/TOC profile cannot be used as source inputs. The ambient
root itself is the caller's explicit capability; this is not an OS sandbox.

Each file must match the **whole-file** recorded length and content digest before
any excerpt is emitted. The reader checks metadata before/after acquisition and
bounds growth while reading. Missing/unsafe files, changed-during-read data,
content mismatch and unsupported encoding/path are separate file outcomes.
Mismatching source text is never returned. No file is rewritten, refreshed,
reindexed or replaced by a Library or other-generation fallback.

`payload.source_read.files` contains per-file outcomes and exact source-handle
excerpts. ByteRange uses the original end-exclusive UTF-8 range; WholeFile means
the whole file. Unknown spans are not widened. Out-of-bounds or split-codepoint
ranges have `invalid_span`; zero-length valid ranges return an empty string.
An excerpt is either returned whole or explicitly omitted, not silently clipped
or expanded to nearby source. XML snippets are original XML bytes, including
CDATA/entities, rather than a synthetic Lua wrapper or decoded body.

`all_requested_files_verified` and `all_requested_spans_verified` refer **only**
to the source handles returned by the preceding evidence-resolution stage. Empty
selection is not verification. Source, evidence and contributor truncation remain
separate; these flags do not cover omitted/missing evidence or all addon files.
The original graph `evidence_resolution.source_bytes_verified` remains false:
that owner performed only metadata validation. The separate project read-back
report carries the observed byte-verification result without rewriting history.

Optional top-level `source_limits` belongs to the request and is rejected unless
`--source-root` is supplied. Defaults (all fields required when specified):

```json
"source_limits": {
  "max_files": 16,
  "max_source_handles": 128,
  "max_file_bytes": 1048576,
  "max_read_bytes": 8388608,
  "max_excerpt_bytes": 16384,
  "max_total_excerpt_bytes": 262144
}
```

Hard ceilings are 256 attempted files, 4,096 handles, 16 MiB per file, 64 MiB total
reserved reads, 64 KiB per excerpt and 4 MiB total excerpt UTF-8 bytes. Each file
reserves its expected size plus one overflow-detection byte before opening;
`reserved_read_bytes` is this conservative upper bound, not measured disk traffic.
Counts include failed attempts, so failures cannot bypass the read budget.

The original explanation `limits.max_output_bytes` bounds the **entire combined
payload**, including retained evidence and source excerpts. JSON-escaped text is
charged separately from raw UTF-8 excerpt bytes. Metadata is budgeted before
reading and text before copying. Insufficient fixed metadata space fails with
BudgetExceeded; later output/record/I/O/excerpt limits give explicit truncation
and exact counts of unrepresented selected handles. There is no hidden rerun
with altered query limits. A large WholeFile excerpt may be omitted while its
content is successfully verified.

This route returns `wow-service/graph-source-read-result/1`; the prior bundle-read
and snapshot schemas are unchanged. Source problems keep the result Partial;
any graph/evidence/source budget truncation gives Truncated. Cancellation drops
the whole payload. Directory/file resources close before CLI rendering, and
broken pipe never repeats the service call.

Read-back proves agreement with recorded bytes at the time of the read, not
repository authenticity, correctness of analyzer/recognizer conclusions, runtime
behavior, a historical checkout, or an atomic filesystem snapshot. Different
files may be observed at different times; changes after a read are not detected.
Excerpts are untrusted source data and may contain private text: explicitly
select the local root and do not redistribute output without authorization.

Owner seams: `RetainedProjectGraphEvidence::admit_with_sources`,
`RetainedProjectSourceManifest::read_sources`,
`wow-service::graph::execute_graph_bundle_source_read`. No new framework edge,
parser, test fixture or complete E2/E3 acceptance claim is introduced.
