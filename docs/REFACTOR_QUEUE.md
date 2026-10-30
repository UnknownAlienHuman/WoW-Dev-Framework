# Implementation order

[Current status](CURRENT_STATUS.md) identifies what exists. This file orders the
remaining work; [IMPLEMENTATION.md](IMPLEMENTATION.md) owns the active check
contract. Change existing owners and their real callers. A source review is not
a prerequisite campaign before each implementation change.

## Working one step

Choose the first unfinished prerequisite for the assigned result, then use one
[repository-map route](REPOSITORY_MAP.md#ordinary-check) to read its producer and
consumer at the same revision. Reuse another draft's work before duplicating it.
Define input/output, owner paths and excluded later work in the existing task/PR.
Use [small implementation decisions](IMPLEMENTATION.md#small-implementation-decisions)
rather than create a new layer for every noun in the contract.

Steps below order integration, not unrelated edits: a configuration/error fix can
use its existing caller without waiting for TOC work. Split commits at coherent
handoffs, not into unused scaffolding. Before handoff use
[the check matrix](../CONTRIBUTING.md#choose-the-checks) and update affected facts.
A helper is not complete until consumed; compiled/merged is not observed. No new
task registry, correction number or full-repository audit is required.

<a id="1-first-usable-ordinary-addon-check"></a>

## 1. First usable ordinary addon check

### 1A. Explicit physical-Lua `wow check`

**Result:** explicit addon root + TOC + `retail` + annotation Libraries produces
ordinary Emmy diagnostics and honest completeness through one public Service
operation. No WoW rules in this slice.

Read the [Slice 1A contract](IMPLEMENTATION.md#first-implementation-slice),
[CLI/config/output](FRONTENDS.md#configuration-and-check-input) and the relevant
[Emmy owner rules](EMMYLUA_UPSTREAM.md#configured-analyzer-session). The
[map](REPOSITORY_MAP.md#ordinary-check) owns file paths; this table states when a
handoff is connected, not another specification or list of new public APIs.

| Step | Change | Connected when |
| --- | --- | --- |
| 1 | TOC metadata by effect (T23); reconcile draft #266 | The parser's classification reaches existing package admission/limitation consumers; no second alias/classification table is created. |
| 2 | Profile-free physical source preparation (T32–T36/T38) | Existing Project preparation supplies captured Main/Library bytes, the plan and admitted authority to Service, including empty Main and distinct missing/unsafe/read failures. A changed function argument alone is insufficient. |
| 3 | Configured Emmy behavior (T31/T38/T13/T16) | The same configured session supplies chunk requirements and diagnostics; settings/global-off, Library health and resource causes survive to the consuming checker. No second parser/session. |
| 4 | Public Service assembly (T37/T38) | Merged ordinary input reaches preparation and the internal checker, producing one result or located typed failure. The lower snapshot request is no longer a frontend API. |
| 5 | Ordinary CLI/config/output (T01/T02/T39) | Real arguments/config call that Service operation; text/JSON/errors/exits use its outcome. Replace the ordinary legacy dispatch and exclusive options/help rather than keep a permanent dual route. |

Steps 2–5 must join into the explicit-input-to-output path. Do not introduce
placeholder success, fabricated identities or unused preparation/execution
interfaces to make an isolated step appear finished. Retain still-used legacy
behavior until its callers move; advertise the ordinary command only when wired.

<a id="source-traced-replacement-boundary"></a>
<a id="exact-owner-split"></a>

### Replacement boundary

Replace `LocalCommand -> LocalProjectBackend -> generic Service::check` for the
ordinary checker. Do not satisfy its Reference/Rules/publication prerequisites
with fake identities or a compatibility mode. Ownership is defined in
[Implementation](IMPLEMENTATION.md#code-ownership), not by the current crate count.

### Completion cases

Keep these pending until the product path exists and validation is authorized by
[the owner policy](../AGENTS.md#verification-and-handoff). They are not a demand
for new tests before implementation. Record actual input/result when exercised;
merging code alone does not mark a case observed.

- Clean and invalid typed calls retain exact root-relative diagnostics.
- Top-level ellipsis limits affected files; function varargs, strings and comments
  do not. Missing child Lua, required dependencies and deferred XML remain partial
  while independent diagnostics survive. Missing child parents and leaves agree;
  permission/symlink/I/O failures are not reported as absence.
- Empty selected input is complete; deferred-only input is partial. Malformed
  Library retains structured root/path. Duplicate authority is refused; disjoint
  nested roots and equal relative paths under distinct roots work.
- Invalid diagnostic keys/regex/interval are refused. Equivalent default/set-like
  settings reuse one owner; real analysis changes rebuild it. Global diagnostics
  off suppresses collection without hiding Library health failures or blockers.
- Config outside cwd, CLI overrides and TOC use their specified path bases.
  Text/JSON, errors, resource bounds, cancellation and exit precedence follow
  [the frontend contract](FRONTENDS.md#check-results-and-command-status).

### 1B. Dependencies, XML and chunk injection

After the physical path works, add required/available optional dependency
closure and exact selected load order; mapped XML virtual Lua; ordinary
`(addonName, addonTable)` for marked chunks; one reviewed XML handler adapter;
physical, XML-only and empty scopes; coherent source/TOC/XML/Library updates.
Malformed included XML must preserve independent diagnostics.

### 1C. Special loading

Add channel/`OnlyBetaAndPTR`, reverse `LoadWith` triggers, implicit bindings and
mapped `keystate`, positive target Interface and secure/bootstrap/suppressed-table
contexts. Preserve source Interface zero and unsupported raw `Secure`; retain
lifecycle/retrieval/error metadata and machine SavedVariables declarations without
reading user data. Use [WoW semantics](WOW_LINTER.md), not guesses at client timing.

## 2. Generated Library semantics and selected WoW checks

Use the existing native builder and ordinary Library reader. Extend the existing
`source-report.json` with the selected typed metadata and consume it optionally,
keeping plain Lua valid. Follow the [output/handoff contract](KETHO_RUST_PORT.md#output-and-source-locations)
and [generation publication boundary](KETHO_RUST_PORT.md#existing-source-to-output-path);
the current report is not a raw-source bundle or a legacy cache envelope.
Correct callable Environment, return groups, nilable contents, stride, mixins and
selected restriction/Secret semantics. Then add one useful direct WoW check with
correct semantic binding. Run only selected checks. Preserve source maps and
explicit projection loss; no provider/capability/evidence platform.

## 3. Thin LSP/MCP adapters

Reuse the same public Service operation, exact unsaved text/version and mapped
positions; reject stale publication and accept standard empty results/version
gaps. Advertise only implemented MCP operations. General Lua editor intelligence
stays upstream. Protocol details live in [FRONTENDS.md](FRONTENDS.md).

<a id="direct-source-lookup-and-remaining-platform-removal"></a>

## 4. Direct source index, search and context

Use one rebuildable SQLite DB for exact selected bodies, symbols/spans and
consumed relations. Query after restart without Project/Emmy/Graph replay: body
FTS, exact symbol lookup and readable located context. Replace/update/delete
transactionally; distinguish complete empty, unavailable/partial and truncated;
use real SQLite cancellation where supported. Checking never requires this DB.

## 5. Remove displaced platform code

With each final caller, remove its obsolete publication/replay, generic results,
recognizer pack/plan/matcher/proof, graph fingerprints, nested SQLite/hex/JSON,
Store lifecycle, Search federation or Context packs plus exclusive options,
fixtures and prose. Do not defer all cleanup to one final rewrite or delete
still-consumed code merely because its name is on this list.

## 6. Measure a real workflow

After the public workflows run, measure one selected real source: file count,
elapsed time, database size and demonstrated bottleneck. Fix it and compare the
same input. No benchmark/telemetry/certification platform is required.
