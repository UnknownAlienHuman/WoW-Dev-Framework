# Implementation order

[Current status](CURRENT_STATUS.md) identifies what exists. This file orders the
remaining work; [IMPLEMENTATION.md](IMPLEMENTATION.md) owns the active check
contract. Change existing owners and their real callers. A source review is not
a prerequisite campaign before each implementation change.

## Working one step

Choose the first unfinished prerequisite for the assigned result, then use the
[repository map](REPOSITORY_MAP.md#ordinary-check) to read its producer and
consumer at the same revision. Keep one bounded change: expected input/output,
owner paths and explicit later-slice exclusions. If another draft owns the same
boundary, inspect its diff before implementing it again.

Before handoff, follow [the review and check matrix](../CONTRIBUTING.md#choose-the-checks)
and update only the affected status/trace/contract. A new helper is not a finished
step until its caller uses it; a merged component is not an observed workflow.
No new task registry, correction number or full-repository audit is required.

<a id="1-first-usable-ordinary-addon-check"></a>

## 1. First usable ordinary addon check

### 1A. Explicit physical-Lua `wow check`

**Result:** explicit addon root + TOC + `retail` + annotation Libraries produces
ordinary Emmy diagnostics and honest completeness through one public Service
operation. No WoW rules in this slice.

Read the [Slice 1A contract](IMPLEMENTATION.md#first-implementation-slice),
[CLI/config/output](FRONTENDS.md#configuration-and-check-input) and the relevant
[Emmy owner rules](EMMYLUA_UPSTREAM.md#configured-analyzer-session). These are the
behavioral homes; this queue does not restate their limits or data models.

Implement in this order, joining producer and consumer as each becomes usable:

| Step | Change | Existing code to inspect |
| --- | --- | --- |
| 1 | Classify TOC metadata by effect (T23); reuse/reconcile draft #266 | `crates/wow-project/src/load/toc.rs`, `metadata.rs`, `package_closure/projection.rs` |
| 2 | Profile-free target, physical-Lua selection, empty Main and safe source/error classification (T32–T36/T38) | `crates/wow-project/src/load/mod.rs`, `disk.rs`, `disk/library.rs` |
| 3 | Same-session chunk requirements, structured Library errors and canonical settings/global disable (T31/T38/T13/T16) | `crates/wow-emmy/src/configured_session.rs`, `configuration.rs`, `syntax/analyzer.rs` |
| 4 | One public Service operation owns preparation, blockers and exact snapshots; internalize the lower request/check (T37/T38) | `crates/wow-service/src/addon_check.rs`, `addon_check/project_limitations.rs`, `lib.rs` |
| 5 | Connect ordinary CLI/config/result/errors (T01/T02/T39) | `apps/wow/src/args.rs`, `main.rs`, `output.rs` |

These are implementation steps, not new registries or public APIs for every row.
Keep existing callers intact until replaced. Do not advertise the ordinary
command until its source-to-output path exists.

<a id="source-traced-replacement-boundary"></a>

### Replacement boundary

Replace `LocalCommand -> LocalProjectBackend -> generic Service::check` for the
ordinary checker. Do not satisfy that route's Reference/Rules/publication
prerequisites with fabricated identities or a compatibility mode.

<a id="exact-owner-split"></a>

### Owner split

[Project, Service, Emmy and frontend responsibilities](IMPLEMENTATION.md#code-ownership)
are defined once in the implementation guide. Path-sorted analysis snapshots do
not represent WoW runtime order.

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
