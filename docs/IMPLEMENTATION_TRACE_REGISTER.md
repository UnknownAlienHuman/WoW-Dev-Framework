# Post-merge implementation trace register

**Documentation baseline:** PR #244, merge `0fdc34c4a8afb1af51ec47d2038734d02500af42`  
**Current `main` checked:** `e40b64ffa1c16851fa657c0bd7fa0073e3cc33d9`  
**Implementation work started:** 2026-10-24

This is the active addendum to [AUDIT_REGISTRY.md](AUDIT_REGISTRY.md). The
baseline register accounts for the old 329-document corpus. This file records
new producer-to-consumer traces, implementation decisions and exact observations
after that merge. [CURRENT_STATUS.md](CURRENT_STATUS.md) is the concise live
checkpoint.

A source trace is not an executed user workflow. Documentation, compilation and
Clippy can move a row to **implemented**, but only an exact ordinary command or
protocol input/result moves it to **observed**.

## Status language

- **traced** — the named source chain was followed far enough to support the
  decision;
- **partially implemented** — some real handoff exists but its advertised caller
  or result is incomplete;
- **implemented** — code exists in `main` and passed the named exact-head gate;
- **observed** — the ordinary user workflow was exercised and recorded;
- **unexecuted** — no end-to-end observation exists.

## Active implementation traces

| ID | Chain | Code status | User observation |
| --- | --- | --- | --- |
| T01 | CLI check -> transitional local backend -> generic Service -> output | traced; replacement CLI adapter open | unexecuted |
| T02 | ordinary config -> root/flavor/TOC/Library/dependencies | traced design; open | unexecuted |
| T03 | flavor/optional Interface/load context -> TOC selection | traced design; open | unexecuted |
| T04 | bounded disk/Library bytes -> exact analyzer snapshots | lower readers exist; direct acquisition open | unexecuted |
| T05 | selected XML facts -> virtual Lua units/maps | lower preparation exists; direct pure handoff open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented in `main` by PR #247; run #422 passed | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented in `main` by PR #249; run #440 passed | unexecuted |
| T08 | unsaved editor text -> configured project diagnostics | overlay exists; project reuse open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced design; selected-load update open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced design; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped analysis context | traced design; open | unexecuted |
| T12 | upstream diagnostic -> framework DTO -> transitional LSP payload | implemented in `main` by PR #246; run #418 passed | unexecuted |
| T13 | caller diagnostic config -> optional semantic collectors | implemented in `main` by PR #247; run #422 passed | unexecuted |
| T14 | exact snapshots -> direct Service diagnostic/limitation result | implemented in `main` by PR #248; run #425 passed | unexecuted |
| T15 | compiled pin/report evidence -> workspace identity -> legacy admission | implemented in `main` by PR #249; run #440 passed | unexecuted |
| T16 | requested Main snapshot -> configured-owner reuse admission | implemented in `main` by PR #250; run #445 passed | unexecuted |

The previous register incorrectly cited runs #437 and #442 for T15/T16. GitHub
Actions associates the successful exact-head runs with #440 and #445.

## Fixed decisions for open traces

### T01 — do not extend the old generic check platform

`apps/wow/src/args.rs` still routes `wow check` through `LocalCommand`,
`LocalProjectBackend`, generic `Service::check`, global component capability
reduction and presentation output. That path requires Reference, Rules,
compatibility and Project-publication state for type-only work; empty selection
expands to every rule.

**Decision:** the public CLI must call the delivered `wow-service::addon_check`
after ordinary acquisition. Never fabricate empty platform objects or wrap the
direct result in the legacy component/capability result. Remove the old path with
its last callers.

### T02 — publication configuration is not user configuration

`wow-project::ProjectConfiguration` contains Project/profile/source/generation
identities, probe/report binding, capability policy, graph budgets and canonical
publication digests.

**Decision:** ordinary configuration contains root, optional TOC, flavor,
optional target Interface, Library directories, dependency roots, selected
checks, Emmy diagnostic settings and failure threshold. CLI values override file
values under the path-base rules in [IMPLEMENTATION.md](IMPLEMENTATION.md).

### T03 — use a small load target

Current useful TOC logic is coupled to `ProfileIdentity` and a mandatory positive
Interface. The public operation needs flavor, optional target Interface and
`TocLoadContext` without invented profile/source digests.

**Decision:** extract that target, preserve explicit TOC selection through an
Interface mismatch, require explicit choice for ambiguity, support retail/
Mainline first, and keep unsupported flavor/load contexts explicit.

### T04 — reuse safe readers, not publication inventory

`ProjectInputDirectory`, Library discovery and `ProjectInputFile` already enforce
held roots, no-follow regular-file reads, path/case/UTF-8 checks, bounds,
deterministic ordering and exact content identity. `ProjectInputInventory::build`
adds publication configuration and rejects physical-empty Main.

**Decision:** direct acquisition converts selected exact files into the snapshots
required by `check_addon`; it does not construct publication inventory. Read only
the selected TOC/dependency closure so unrelated variants cannot block the run.

### T05 — split pure XML preparation from receipts

`wow-project::xml_lua` already owns inline-unit extraction and entity/newline/
discontinuous source maps. Its public wrapper also requires configuration,
generation, capability and report identities.

**Decision:** expose pure preparation from selected load facts, captured bytes,
direct limits and compiled analyzer identity. Keep one legacy wrapper while old
publication callers remain. Preserve located malformed-unit failure and
independent usable inputs.

### T08 — keep overlay ownership, replace one-file analysis

Overlay code correctly owns exact unsaved text, client version, internal revision
and UTF-8/UTF-16 conversion. Its analyzer input is isolated from Main/Library/XML.

**Decision:** pass exact text overrides to the same configured project/check
owner. Accept increasing nonconsecutive client versions, reject stale versions,
and never publish a stale/cancelled candidate over newer text. PR #246 fixed only
the diagnostic payload, not project reuse.

### T09 — file deltas and selection/config changes are different

The older `AnalyzerSession` has exact add/update/remove plus conservative semantic
reindexing. `LiveProjectStore::update` rejects TOC/package-selected plans.

**Decision:** use a physical delta only when selected membership, load context,
Library identity and analyzer configuration are unchanged. TOC/XML/dependency/
Library/config changes rebuild selection and owner cold. Removal deletes
associated virtual units, maps, diagnostics and index rows. Publish atomically.

### T10 — one generated-Library manifest

The builder writes Lua and `source-report.json`; the ordinary Library reader reads
Lua only. The old native-artifact reader expects another envelope and ignores
sidecars/mappings/limitations. Current sidecars contain names/source links while
typed values remain in source facts.

**Decision:** keep one shared manifest DTO owned with annotation output. Project
returns optional bounded exact bytes; Service validates/consumes typed facts and
maps needed by implemented checks. Plain Lua Libraries remain valid for typing.
No converter, original checkout or old Reference snapshot is required.

### T11 — mapped helper context follows pinned Emmy semantics

Pinned Emmy retains final-position multi-results and table identity; existing
transformed-workspace/source-map machinery can map replacements to original
ellipsis spans.

**Decision:** one analysis-only helper/private table per supported ordinary addon;
replace only chunk-owned ellipsis expressions in memory; mark synthetic origins
context-only; compose maps; keep XML handler, free inline, suppressed table,
secure and bootstrap contexts separate. This remains unimplemented and
unobserved.

## Implemented slices

### T12 — diagnostic contract

PR #246 (`b56c346` on merged history) removed the four-code admission filter,
retained string/numeric/missing codes, exact bounded messages and upstream
severity, mapped exact byte spans, initialized standalone Lua 5.1 STD and passed
the real payload through the transitional LSP. Unknown families become `Other`
rather than disappearing. Run #418 passed repository policy, formatting,
workspace compilation, strict Clippy and tracked-file checks.

### T06/T13 — configured Main/Library/virtual owner

PR #247 (`b40f701` on merged history) added `EmmyAnalysisConfiguration` and
`ConfiguredAnalyzerSession`:

- explicit exact Main, Library and optional virtual snapshots;
- caller diagnostic policy, fixed Lua 5.1 and no `.emmyrc` discovery;
- one embedded STD initialization;
- diagnostics-only collection without member/call/local-flow;
- explicit semantic collection through the same owner;
- compatibility over Main, Library set, virtual identity/generation and
  diagnostic configuration.

The lower registration constructor still starts from its legacy fixed config;
the configured owner replaces the supported diagnostic policy before collection.
This is cleanup to perform with legacy-caller removal, not a reason to create
another analysis. Run #422 passed the exact-head build gate.

### T14 — direct Service result

PR #248 (`d96c2fb` on merged history) added
`wow-service::addon_check::check_addon` over exact already-acquired snapshots.
It preserves source role, category, code, message, kind, severity, path, content
identity and byte span; separates findings from limitations; and avoids the old
Reference/Rules/publication/graph/store/presentation stack.

The result is deliberately `Partial`: addon chunk arguments are not bound, and
virtual input also reports missing exact XML projection and handler context.
Root/TOC/Library acquisition and CLI wiring remain T01–T05. Run #425 passed.

### T07/T15 — compiled provenance versus report evidence

PR #249 distinguishes `compiled_pin` from `compatibility_report` identity.
Compiled identities bind exact crate/version/revision/tree without fake report
hashes. Legacy callers request paired report evidence through a typed fallible
boundary; compiled identity cannot silently satisfy that contract. Run #440
passed. No direct user workflow was observed.

### T16 — exact Main reuse admission

PR #250 changed `ConfiguredAnalyzerSession::compatible` to require the exact Main
`snapshot_id`. Library comparison remains order-insensitive; virtual identity/
generation and structural diagnostic configuration remain exact. The older
mutable session's CAS/delta path is a separate contract. Run #445 passed. No
public retained-owner caller or workflow was observed.

## Documentation reconciliation ledger

A reread of current `main`, merged PRs and canonical instructions found these
post-merge documentation defects:

| ID | Drift | Resolution |
| --- | --- | --- |
| D01 | root README still said the current adapter drops most diagnostics/messages | updated to current PR #246–#250 state |
| D02 | `apps/wow/README.md` described configured session/direct Service as future work | updated to delivered lower owners and remaining CLI handoff |
| D03 | `AGENTS.md` told agents to factor owners that already exist | changed to mandatory reuse of `ConfiguredAnalyzerSession` and `check_addon` |
| D04 | queue mixed delivered T12–T16 work with open acquisition/CLI work | split delivered prerequisites from remaining item 1 |
| D05 | Emmy guide said T06 remained open and linked an obsolete anchor | replaced with current owner contract and open adapters |
| D06 | trace register cited runs #437/#442 | corrected to #440/#445 |
| D07 | baseline audit environment note could be read as current verification | current status now states the baseline/audit distinction and supplied run evidence |
| D08 | baseline register retained 31 links to headings renamed in the active implementation/frontend guides | repaired each destination by subject; `xtask check` now validates local Markdown paths and anchors |

This reconciliation is documentation/source inspection. It does not change any
T01–T16 runtime observation.

## Executed observations still required

Before marking a workflow observed, record the exact command/protocol sequence,
fixture/input identities and result:

- public `wow check` with a valid and invalid typed call preserving upstream
  code/message/severity/addon location;
- generated Library plus optional manifest, and plain Lua Library without it;
- physical, mapped XML and XML-only programs;
- shared addon private table and one verified handler signature;
- add/edit/remove/rename plus TOC/XML/dependency/Library change without stale
  diagnostics;
- unsaved LSP edit, valid version gap and stale-result rejection;
- MCP unknown notification/metadata plus useful check result;
- later-process body/symbol search returning readable exact source context;
- truthful cancellation at the granularity actually supported.

Update this register when evidence changes a boundary or observation. Do not turn
it into another product specification, percentage dashboard or acceptance-tier
platform.