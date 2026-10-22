# Audit coverage registry

[Audit](AUDIT.md) owns conclusions; [Queue](REFACTOR_QUEUE.md) owns tasks.
This register records inspected code, evidence, remaining gaps and corrections.
Do not load the whole register into every implementation task.

**Latest inspected baseline:** `0c46e48bc7bf0018e95940dd946d9b43fb162955`,
2026-10-22. Untouched rows retain their earlier evidence; this date does not
revalidate every function. Historical detail lives in Git, not repeated diaries.

## Evidence levels

**S** = source chain traced; **P** = partial inspection; **U** = unverified;
**R** = the named public/runtime behavior exercised; **N/A** = not applicable.
Source and runtime are separate columns. Clippy, an inventory scan, a unit test
or a fake producer cannot promote a whole workflow to R.

Evidence keys:

- **A**: [source/architecture review](AUDIT.md), baseline `3c3d5bc5`, delivered
  in #237. Structural inventory covered all packages, not every function's logic.
- **B**: [Store and recognizer review](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/0c46e48bc7bf0018e95940dd946d9b43fb162955/docs/AUDIT_REGISTRY.md#completed-incremental-reviews),
  baseline `3c5356f5`, delivered in #238.
- **C**: [Core/Service ownership review](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/0c46e48bc7bf0018e95940dd946d9b43fb162955/docs/AUDIT_REGISTRY.md),
  baseline `4f429d5e`, delivered in #239.
- **D**: [frontend operation inventory](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/0c46e48bc7bf0018e95940dd946d9b43fb162955/docs/AUDIT_REGISTRY.md#2026-10-22--frontend-operation-inventory),
  baseline `12056d72`, delivered in #240.
- **E**: current generator-workflow and cancellation recheck at `0c46e48b`, below.

## Coverage and outstanding work

Paths are repository-relative. Historical evidence is not a claim of new runtime
execution. Issues contain implementation tasks, not extra audit gates.

| Area | Source | Runtime | Evidence / inspected boundary | Still unchecked or unfinished |
| --- | --- | --- | --- | --- |
| Workspace ownership | S | N/A | A: all 16 manifests, dependencies, public entrypoints and large clusters; Audit package table. | Not a semantic review of every function. Recheck changed owners. |
| Canonical documentation | S | N/A | A: root AGENTS/README, Architecture, Implementation, Queue, Completion, umbrella Issues. | Old owner notes and Issue bodies still need reconciliation (#112). |
| Native acquisition | S | U | A/E: builder `src/{lib,io,catalogs}.rs`, native Reference and annotation projection. | TOC/API regular-file mode, bounded batch reading, blocking Git-child cancellation (#201). |
| Ketho semantics | P | U | A: pinned default/false/zero, arrays/stride, callbacks, receiver/order and mapping rules. | Selected-catalog/flavor parity and real source corpus not exercised. |
| Builder -> workflow artifact | S | U | E: `/2` producer, `/1` verifier calls and report-only upload. Calls removed; Lua directories now packaged. | Hosted workflow/artifact download not exercised. Shell-only checks below are not R. |
| Builder -> addon Library | S | U | A/E: old `consumer_support/source.rs` driver and reader ceilings; emitter and actual reader remain disconnected. | Supported producer -> real reader -> addon diagnostic; reconcile limits; remove old driver/verifier (#201/#200). |
| Normal addon configuration | S | U | A: Service disk/native/host inputs and CLI selectors. | Root/flavor/TOC/Library configuration and discovery (#200/#197). |
| Main+Library analysis | S | P | A: shared Project analyzer delivered in #193; cold callers traced. | Actual CLI valid/invalid typed calls; all update/load-selection paths. |
| Diagnostic transport | S | U | A/E: `syntax/analyzer.rs`, Service projection, LSP diagnostics. Four-code filter/message loss/severity coercion remain. | Preserve upstream meaning through public check/LSP (#201/#198). |
| Emmy cancellation | S | U | E corrects A: both session paths pass stop; collector checks before/after files and during conversion; typed cancellation survives. #135 is closed. | No fresh runtime witness. Per-file preemption, registration and Git-child cancellation are separate boundaries. |
| Rule selection/status | S | U | A: `local/projection.rs`, `wow-rules/src/engine.rs`, `service.rs::derive_status`. | Select before work; distinguish findings from requested-scope completeness (#201/#103). |
| Recognizer families | P | U | B: all public built-ins -> Service Graph build; fixed pack/matcher and reopen paths. | Which relations survive direct check/search, actual semantics of every rule (#195/#107/#103). |
| Core/Service DTOs | S | U | C: exports/imports, local result model, canonical round-trips, Search envelopes. | Remove only after real consumers switch (#194/#195/#107). |
| Project stored reads | S | U | A/B: `replay/{mod,publication}.rs`, hydrate/recapture and companion graph. | Later-process file/text reads without rebuilding analysis (#131). |
| Store lifecycle | S | U | B: orphan backup/replacement/quarantine/source-authority vs active retention/GC/journal callers. | Direct-reader migration before deleting active holds/journals (#111/#131/#169/#122). |
| Search | S | U | A/C: nested SQLite/hex/JSON, name/label FTS, four self-sizing page families. | Body/symbol search after restart, direct SQL, updates (#169/#106). |
| Context | S | U | A/C: existing excerpt reader, disconnected hit path, Wire/pack models and Unicode-escaped source. | Matching text -> readable located snippet (#122). |
| Source updates | P | U | A: physical deltas and Service TOC/package refusal. | Add/change/delete/rename and load-selection effects on diagnostics and hits (#109). |
| Public CLI | S | U | D: root dispatcher, every branch enum/help; at least 68 semantic routes. | Three ordinary workflows; remove obsolete operator branches as consumers move (#197). |
| LSP | S | U | D: two profiles, 12 methods/notifications, isolated-file diagnostics and session registry. | Configured analysis, standard empty responses, framed runtime/client checks (#198). |
| MCP | S | U | D: two profiles, two tools plus resources; metadata/notification rejection and registry. | Useful check/search/context, standard extension/notification behavior (#199). |
| Root/file security | P | U | A/E: configured roots, non-execution, blob-mode and blocking-child gaps. | Final filesystem/acquisition paths; not a complete security audit. |
| Performance | U | U | A/C: amplification patterns and counters only. | Actual workflow, selected corpus/revision, reached-stage measurements (#113). |
| Full Rust suite | U | U | Not run in this audit. | Proportional existing checks as implementation reaches the changed consumer. |
| Real addon workflow | U | U | Not exercised. | #125 Phase 1 public entrypoints. |
| Real Blizzard persisted search | U | U | Not exercised. | #125 Phase 2 build/close/reopen/query/excerpt. |
| Full donor corpus / all flavors | U | U | Not exercised. | Do not claim parity from selected emitter examples. |
| Windows/editor runtime | U | U | Not exercised in this audit. | Only required for corresponding final platform claims. |

## Prior decisions retained from B/C/D

These remain source-review results at their recorded baselines, not deletions
already implemented or fresh runtime conclusions. Detailed paths remain in the
linked immutable reviews.

| Review | Consequential result | Constraint on implementation |
| --- | --- | --- |
| B / Store | Backup, replacement, quarantine and source-authority external callers are test-only. Recovery has an internal CLI route. Retention/GC/release and generic Store journals still have Search/Context callers. | Remove orphan API separately; switch direct readers before active lifecycle removal. |
| B / Recognizers | Public built-ins feed only Service Graph build. Calls are typed; construction/mixins/W1-W6/State Core/TOC/XML fabricate fixed packs; Service repeats State Core handoff. | Retain useful source relations, not a general plugin/matcher or proof platform. |
| B / Rules | Two built-in product rules sit behind a general engine; selection and completeness happen too late. | Direct selected checks; preserve uncertainty and locations. |
| C / Core | Canonical JSON is imported by 136 production files; identity/evidence models still have live consumers. Callable lowering has a real role. | No bulk deletion based on public exports, schema counts or `allow(dead_code)`. |
| C / Service | Root exports 46 names; eight have external production imports. Local results expose platform state; Search still retains a separate envelope/journal. | Shrink through actual callers; preserve useful diagnostics and cancellation, not DTO compatibility. |
| C / Round-trips | 32 canonical-bytes -> JSON conversions in 21 production files; four eight-pass Search paginators. | Typed handoffs and external output bounds; no replacement generic serialization platform. |
| D / CLI | Status/check 2; native/source/overlay 5; graph 9; project 6; search 31; context 12; transport capability dump 1. | Reduce to generation, configured check and direct index/search/context, not aliases for every old route. |
| D / Protocols | Two LSP and two MCP profiles wrap private operation registries; MCP exposes status/context-inspect rather than the target workflows. | Thin standard adapters to surviving operations; no second Lua engine or profile matrix. |

## E — generator handoff and cancellation correction

Inspected the complete public builder `src/{lib,io,catalogs}.rs`,
`tools/xtask/src/library.rs`, `.github/workflows/current-source-bundle.yml`,
`wow-annotations/tests/consumer_support/{source,source_report}.rs`, and Emmy
`src/session.rs` plus `src/syntax/analyzer.rs`.

**Workflow repair:** removed two calls feeding `/2` output to the obsolete `/1`
verifier. Both steps still warn on exit 3 and reject `input_failures != []`.
A success-only artifact now includes both generated Lua directories and reports;
the separate diagnostics artifact remains `always()`. The old verifier/test
producer, actual Library reader, incompatible limits and Git I/O gaps are not
fixed by this workflow change.

**Audit correction:** #135 was closed on 2026-10-14. Current
`session.rs::{analyze_member_call_session_with_local_flow,collect}` pass stop;
`collect_registered` checks it before/after each file and within conversion;
`map_syntax_error` preserves `Cancelled`. The prior audit repeated an obsolete
issue body as a present defect. A per-call token snapshots stop; this is not
in-flight preemption of synchronous upstream work. Git child cancellation is a
different, still-open gap. No native cancellation test was rerun here.

**Executed, narrow evidence:** workflow YAML parsed; every embedded script passed
`bash -n`; both changed generation blocks ran with stub Git/Cargo and real
shell/jq for exit 0, exit 3, input failure, generation failure, absent report,
missing field and null field. All 14 control-flow cases behaved as intended.
This is not builder or hosted-workflow execution. Relative file links and diff
whitespace were checked. Build-gate outcome belongs to the implementing PR.

**Not executed:** public builder, real Library loading, real donor corpus, hosted
artifact download, full Rust suite, Windows/editor sessions, benchmarks. No
coverage row was promoted to R in this pass.

## Next review / implementation boundary

1. #201/#200: supported producer -> actual Library reader -> addon diagnostic;
   move useful old-producer checks and reconcile finite limits before deleting it.
2. #112: extract remaining coordinate/TOC/XML facts, delete obsolete owner notes
   and repair the seven known Store links without restoring deleted E2 contracts.
3. Add exact command/input/result only when a real public workflow runs. Recheck
   issue state and current code before repeating a historical finding.

Update this register in the same PR as each pass. Preserve the inspected commit,
what changed and what remains unknown. Do not duplicate the queue or append an
unbounded progress diary.
