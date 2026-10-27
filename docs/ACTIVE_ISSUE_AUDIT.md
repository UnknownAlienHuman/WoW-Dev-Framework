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

### Root overlap is a file-authority problem

Duplicate root and snapshot-ID checks are insufficient. Snapshot identity includes
universe/content, so one physical file may receive different Main and Library IDs;
overlapping recursive Library roots may share files while whole snapshots differ.
Validate admitted file authorities after acquisition and reject the same file
across roles/roots. Do not reject nested roots when selected file sets are
disjoint, and do not reject equal relative paths in distinct roots.

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
role, snapshot/root token and relative path as fields; Service maps the snapshot
to the configured Library root and returns invalid input. Project attaches paths
to every file-specific acquisition error. Message parsing and synthetic analyzer
roots are not admissible provenance.

### CLI success and error channels are separate

Strict explicit config rejects unknown fields; each Library root must yield an
admitted Lua inventory. Complete/partial results are written once to stdout.
Usage/input/cancel/internal failures occur before a result and go to stderr; JSON
mode uses one bounded typed error object and leaves stdout empty. Output failure
never causes a retry or a second envelope.

## Maintenance rule

When an owner/result boundary changes, update current status and trace; update the
queue only if order changes; update only affected Issue bodies. Before deleting
code or trusting an old census, recheck current production callers.
