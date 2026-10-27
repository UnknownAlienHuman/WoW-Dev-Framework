# Active Issue instruction audit

**Audit base:** `main` after PR #269
(`f674ccc7615d63c76c0781aef2150e49ea442bf8`)  
**Scope:** all 21 open implementation Issues  
**Source traces:** T01–T39

Issue bodies are task instructions, secondary to current status, queue,
implementation guide and trace register. They must not copy delivery history,
widen the active slice or preserve a removed result shape.

## Register

| Issue | Status | Durable boundary |
| --- | --- | --- |
| #103 recognizer rules | confirmed | keep only typed facts with a named diagnostic/search/navigation consumer |
| #106 Blizzard UI workflow | confirmed | later direct body/symbol/search/context workflow |
| #107 graph duplication | corrected | use loaded/generated Library and direct origins; closed #196 is historical delivery |
| #109 coherent updates | corrected | update one target/scope/root-authority/session owner; no second session/profile identity |
| #111 Store lifecycle | corrected | caller census is a source snapshot; recheck before deletion |
| #112 formats/documentation | corrected | small target/plan/input-origin/result formats; no profile/plan-version replacement platform |
| #113 performance | confirmed | measure one real workflow only after it exists |
| #117 TOC/XML relationships | reconciled | Slice 1B+ context/index work; XML does not block Slice 1A physical diagnostics |
| #118 lineage/fingerprints | confirmed | remove remaining fingerprint/proof plumbing with real callers |
| #122 readable context | confirmed | direct located result over exact indexed text |
| #125 product completion | reconciled | final observed workflows, not a progress ledger |
| #131 direct reads | confirmed | later-process direct index read without Project/Emmy/Graph replay |
| #169 direct FTS | confirmed | body/symbol rows in the same direct current index |
| #191 umbrella | corrected | moving delivery list removed; current status/queue/trace own state |
| #194 result DTOs | corrected | one Service-owned assembly; only requested-scope blockers enter the result |
| #195 recognizer handoff | corrected | old commit census is historical; recheck actual callers before each removal |
| #197 CLI | corrected | strict explicit config, one Service call, root-relative results, typed stderr errors and exit precedence |
| #198 LSP | reconciled | later adapter calls the same public Service operation, never the lower request constructor |
| #199 MCP | reconciled | later standard profile calls the same public Service operation |
| #200 Project input | corrected | target/scope plan, physical projection, root/file authority and structured failures |
| #201 end-to-end | corrected | first observation covers authority collisions, partial input and actionable errors |

## Material source corrections

### Root overlap is a normalized input-path problem

Duplicate root and snapshot-ID checks are insufficient. Snapshot identity includes
universe/content, so one selected host path may receive different Main/Library
IDs; nested recursive Library roots may overlap while whole snapshots differ.
A role/ordinal root slot plus relative path alone is insufficient because the
same path has a different relative spelling under nested roots.

Project must wrap crate-private `open_workspace`, retain request-local
`InputRootId` (role plus ordinal) and a nonserialized lexical absolute root
descriptor, then derive one conservative portable case-folded
`InputPathAuthority` per admitted Lua using Project's existing case-collision
policy. Case-only spellings intentionally collapse, but inode/content/hard-link
identity is not used. Reject duplicate path authorities across roles/roots, not
root ancestry. Equal relative paths in distinct roots remain valid when joined
authorities differ; identical whole Library snapshots remain duplicate input.
Legacy ambient `ProjectInputDirectory::open` is not the Slice 1A root boundary.

### The frontend cannot be the trusted assembler

The lower request accepts caller-supplied snapshots and arbitrary limitations.
Without one Service-owned ordinary operation, CLI/LSP/MCP could combine a plan's
blockers with unrelated Main bytes. Service owns Project preparation, plan/Main
binding, root/file-authority validation, requested-scope limitation projection,
snapshot construction and the call to the lower check. Frontends resolve ordinary
config and render only.

### Input failures require structured origin

Current Library parse failure keeps only a relative path, loses it at the
configured-session boundary and becomes internal component failure. Carry source
role, snapshot/`InputRootId` and relative path as fields; Service maps the snapshot
to request-local `InputRootId` and returns a small operation-specific error whose
public class is only `invalid_input`, `cancelled`, or `internal`. Project attaches
root ID/path to every file-specific acquisition error. Source/workspace/diagnostic/
result-size budget refusal is invalid input; analyzer invariants, serialization
defects and output I/O are internal. Message parsing, absolute paths and synthetic
analyzer roots are not admissible provenance; do not expand global `ServiceError`
into a provenance platform.

### CLI success and error channels are separate

Strict explicit config is one regular no-follow file read through the narrow
Project config helper. It reuses duplicate-key/depth-bounded `StrictJson`, rejects
unknown outer fields and is capped at 256 KiB. Merge config and CLI before validating
required fields; defaults are text, `fail_on: error` and upstream Emmy diagnostic
settings. Library input is capped at 64 roots,
8,192 files and 256 MiB aggregate; each root must yield Lua. Complete/partial
results are pre-encoded under a 32 MiB cap and written once to stdout. Pre-result
errors go once to stderr; JSON leaves stdout empty. Exceeding the public result
size is invalid input (`64`); serialization invariants, write failure and
cancellation-handler setup are internal (`4`). Check cancellation after encoding
and before the first byte; after commit, no retry/replacement/second envelope.
Error messages are capped at 4 KiB; `root_index` is Library-only and zero-based,
and paths are relative to the role's configured root.
Explicit Error > Warning > Information > Hint ranking controls `fail_on`; visible
`Unknown` severity conservatively triggers every threshold. Diagnostics and
limitations use explicit role/path/span/code ordering in both text and JSON.

### Partial input is not the same as invalid input

A missing addon root/explicit TOC, unsafe path or bytes, unrecoverable TOC syntax,
resource-bound refusal or malformed Library is a pre-result `invalid_input`. A
missing direct Lua named by a safely parsed TOC, unresolved required dependency or
condition, deferred XML, selected environment uncertainty or actual top-level
chunk ellipsis is a located partial result and preserves independent diagnostics.
Ellipsis ownership is determined by a bounded collector on the same configured
Emmy AST (`LuaLiteralToken::Dots` without an enclosing `LuaClosureExpr`), not by
raw text or a second parse/session. It retains the lowest-starting exact occurrence
per affected file.
Repeated Main occurrences, Interface without a positive target, `LoadWith` and
unused lifecycle/display/custom metadata remain observations for `LuaTypes`.

Slice 1A exposes no WoW-check option. It runs configured Emmy diagnostics and zero
WoW rules. Human text escapes control characters; typed/JSON strings stay exact.

### Pinned diagnostic configuration is not strict by itself

At the pinned EmmyLua revision, `EmmyrcDiagnostic` has no
`deny_unknown_fields`, and `LuaDiagnosticConfig::new` logs then drops invalid
`globalsRegex` entries. Slice 1A therefore prevalidates the exact nested key set
and compiles every expression before constructing upstream configuration. Unknown
or invalid nested settings are `invalid_input`, not silently repaired behavior.
`diagnosticInterval` is rejected by the direct checker: the pinned code-analysis
crate never reads it, while LS handlers use it only for debounce. It cannot be a
no-op CLI setting or part of analyzer-owner compatibility.

## Maintenance rule

When an owner/result boundary changes, update current status and trace; update the
queue only if order changes; update only affected Issue bodies. Before deleting
code or trusting an old census, recheck current production callers.
