# Read-only retained graph commands

```text
wow graph subgraph --snapshot graph.json --request subgraph.json --format json
wow graph axis     --snapshot graph.json --request axis.json --format text
wow graph explain  --snapshot graph.json --request explain.json --format json
```

`graph.json` is the complete serialized `wow_graph::GraphPartitionSnapshot`
(`wow-graph/partition-snapshot/e2-a/1`), including its foundation, registry,
producer partitions and materialized snapshot. It is **not** a check result,
subgraph, bare `GraphSnapshot`, source directory, or an editable list of edges.
[Graph build](GRAPH_BUILD.md) exports a first-party file/direct-load snapshot from
the existing project configuration. Native graph-owner consumers may also export
complete partition snapshots. The three read commands below do not create a graph
or acquire coherent ProjectStore; semantic producer construction remains separate.

Every command invokes `wow_service::graph::execute_graph_read` once. The app
reads explicit bounded artifact bytes and prints the returned result. The service
validates the complete imported graph through its owner and invokes the existing
subgraph/axis/explanation implementation. There is no Lua session, project/source
scan, current pointer, database, publication, implicit retry or continuation.

## Request format

```json
{
  "schema": "wow-service/graph-read-request/1",
  "query": {
    "operation": "subgraph",
    "parameters": {
      "snapshot_id": "<exact materialized graph-snapshot:sha256:...>",
      "roots": ["<exact graph-node:sha256:...>"],
      "direction": "outgoing",
      "relations": ["calls"],
      "confidence": "proven_and_derived",
      "limits": {
        "max_depth": 4,
        "max_nodes": 256,
        "max_edges": 1024,
        "max_scanned_edges": 500000,
        "max_expansions": 100000,
        "max_output_bytes": 1048576
      }
    }
  }
}
```

The angle-bracket placeholders must be replaced with IDs from the retained
artifact. Display names, logical paths and `current` are not root selectors.
The operation must match the CLI subcommand. Parameters are the unchanged
serialized owner query; duplicate/noncanonical collections reject at execution.

For `axis`, parameters are a `GraphAxisQuery`: `snapshot_id`, `axis`, exact
`profile_digest`, `roots`, `traversal`, `confidence`, and subgraph `limits`.
Obtain the profile with `GraphAxisProfile::bind` against that artifact's exact
registry when preparing the request. Execution rebinds the reviewed recipe and
requires the identical digest; no arbitrary profile is selected from a name.
See [AXIS_USAGE.md](../../crates/wow-graph/AXIS_USAGE.md).

For `explain`, parameters are a `GraphExplainQuery`:

```json
{
  "schema": "wow-service/graph-read-request/1",
  "query": {
    "operation": "explain",
    "parameters": {
      "snapshot_id": "<exact graph-snapshot:sha256:...>",
      "subject": {"kind": "relation", "id": "<exact graph-edge:sha256:...>"},
      "limits": {"max_scanned_assertions": 500000, "max_supports": 128, "max_output_bytes": 1048576}
    }
  }
}
```

Use `kind: "entity"` and a node ID for entity explanations. See
[EXPLANATION_USAGE.md](../../crates/wow-graph/EXPLANATION_USAGE.md) and
[SUBGRAPH_USAGE.md](../../crates/wow-graph/SUBGRAPH_USAGE.md). Native callers can
use the public service reexports and `GraphReadRequest::new(GraphReadQuery::...)`.
The application itself imports no lower framework crate.

## Result and authority

`wow-service/graph-read-result/1` contains the operation, status, exact validated
snapshot/universe/generation/source-context/registry, original owner payload,
explicit boundaries, and structured failure when applicable. Failures never
include a substituted or partly successful payload. Source/proposal strings and
upstream error prose are not copied into error messages.

`snapshot_input_digest` and `request_input_digest` identify accepted-size raw
transport bytes. `request_digest` hashes the canonical typed request. The
`result_digest` hashes the canonical envelope **without** the result-digest field;
all hashes use SHA-256 with `sha256:` encoding. Object keys use the existing core
canonical ordering. Input whitespace/key order can change transport digests, not
the canonical query digest or the graph's own identity. Host filenames, cwd,
timestamps and credentials never enter semantic output.

The outer status preserves the exact owner projection's Complete/Partial/
NotEvaluated/Truncated state. Explanation results remain Partial while their
external evidence/derivation/conflict boundaries are unresolved, even when all
retained supports were enumerated. Axis and subgraph payloads keep all original
coverage, evidence, confidence and truncation details. The outer
`absence_authoritative` is always false: a standalone retained artifact does not
certify a coherent live project or client state. A narrower owner absence flag,
where present, applies only to that exact stored-graph query.

`text` prints a status heading and the complete escaped, pretty-printed receipt;
it does not omit evidence, blockers or partial states. JSON is the canonical
service envelope followed by a transport newline. Both modes have identical
semantic status.

## Bounds, cancellation and exits

Input: 16 MiB snapshot, 64 KiB request, 64 nested JSON levels, 16 KiB decoded
strings/keys, 1,000,000 snapshot values/keys, and 16,384 request values/keys.
Duplicate decoded JSON keys, unknown fields, nulls, floats and malformed input
reject before domain interpretation. Integer values still undergo the owner's
canonical validation. The preflight retains only object keys and counters; it
does not construct another DOM. Admission limits are service-owned, not fields
that an input artifact may raise.

File reads check the open file's type and size and stop at the byte ceiling,
including growth after opening. The selected leaf must be a regular non-symlink
file. This explicit-artifact reader is not a sandboxed source-root walker and
does not promise protection against concurrent ancestor/path replacement.
No arbitrary-directory trust is derived from this input form.

Owner limits apply unchanged to query work and payload. The entire canonical
service envelope is additionally bounded to 8 MiB + 64 KiB; text transport to
32 MiB. These are independent ceilings, not invisible additions to an owner's
payload budget. An oversized result fails rather than removing evidence.

Cancellation is checked during file reads, JSON preflight and owner work, and
before returning a result. Files close before service invocation; graph views
close before the envelope returns. A late signal may replace an unwritten result
with cancellation without invoking the operation again. Individual open/read,
Serde, canonicalization and owner-validation calls are not forcibly interruptible.
No durable lease or crash-recovery guarantee is claimed by this one-shot route.

| Exit | Meaning |
|---|---|
| 0 | Complete exact owner projection, not clean project/runtime certification |
| 2 | Partial, NotEvaluated or Truncated |
| 3 | Structured request, identity, input, unsupported-operation or budget failure |
| 4 | Encoding/internal failure or output loss, including broken pipe |
| 64 | CLI syntax, signal setup or artifact-file acquisition error |
| 130 | Cancellation |

Subgraph/axis/explain are the only operations in this version. No path cursor,
current selector, source mutation or generic service invocation is exposed. E2,
E3-C, E7 and public product acceptance are not advanced by this transport slice.
