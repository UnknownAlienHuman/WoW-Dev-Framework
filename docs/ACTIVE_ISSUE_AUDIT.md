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
| #103 recognizer rules | confirmed | keep only facts with named consumer |
| #106 Blizzard UI workflow | confirmed | later direct source search/context |
| #107 graph duplication | corrected | loaded Library/direct origins; #196 historical |
| #109 coherent updates | corrected | one target/scope/authority/config/session owner |
| #111 Store lifecycle | corrected | census snapshot; recheck before deletion |
| #112 formats/documentation | corrected | small target/authority/result/error/config forms |
| #113 performance | confirmed | measure only an existing workflow |
| #117 TOC/XML relationships | reconciled | Slice 1B+; XML does not block physical diagnostics |
| #118 lineage/fingerprints | confirmed | remove with final callers |
| #122 readable context | confirmed | exact indexed located result |
| #125 product completion | reconciled | final observed workflows only |
| #131 direct reads | confirmed | later-process direct index read |
| #169 direct FTS | confirmed | body/symbol rows in same index |
| #191 umbrella | corrected | status/queue/trace own state |
| #194 result DTOs | corrected | one Service assembly; blockers only |
| #195 recognizer handoff | corrected | old census historical; recheck callers |
| #197 CLI | corrected | strict config/path bases/result/error/exit |
| #198 LSP | reconciled | same Service owner; debounce outside analysis identity |
| #199 MCP | reconciled | same Service owners; no parallel assembly |
| #200 Project input | corrected | target/scope, safe roots, authority, failures |
| #201 end-to-end | corrected | exact authority/config/partial/error observations |

## Material source corrections

### Selected-path authority

Project wraps crate-private component-wise `open_workspace`, keeps request-local
`InputRootId` plus nonserialized lexical absolute root descriptor, and derives
portable case-folded `InputPathAuthority = root descriptor + relative path`.
Reject duplicate joined authorities, not ancestry. Equal relative paths in
distinct roots and disjoint nested roots remain valid. Do not use inode/file ID,
content, hard-link or snapshot identity. Ambient `ProjectInputDirectory::open` is
not Slice 1A trust boundary.

### Service owns assembly

One public Service ordinary operation owns Project preparation, plan/Main/
authority validation, requested blockers, snapshots/root mapping, same-owner Emmy
requirements and lower invocation. Frontends resolve config/render only.

### Structured errors and partial input

Carry code, role, snapshot/`InputRootId`, role-relative path and bounded message.
Library parse failure is caller invalid input, not component outage. Missing root/
explicit TOC, unsafe bytes/path, unrecoverable TOC syntax, malformed Library or
caller budget refusal is pre-result invalid input. A safely parsed TOC naming a
missing child Lua, required dependency, unresolved included condition, deferred
XML or selected typing context is partial and preserves findings.

Top-level chunk ellipsis is detected by one bounded collector on the same
configured Emmy AST (`LuaLiteralToken::Dots` without enclosing `LuaClosureExpr`),
not raw scan/second parser.

### Strict config, path bases and publication

Read one explicit 256 KiB no-follow config using duplicate-key/depth-bounded
`StrictJson`; reject unknown fields/discovery. Relative `--config`, CLI root and
CLI Libraries resolve from cwd; config root/Libraries from config parent; TOC
always resolves after merge against final addon root. CLI scalar overrides retain
CLI origin; CLI Library array replaces config array. Normalize lexically only,
without filesystem canonicalization/root-link traversal.

Defaults are text, `fail_on: error`, upstream default diagnostics. Aggregate
Library bounds: 64 roots, 8,192 files, 256 MiB. Pre-encode <=32 MiB and write one
envelope; result stdout, pre-result error stderr; final cancellation checkpoint
before first byte. Message <=4 KiB; Library-only zero-based root index; role-
relative path. Explicit severity rank; `Unknown` meets every threshold.

### Diagnostic config semantics

Pinned `EmmyrcDiagnostic` accepts unknown fields, drops invalid regexes and has
LS-only `diagnosticInterval`. Direct checker prevalidates exact keys/regexes and
rejects interval. Current raw JSON equivalence falsely splits omitted config from
`{}` and reordered/duplicated set-like fields. Canonical analysis projection
preserves `enable`, sort/dedups lists/regexes, sorts severity by code and omits
interval. Equivalent semantics reuse; real changes rebuild. LS debounce stays LSP
state.

## Maintenance rule

When an owner/result boundary changes, update status/trace; queue only if order
changes; affected Issues only. Recheck production callers before deletion.
Secondary instructions link to canon rather than copying it.
