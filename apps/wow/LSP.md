# Selected private Lua syntax LSP profile

```text
wow lsp --transport stdio --profile lsp-3.18-stdio-v1 --host host.json
```

This foreground profile uses LSP 3.18 Content-Length stdio framing and one
synchronous Service session over one explicitly configured Project workspace.
Project owns all private document bytes, versions, URI validation and UTF-8/UTF-16
coordinates. The existing compiled Emmy adapter analyzes the exact final overlay.
No source is saved or executed.

The provider identifier is `wow.private-syntax`. It covers only the existing
selected local Lua/documentation syntax and generic diagnostic families.
Published dependencies, Reference, Graph, Rules and runtime are not evaluated.
This profile is an executable subset; full E7-A and named-editor qualification
remain separate. Existing MCP profiles retain their own framing and registries.

## Explicit host

```json
{
  "schema": "wow-service/lsp-private-overlay-host/1",
  "workspace_root": "C:/explicit/existing/workspace",
  "workspace_id": "workspace:explicit",
  "privacy": "session_private_metadata_and_diagnostics",
  "analyzer_report": { "path": "compatibility.json" },
  "limits": {
    "max_documents": 4,
    "max_document_bytes": 65536,
    "max_private_bytes": 1048576,
    "max_live_snapshots": 32,
    "max_leases": 16,
    "max_lines": 4096,
    "max_edits": 64,
    "max_work": 4194304
  },
  "reconcile_on_save": false
}
```

Host fields are strict and required. The report is relative to the explicit host
file, validated by the existing analyzer owner, and may additionally declare its
digest and byte length through the existing Project disk schema. A synthetic
report does not qualify a current upstream donor. No config includes, environment
interpolation, parent search or editor/Git/WoW root discovery occurs.

The absolute local root is registered during `initialize`; any supplied
`workspaceFolders` (at most one) or fallback `rootUri` must match its exact
canonical URI. Neither changes the configured root. An empty folder selection
uses the explicitly configured operator root. The selected path profile rejects
noncanonical URI aliases and unsupported schemes, traversal, encoded separators,
device/UNC/ADS and symlink/reparse acquisition. Dynamic workspace-folder changes
are unsupported.

## Implemented wire operations

| Wire method | Service operation |
|---|---|
| `initialize` | `frontend_session_open` |
| `textDocument/didOpen` | `frontend_document_open` |
| `textDocument/didChange` | `frontend_document_change` |
| `textDocument/didSave` | `frontend_document_save` |
| `textDocument/didClose` | `frontend_document_close` |
| `textDocument/diagnostic` | `frontend_document_diagnostics` |
| `$/cancelRequest` notification | `frontend_operation_cancel` observation |
| `shutdown` | `frontend_session_close` |

`initialized` and `exit` are protocol bookkeeping. Initialize requires client
pull-diagnostic support; no unsupported language feature is advertised. UTF-8 is
chosen when offered, otherwise UTF-16 when offered or when encoding capabilities
are omitted. The encoding remains fixed. The initialize result carries the exact
reviewed registry/schema digests, content-addressed transport profile and limits in
`capabilities.experimental.wow`.

Documents must have language ID `lua` and nonnegative versions within the LSP
integer range. A change requires the next version exactly. Ordered edits refer
to successive intermediate texts; a missing range is a full replacement. The
deprecated `rangeLength` field is not admitted by this initial strict change
profile. Rejected change shapes, stale versions, invalid ranges/encoding or
mismatched save text mark the document as requiring resynchronization. Further
diagnostics refuse with `ContentModified` until `didClose` followed by a full
`didOpen`; an old cached empty or unchanged report cannot mask the refusal.

Save without text acknowledges the private content. With
`reconcile_on_save: true`, Project safely reads the explicit disk path and reports
equality or conflict; it never publishes a generation. A failed new save clears
the previous disk observation. The existing disk reader's 1 MiB file limit can
refuse reconciliation even when a larger private document is admitted.

## Exact diagnostic result and lifetime

Pull results use standard full/unchanged reports plus the profile's `wow` binding
extension, including for empty results. That binding preserves the exact
session/registry/overlay/version/digest, actual analyzer backend/workspace/result
identities, save observation when available, coverage, lease closure and
`NotEvaluated` scope. Each diagnostic's `data.original` preserves the complete
normalized Emmy diagnostic, including its classification, advisory rollout,
severity and original UTF-8 span; LSP positions are exact Project projections.
Messages contain a static family description rather than source text.

Only one metadata result is cached. A matching `previousResultId` can reuse it;
another document or changed snapshot never borrows its result. Private source
and analyzer-input copies remain governed by Project budgets and real leases.
Operation leases close before a result is delivered; the session workspace stays
owned until close. Shutdown, abnormal EOF, framing/output failure and a caught
panic synchronously close the session. Default logs and errors exclude source
bodies and absolute workspace roots.

## Bounds and limitations

There is one active synchronous request and no semantic queue. Headers are at
most 8 KiB; input/output bodies are at most 4 MiB; JSON nesting is at most 32.
Headers are ASCII CRLF with one Content-Length and an optional UTF-8 Content-Type;
duplicate/unknown headers, unsupported encoding, overflow and truncation refuse.
Output size is measured before encoding/writing a frame. Diagnostic count is
bounded at 2048 and retained report bytes at 2 MiB. Project limits still apply;
host document bytes may not exceed 2 MiB. These are charged bounds, not measured
total process/analyzer heap guarantees.

An already running synchronous operation cannot consume a wire cancellation.
The cancel operation reports only an exactly observed completed ticket or no
cancellation claim. Process stop reaches the existing owner checkpoints; an
upstream synchronous call completes before observing it. Progress, asynchronous
wire cancellation, reconnect, transport deadlines and durable response journals
are not implemented. No hover, definition, completion, workspace diagnostics,
push diagnostics, commands, edits, daemon or network listener is advertised.

Stdout contains framed messages only. Graceful shutdown/exit is 0; abnormal
exit/EOF or framing failure is nonzero; mandatory-close/output/internal failure
is 4; observed process cancellation is 130. Unadvertised requests return
MethodNotFound, resynchronization returns ContentModified, and notification
refusals emit bounded redacted `window/logMessage` notifications.
