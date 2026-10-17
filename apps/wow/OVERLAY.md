# Explicit private Lua overlay check

`wow overlay check --request <request.json>` calls one concrete Service operation.
Project registers the explicit existing local absolute root, opens the supplied
private Lua document, applies guarded atomic changes, lends the exact final
snapshot to the existing compiled Emmy syntax adapter, and closes its resources
before returning metadata and diagnostics. It never saves the supplied source.

This is an ephemeral single-document consumer of the E7-A Project owner. The
workspace state is `registered_unmaterialized`, with `no_published_base`.
Published dependencies, Reference facts, Graph, Rules and runtime are
`NotEvaluated`. Empty diagnostics covers only the existing selected analyzer
families; it is not complete project or WoW correctness. LSP, daemon, multi-request
frontend sessions and the fixed MCP profiles are separate capabilities.

## Request

The strict schema is `wow-service/private-overlay-check/1`; every shown field is
required. Unknown fields, explicit null ranges and unsupported encodings fail.

```json
{
  "schema": "wow-service/private-overlay-check/1",
  "workspace_root": "C:/explicit/existing/workspace",
  "workspace_id": "workspace:explicit",
  "privacy": "session_private_metadata_and_diagnostics",
  "analyzer_report": { "path": "compatibility.json" },
  "limits": {
    "max_documents": 1,
    "max_document_bytes": 1048576,
    "max_private_bytes": 16777216,
    "max_live_snapshots": 8,
    "max_leases": 4,
    "max_lines": 65536,
    "max_edits": 64,
    "max_work": 67108864
  },
  "document": {
    "relative_path": "main.lua",
    "client_version": 0,
    "content": "local value = 1\n"
  },
  "changes": [],
  "output_encoding": "utf16",
  "reconcile_disk": false,
  "max_output_bytes": 1048576
}
```

The compatibility report is an explicit artifact relative to the request file;
its canonical digest and compiled backend identity are validated by the existing
Emmy owner. The artifact can additionally declare `content_digest` and
`byte_length` together using the existing Project disk-input schema. It is never
fetched, generated or selected implicitly. A synthetic compatibility fixture does
not qualify the moving upstream donor.

Each `changes` element contains `expected_version`, `expected_content_digest`
(`sha256:<64 lowercase hex digits>`), `client_version`, `encoding` (`utf8` or
`utf16`), and nonempty `edits`. A client version must equal the prior version plus
one. An edit has `text` and optionally `range` with zero-based `start`/`end`
`line`/`character` positions. Omitting range replaces the entire document.
Ordered edits refer to successive intermediate documents. Any invalid edit
refuses the complete batch and preserves the prior immutable snapshot.

Project alone owns byte/position conversion. UTF-8 positions count bytes;
UTF-16 positions count code units. Code-point/surrogate splits and positions
inside CRLF are refused. The CLI binds each change to its exact preceding
snapshot after validating the caller's expected version and digest.

## Ownership, limits and disk state

Registration normalizes lexical separators once for both the directory
capability and canonical URI. The local path policy refuses traversal, device,
UNC, ADS and symlink/reparse traversal; descendants are opened relative to held
directory handles. Only explicitly requested disk reconciliation reads the Lua
path. Registration does not scan, watch, infer a project or read source files.

The opaque process-local session token is authority to its workspace. Public
session/registration/document/snapshot IDs are metadata, not transferable access
tokens or durable Project generation IDs. Independent registrations remain
isolated. Leased old snapshots retain their original bytes and descriptors.
Document/workspace close reports Busy until their actual leases finish; default
private source is released synchronously after close.

Limits bound document count and bytes, retained private source/line indexes,
leased analyzer-input source copies and conservative edit-copy scratch, live snapshots, leases, lines, edits and work
per owner call. These are charged owner limits, not total analyzer/process heap
measurements. The existing analyzer and disk-input owners retain their own finite
limits. Disk reconciliation currently uses the existing 1 MiB per-file reader;
a larger source refuses rather than claiming equality. The Service request is
bounded by the existing 32 MiB configuration reader and at most 4096 changes;
output is bounded by the selected limit, at most 16 MiB.

`reconcile_disk: false` returns `content_acknowledged_only`. True reacquires the
explicit file and returns `disk_reconciled` or `conflict_with_disk`. Neither
publishes a saved generation. Missing/unsafe files and cancellation refuse.
Default result/error output contains no source bodies or absolute workspace URI.
The input JSON itself contains caller-supplied private bytes; its storage and
retention remain under caller control.

## Result and exit

`wow-service/private-overlay-check-result/1` includes closed workspace metadata,
the final exact overlay and transition descriptors, save observation, actual
Emmy report, projected diagnostic positions and explicit nonclaims. Positions
correspond in order to diagnostics; absent spans remain unavailable. A refusal
does not produce a partial success result. Mandatory lease/workspace cleanup
also runs on analysis failure or cancellation. Ctrl-C uses the existing stop
flag; an already running synchronous analyzer call observes it at its existing
owner checkpoints.

Stdout is one JSON result plus LF. Exits: 0 selected families have no diagnostics;
2 diagnostics; 3 prior identity/resynchronization failure; 64 invalid input;
130 cancellation; 4 other refusal/output failure.
