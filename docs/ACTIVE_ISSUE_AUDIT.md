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
| #109 coherent updates | corrected | update one target/scope/authority/config/session owner; no second session/profile identity |
| #111 Store lifecycle | corrected | caller census is a source snapshot; recheck before deletion |
| #112 formats/documentation | corrected | small target/authority/result/error formats; no replacement platform |
| #113 performance | confirmed | measure one real workflow only after it exists |
| #117 TOC/XML relationships | reconciled | Slice 1B+ work; XML does not block physical diagnostics |
| #118 lineage/fingerprints | confirmed | remove remaining fingerprint/proof plumbing with real callers |
| #122 readable context | confirmed | direct located result over exact indexed text |
| #125 product completion | reconciled | final observed workflows, not progress ledger |
| #131 direct reads | confirmed | later-process direct index read without replay |
| #169 direct FTS | confirmed | body/symbol rows in the same current index |
| #191 umbrella | corrected | current status/queue/trace own moving state |
| #194 result DTOs | corrected | one Service assembly; requested blockers only |
| #195 recognizer handoff | corrected | old census is historical; recheck callers |
| #197 CLI | corrected | strict config, one Service call, exact output/error/exit |
| #198 LSP | reconciled | same public Service owner; LS debounce outside analyzer identity |
| #199 MCP | reconciled | same public Service owners; no parallel assembly |
| #200 Project input | corrected | target/scope plan, safe roots, path authority, structured failures |
| #201 end-to-end | corrected | first observation covers authority/config/partial/error behavior |

## Material source corrections

### Root overlap is a selected-path problem

Snapshot/root IDs are insufficient. Project wraps crate-private component-wise
`open_workspace`, retains request-local `InputRootId` plus nonserialized lexical
absolute root descriptor, and derives a portable case-folded
`InputPathAuthority = root descriptor + canonical relative path`. Reject duplicate
joined authorities, not root ancestry. Equal relative paths in distinct roots and
disjoint nested roots remain valid. Do not use inode/file ID, content, hard-link
or snapshot identity. Ambient `ProjectInputDirectory::open` is not Slice 1A's
trust boundary.

### Service owns trusted assembly

The lower request accepts independent snapshots and limitations. One public
Service ordinary-check operation owns Project preparation, plan/Main/authority
validation, requested blockers, snapshot construction, snapshot/root mapping,
same-owner Emmy requirement collection and lower invocation. Frontends resolve
config/render only.

### Input failures keep structured origin

Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
message. Library parse failure becomes caller `invalid_input`, not component
outage. Project file-specific failures keep root/path. Never recover origin from
messages, absolute paths or synthetic analyzer roots. Use a small operation error
(`invalid_input | cancelled | internal`), not global error-provenance expansion.

### Partial input differs from invalid input

Missing root/explicit TOC, unsafe bytes/path, unrecoverable TOC syntax, malformed
Library or caller-driven budget refusal is pre-result invalid input. A safely
parsed TOC naming a missing child Lua, required dependency, unresolved included
condition, deferred XML or selected typing context is partial and preserves
findings. Repeated Main occurrences, Interface without target, `LoadWith` and
unused metadata remain observations.

Top-level chunk ellipsis is detected by one bounded collector on the same
configured Emmy AST: `LuaLiteralToken::Dots` without enclosing `LuaClosureExpr`,
one exact occurrence record per affected file. No raw scan/second parser.

### Strict config and output

Read one explicit 256 KiB no-follow config using duplicate-key/depth-bounded
`StrictJson`; reject unknown fields and implicit discovery. Merge CLI/config before
required-field validation. Defaults are text, `fail_on: error`, upstream default
diagnostics. Aggregate Library bounds are 64 roots, 8,192 files, 256 MiB.

Pre-encode result/error <= 32 MiB and write one envelope. Results stdout; admitted-
format pre-result errors stderr; early grammar errors text stderr. Check
cancellation after encoding/before first byte. Error message <= 4 KiB; Library-only
zero-based root index; role-relative path. Explicit severity rank controls
`fail_on`; preserved `Unknown` meets every threshold.

### Pinned diagnostic config needs strict admission and semantic identity

Pinned `EmmyrcDiagnostic` lacks unknown-field rejection, invalid `globalsRegex`
entries are logged/dropped, and `diagnosticInterval` is consumed only by EmmyLua
LS debounce. Direct checker prevalidates exact keys/regexes and rejects interval.

Current `EmmyAnalysisConfiguration::equivalent` raw-compares serialized JSON,
which falsely distinguishes omitted diagnostics from `{}` (`Some(500)` vs `None`)
and reordered/duplicated set-like fields. Replace it with a canonical analysis
projection: preserve `enable`; sort/dedup `disable`, `enables`, `globals`,
`globalsRegex`; sort severity by diagnostic code; omit interval. Equivalent
analysis semantics reuse one owner; real semantic changes rebuild cold. LS
debounce remains LSP state.

## Maintenance rule

When an owner/result boundary changes, update current status and trace; queue only
if order changes; only affected Issues. Before deletion or trusting a census,
recheck production callers. Secondary instructions link to canon rather than
copying it.
