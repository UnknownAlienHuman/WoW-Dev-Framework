# Documentation rewrite basis

Reviewed 2026-10-21 against `3e446746ece1b93a51d438a4c8f17cfd21d73beb`.
Preparation evidence, not another normative architecture or task tracker.
[Architecture](ARCHITECTURE.md) owns scope; [implementation](IMPLEMENTATION.md)
and the [queue](REFACTOR_QUEUE.md) route changes. Do not load this whole review
as mandatory context for every task.

## Overall diagnosis

The product problem is not merely excess lines or old format names. Five design
choices keep recreating the same failure:

1. Work closes around a crate/representation, while the producer, actual reader
   and user-visible result remain disconnected. The builder/verifier mismatch
   and old test driver are concrete examples.
2. Adapters preserve identities, classifications and proofs while discarding
   useful meaning. Emmy messages/severities and readable source are examples.
3. Saved data is treated as a recipe to recompute and certify, not a cheap query
   result. Project hydration and nested Search SQLite are examples.
4. Fixed product behavior is routed through a general platform: matcher packs,
   publication/retention protocols and feature-combination formats.
5. Documentation, Issues and tests describe different revisions. Editing one
   document or passing Clippy does not reconcile those descriptions.

Our refactoring repeated these mistakes: adding a public generator without
switching its test producer/readers, mechanically replacing schema assertions,
reporting deletions instead of a completed workflow, and leaving the umbrella
Issue pointing at merged #192 and completed #193 as the next work.

## Scope of this review and delivered work

Compared Gemini's [original audit](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/e6961862853506adad63fe296991c5b12a24fe13/docs/AUDIT_2026-10-17.md)
and [old architecture](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/e6961862853506adad63fe296991c5b12a24fe13/docs/ARCHITECTURE.md)
with current instructions, selected producers/readers, frontend/rule/diagnostic
paths and pinned Ketho/Emmy APIs. The inspected source tree is
`12bd0cbd253a2fccdd84a5bc2b74c06e671f8037`, also the checked #235 tree.

There are 847 Rust files / 361,367 LF lines; Service contributes 182 files /
94,350 lines. Counts include tests, examples and support code. They are not a
production-only count, measured saving or deletion quota.

Delivered: one Main/Library analysis, one statement-catalog parse, one final
annotation emission and a native generation entrypoint. Wasm, Reference Pack
administration, Core E0 envelopes, one-shot replay registries, lineage/comparison,
Project historical/catalog/replay tag ladders and substantial obsolete plans were
removed. These are useful changes, not proof that the three user workflows work.

No full Rust tests, real-addon workflow, donor corpus, Windows protocol session
or benchmark was run in this review. Rust/Cargo are unavailable in this local
review environment. Source conclusions below are not claimed executed failures.

## Current defects and missing user behavior

Paths below are relative to the repository at the reviewed commit. Existing
Issues own fixes; this table is evidence for the rewrite, not a parallel backlog.

| Problem | Concrete source / effect | Existing task and replacement |
| --- | --- | --- |
| Emmy diagnostics are reduced to four codes. | `crates/wow-emmy/src/syntax/analyzer.rs::accepted_code` retains syntax, doc-syntax, assignment and parameter mismatch only; the collection loop silently skips everything else. | #201: use configured upstream diagnostics, not a new local allowlist/type checker. Explicit justified suppressions are different from withholding every unlisted code. |
| Diagnostic meaning is lost. | The same collector does not retain `Diagnostic.message` and forces `normalized_severity = Error`; `EmmySyntaxDiagnostic`/Service `GenericFinding` have no message. LSP emits the constant `Diagnostic from the selected local syntax adapter family`. CLI text serializes findings as JSON. | #201/#198: carry bounded upstream message/code/severity/location through the existing DTOs and render a readable error. No translated diagnostic catalogue or approval registry. |
| Rule selection is applied after execution. | `wow-service/src/local/projection.rs::check_context` calls the engine before filtering requested rules. `wow-rules/src/engine.rs::run_providers` executes both API and Secret providers. | #201: pass the selected rule set before provider work; do not spend another provider's work/budget for an unrequested check. |
| Command status is based on unrelated subsystem completeness. | `local/projection.rs::components` considers Secret capability regardless of selected rules and always adds a Degraded/Partial annotation component for native receipts. `service.rs::derive_status` puts any partial component before Findings. | #201: retain actual findings and report completeness of the requested analysis separately. Do not label missing required knowledge clean; optional/unselected features must not decide another check's outcome. |
| Editor analysis is not the configured addon analysis. | `wow-service/src/frontend/lsp/diagnostics.rs` calls `analyze_syntax_with_stop` and enforces exactly one file. The path does not consume the configured Main/Library analysis delivered in #193. | #198/#200: decide and document the WoW frontend boundary; reuse the configured owner or upstream server. Do not copy another general LSP engine. |
| Public generator and verification workflow disagree. | Builder `src/lib.rs` emits `wow-native-source-build/2`; `tools/xtask/src/library.rs::verify` accepts only `/1` plus an embedded `library`. `current-source-bundle.yml` connects them. | #201: align the useful reader with current output or remove the obsolete verifier. A tag-only substitution cannot fix the changed shape. |
| Tests use a different generator. | `wow-annotations/tests/consumer_support/source.rs` includes `examples/support/mod.rs`, not the supported application. Old Git-I/O/catalog helpers duplicate the builder's copies. | #201: application checks invoke the supported producer; retain type/map cases at their owner and remove the superseded driver. No third driver or reverse application dependency. |
| Producer/consumer limits disagree. | Native output allows 4096 files / 8 MiB per file / 64 MiB; ordinary Library input allows 1024 / 1 MiB / 16 MiB. | #200/#201: agree finite limits at the real handoff. No real-corpus overflow was demonstrated; do not claim compatibility merely because each side is bounded. |
| Normal setup still demands internal evidence. | `local/{disk_input,input,native_input}.rs` consumes explicit host/configuration/Reference/analyzer-report inputs; `apps/wow/src/args.rs` requires config/project. Engine constructors do not call `init_std_lib`. | #200/#197: ordinary root/flavor/TOC/Library settings and bounded discovery; load appropriate upstream standard declarations once. Do not manufacture Reference evidence or add installers. |
| Persistence repeats analysis. | `wow-project/src/replay/mod.rs::hydrate_owner` reconstructs/publishes then compares a full recapture; `replay/publication.rs` validates reconstructed graph contents. | #131: query stored facts/text without Emmy/graph reconstruction. Removing only the final equality is not the fix. |
| Search is not yet repository body search. | `wow-store/src/search/mod.rs` stores an inner SQLite image as `database_hex`; existing file FTS covers labels, with separate source-name observations. | #131/#169/#106: ordinary file/text/symbol-span tables and FTS, queried after reopen. Specify token/phrase versus substring semantics; do not add another engine. |
| Search-to-readable-source handoff is missing. | Current search/context route excludes source excerpts; `wow-context/src/pack/source_excerpt.rs` already has a bounded reader. `render.rs::emit_record` escapes punctuation into `\uXXXX`. | #122: connect the hit to matching indexed text and readable path/range/fenced code. Preserve source as data and never use old offsets on new text. |
| Updates do not cover the main addon load path. | Physical deltas reuse syntax trees but reindex remaining files; Service live updates reject TOC/package load plans. `reused_files` is not semantic reuse. | #109: connect existing reload/rebuild and preserve changed/deleted-source behavior. No invented incremental framework or speedup. |
| Result bureaucracy survives Core envelope removal. | Four Search paginators contain `for _ in 0..8`; canonical admission builds a JSON tree and rejects negative integers/floats/None; Context still has duplicate wire representations. | #194/#169: remove self-size/self-identity dependencies at actual readers; serialize ordinary DTOs once with external output bounds. Preserve admission for validated owners. |
| Fixed relationships pass through a generic matcher. | `wow-recognizers/src/source_signals.rs::w1_pack` rebuilds a fixed pack after Rust already selected facts; State Core repeats pack/projection handoff in Service. | #195/#103: map consumed typed facts directly and remove obsolete adapters/packs, not their useful relations. |
| Graph navigation can regenerate an existing Library. | `wow-service/src/local/native_library_input.rs::rebuild_for_project` reprojects Reference/annotations/sidecars to compare identities; `graph/build/native_api.rs` consumes it. | #107: reuse loaded Library and callable/source origins; remove requalification from that read path. Ordinary Library use need not acquire this obligation. |
| Feature-format/lifecycle proliferation remains elsewhere. | Annotation `native.rs` still chooses library /3,/4,/5; aliases and frontend hosts have other families. Store retains lifecycle administration outside narrowed scope. | #112/#111 with the relevant consumer: one current representation; rebuild derived output, preserve user files. Project cleanup did not finish these domains. |
| Protocol machinery rejects valid ordinary usage. | `apps/wow/src/mcp.rs` closes on unhandled notifications/rejects nonempty `_meta`; LSP uses private no-match errors and disclosure gates. | #198/#199: standard empty results/extensions and useful existing operations. External protocol evolution is not internal cache compatibility. |

The first four rows are newly traced user-output defects/limitations. The prior
owner note incorrectly elevated the four-code slice into a permanent product
boundary; it is corrected in this change. One shared engine is not enough when
its results are then discarded or misrepresented.

## Generator acquisition details not to lose

`apps/wow-reference-builder/src/io.rs` uses `cat-file blob` for TOC/API inputs
without checking tree mode. The alias reader in `catalogs.rs` already validates
ordinary-file mode and exact path. Reuse it. A prior local Git demonstration
returned a mode-120000 symlink's target text as a blob; this does not demonstrate
symlink following, outside-root access or Lua execution by the builder.

The Git helper has no stop parameter; blocking `read_to_end`/`wait` cannot observe
cancellation. Checkpoints before/after it do not make the child interruptible.
Thread cancellation through the existing helper and reap the child; no job system.
Keep new-only output/report-last behavior. Failure codes are 2 for arguments,
5 for generation failure, 6 for observed cancellation and 8 for signal setup.

The workflow selects six Ketho catalog files, not every resource the donor or
parser supports. Keep catalog implementation coverage, selected bundle coverage
and actual consumer coverage distinct. No full donor parity was demonstrated.

## Our regressions and audit corrections

- #232's `wow-service/src/live_project/tests.rs:465–485` mutates `obsolete_schema`,
  then `serde_json::from_value(changed)?` exits before the expected rejection
  assertion. Repair the existing boundary check, keeping source-substitution cases.
- `local/platform_input/tests/empty_packages.rs:101–109` and
  `platform_input/tests.rs:441–447` still expect semantic selection failures during
  decode. Trace `ProjectConfiguration::validate` in hydration before changing the
  expectation. No invalid-hydration bypass is established. Remove the duplicate
  obsolete-schema case in `wow-project/tests/primary_callable_replay.rs:396–402`.
- Our #225 producer move left the previous test producer and incompatible consumer.
  Our #232 tag edits left contradictory prose. #191 still named merged #192 and
  completed #193 as active work until this review. Fix actual handoffs and entry
  instructions; do not recreate delivered functionality from stale status text.
- Repeated helper-workflow/CI repairs and line-deletion reports displaced delivery.
  Use one inspected worktree and normal Git writes. After two failed approaches,
  change the failed assumption. No new evidence platform or broad test campaign.
- Gemini's cited XML vector is charged **after** the loop; that does not prove
  quadratic work. `allow(dead_code)` does not prove no consumers; `generated_api.rs`
  has a live role. No measured 70–80% runtime saving exists.
- `CheckRequest::with_rules` rejects unknown rule IDs. The late-selection finding
  above concerns known requested rules, not an unknown-rule bypass.
- Keep plain DTO decoding distinct from admission of invariant-bearing owners.
  Removing wire duplication must not manufacture validated executable state.

## Material for the new documentation

Keep only the manifesto, workflow implementation routes, queue and the necessary
owner details. A workflow recipe states ordinary inputs/defaults, actual producer
and reader, observable output/error, and displaced code to remove. It distinguishes
implemented code from exercised behavior. No parallel state/acceptance framework.

| Old notes | Useful facts to transfer once | Requirements to retire |
| --- | --- | --- |
| `PLATFORM_SOURCE_CONFIGURATION.md`, `PLATFORM_STORE_NAMESPACE.md` | Source/flavor, Main/Library, isolation and actual reopen limits. | Frozen epochs, migrations, old-byte preservation and claims that the CLI is absent. |
| `PLATFORM_DIRECT_PRODUCERS.md`, `PLATFORM_DIRECT_SERVICE.md`, `PLATFORM_RECOGNIZER_ASSERTIONS.md` | Consumed TOC/XML/call/state relations, locations and ambiguity. | Staged proof choreography, version histories and fingerprint acceptance. |
| `PLATFORM_RAW_INVENTORY_GRAPH.md`, `PLATFORM_INVENTORY_SPANS.md` | Indexed versus loaded files; whole-file versus byte-range locations. | Inventory certificates and legacy replay acceptance. |
| `PLATFORM_XML_SOURCE_MAPS.md` | Entities/newlines, CDATA/comment gaps, virtual Lua and ambiguous carets. | Replay-version narratives and historical gate counts. |
| `wow-recognizers/e2/PARTITIONS_AND_VERSIONING.md` | Replace stale derived relations on source/selection change. | Frozen report9, shadow/holdout/promotion and feature matrices. |

The nine notes and seven broken Store links recorded in #112 still need semantic
consolidation. Do not restore deleted E2 specifications to satisfy those links.

Reuse [Ketho's pinned emitter](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/luasrc/annotate/init.lua)
for defaults including false/zero, arrays, receivers, parameter/return order and
callbacks; preserve maps and explicit unsupported types instead of inventing any.
Reuse [the pinned Emmy API](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/lib.rs)
for analysis/library setup. STD is not WoW runtime authority; changed-only updates
do not establish dependent-file invalidation. These comparisons are not corpus parity.

## Delivery order, not another cleanup phase

First #200/#201: public generation -> loaded Library -> addon check, with readable
upstream diagnostics and honest requested-scope status. Reuse the existing small
consumer shape; check a valid and invalid call at the addon location and one
missing/partial input. Do not replace this with an emitter-only test.

Parallel #131/#169/#122: persisted source -> later-process body/symbol query ->
matching located excerpt. A body-only term must be found; changed/deleted source
must not leave stale hits. Keep actual FTS semantics explicit.

Then #198/#199 expose these same useful results to editors/agents. #194/#107/#112
reductions attach to those consumers; they are not an unlimited prerequisite phase.
General Lua features belong to upstream EmmyLua. Preserve source/configuration,
root bounds, cancellation and uncertainty without a new proof/lifecycle platform.

This review is not a complete audit of every source file, a release qualification,
or a claim that future documentation will be error-free.
