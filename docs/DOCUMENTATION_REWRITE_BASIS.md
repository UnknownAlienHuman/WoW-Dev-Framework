# Documentation rewrite basis

Reviewed 2026-10-21 against `687049b66f64304422649e857ec3a7a38b2c8a4c`.
This is preparation evidence, not another normative architecture or work queue.
The [manifesto](ARCHITECTURE.md) still owns scope; [Issues and queue](REFACTOR_QUEUE.md)
own implementation. Do not require every developer to load this document.

## What was actually reviewed

The [original Gemini audit](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/e6961862853506adad63fe296991c5b12a24fe13/docs/AUDIT_2026-10-17.md),
its [architecture](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/e6961862853506adad63fe296991c5b12a24fe13/docs/ARCHITECTURE.md),
current instructions, selected producer/reader paths, replay changes #231–#233,
and the secondary platform notes listed below were compared. The local source
copy matches tree `63e5751b40ee9d66400da68704867893c9b599f9` from #233's checked archive.

Current tracked source has 847 Rust files / 361,367 LF lines, including tests,
examples and support code. Service accounts for 94,350 of those lines. This is
not a production-only count, deletion target or performance measurement.
No full Rust test suite, real-addon workflow, donor corpus or benchmark was run
in this review. Findings below distinguish source-level proof from runtime evidence.

## Mistakes introduced or repeated by our own refactoring

### 1. A replay regression survived a green compile/lint gate

In [the TOC/XML Service test, lines 465–485](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/687049b66f64304422649e857ec3a7a38b2c8a4c/crates/wow-service/src/live_project/tests.rs#L465-L485),
#232 replaced a historical schema mutation with `obsolete_schema`, but left
`serde_json::from_value(changed)?` before the rejection assertion.
`ProjectReplay` has `deny_unknown_fields`: the last mutation returns an error
from the test instead of reaching `hydrate(...).is_err()`. This is a confirmed
source-level test defect, not an executed test failure or a production exploit.

Repair the existing check at the actual decode/admission boundary. Do not restore
a deleted schema, suppress the error, or delete the useful source-substitution
case just to regain green checks. Owner: #112.

### 2. Removing tag dispatch also moved validation between phases

[The empty-package test, lines 101–109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/687049b66f64304422649e857ec3a7a38b2c8a4c/crates/wow-service/src/local/platform_input/tests/empty_packages.rs#L101-L109)
still requires missing retention limits to fail in `ProjectReplay::from_json`.
The new decoder checks shape/size and selected fact availability; it no longer
contains the old retained-schema selector. `ReplayPlatform::validate_budget`
checks files, not this selection. The semantic requirement remains in
[ProjectConfiguration::validate, lines 767–839](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/687049b66f64304422649e857ec3a7a38b2c8a4c/crates/wow-project/src/configuration.rs#L767-L839),
reached during hydration. The missing native-library-input assertion in
`wow-service/src/local/platform_input/tests.rs` needs the same phase review.

Do not equate a decoded DTO with an admitted project. State which existing owner
rejects a missing selection; align that owner's consumers and checks. This review
does **not** demonstrate that invalid configuration passes hydration. Owner: #112.

### 3. Literal replacement was mistaken for documentation reconciliation

#232 removed exact replay tags but left prose requiring frozen V5/V8 catalogs,
report9 compatibility and historical acceptance. The reviewed audit itself still
said two Emmy engines and an unfinished pack builder existed. Ketho/Emmy guides
also described delivered work as future work. #233 corrected three documents,
not the whole corpus. A token search cannot establish semantic consistency.

Rewrite each active description from its current caller. Remove the old behavior,
not just its identifier. Historical check counts belong in their commit, not in
current acceptance statements. The secondary-note inventory below remains work.

### 4. Process ceremony displaced delivery

Our recent changes spent repeated CI runs repairing text-replacement scripts;
PR #232's surviving product change came after several failed helper runs.
Mechanical edits also left duplicate obsolete-schema cases in
`wow-project/tests/primary_callable_replay.rs:396–402`. This is the same pattern
we were removing: more machinery and green metadata, without checking meaning.

Use one inspected worktree and Git tree/blob writes, not a new apply-workflow for
each replacement. After two failed approaches, inspect the failed assumption and
change method. Keep the existing compile/lint gate; do not turn its artifacts,
line counts or passed compilation into user-workflow acceptance. Do not add a
new evidence service, audit registry or blanket test campaign to fix this process.

## Remaining code findings that new instructions must not conceal

Crate paths in this table are relative to `crates/`.

| Finding and evidence at the reviewed commit | Consequence / next replacement |
| --- | --- |
| `wow-project/src/replay/mod.rs::hydrate_owner` still publishes a reconstructed owner and compares a complete recapture; `replay/publication.rs::read_owner` also validates reconstructed graph contents. | #131 must give ordinary readers stored facts/text without Emmy or graph construction. Removing the final equality alone does not implement persistence. |
| `wow-store/src/search/mod.rs:444,605,742` stores an inner SQLite image as `database_hex`; ProjectFile FTS covers labels, not source bodies. | #131/#169: files/text/symbol spans and FTS in the ordinary database. A body-only term must work after reopening. Merely moving metadata rows is insufficient. |
| Four Search pagination files still contain `for _ in 0..8`, including `entities/query/pagination.rs:170–233`. | #194/#169: remove self-sizing page/cursor payloads and retained-result ceremony. Bound the actual output outside it; do not invent a generic framework preserving the same cycle. |
| `wow-core/src/canonical/admission.rs:88–130,299–309,407–421` still builds a JSON value tree and rejects negative integers, floats and `None`. | Trace remaining real callers first. Removing E0 envelopes did not remove this constraint. Prefer ordinary DTO serialization where exact canonical identity is unnecessary, not a new universal canonicalizer. |
| `wow-context/src/render.rs::emit_record`, lines 836–898, escapes source punctuation and quoted JSON as `\\uXXXX`. | #122: human-readable paths, ranges and fenced source; neutralize dangerous presentation without destroying code readability. Preserve source as data, not instructions. |
| `wow-annotations/src/native.rs:929–936` still selects library `/3`, `/4`, `/5`; `aliases.rs:529–545` selects another feature-dependent tag family. | #112 with the touched annotation route: one current representation. Project replay cleanup is not evidence that all format proliferation is gone. |
| `wow-recognizers/src/source_signals.rs::w1_pack` still constructs a fixed matcher pack for facts already selected in Rust; Service retains substantial graph composition. | #195: replace one consumed relation path with direct typed facts and remove its obsolete adapter/pack. Moving files or removing lint attributes is not simplification. |
| `wow-emmy/src/references.rs:745–780` configures Lua 5.1 but never calls `init_std_lib`; `AnalyzerSession` now correctly owns one shared analysis. | #200: ordinary addon setup must load upstream standard declarations once. Do not recreate the solved split-engine task or promise standard typing merely because engines were joined. |
| `apps/wow/src/mcp.rs:232–234,720–724` closes on an unhandled notification and rejects nonempty `_meta`. | #199: retain valid session traffic and bounded extension metadata. External protocol extensibility is not internal cache compatibility. Do not blindly apply `deny_unknown_fields` everywhere. |

These are inspected paths, not measured latency percentages. Existing single-engine,
single-parser and single-emission work remains useful; it must not be reimplemented.

## New generator gaps found while reviewing our delivery

TOC/API source acquisition calls [the Git helper](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/687049b66f64304422649e857ec3a7a38b2c8a4c/apps/wow-reference-builder/src/io.rs#L30-L86)
with `cat-file blob` but never verifies the selected tree entry's file mode. A
symlink is also a blob. A local Git plumbing reproduction with mode `120000`
returned its link-target text through that read. The builder binary was not run.
This contradicts the guide's regular-file guarantee; it does not prove symlink
following, outside-root reads or Lua execution. The alias-catalog path already
[checks mode and exact path](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/687049b66f64304422649e857ec3a7a38b2c8a4c/apps/wow-reference-builder/src/catalogs.rs#L79-L95).
Reuse that check for TOC/API input, not another source-admission platform.

The same helper has no cancellation argument: `read_to_end` and `child.wait`
cannot observe `STOP`. Checkpoints around calls are not cancellation during a
blocked Git subprocess. Thread cancellation through the existing helper and reap
the child; do not add a daemon/job lifecycle. Latency was not benchmarked.

`run_with_stop` returns 2 for arguments, 5 for generation failures, 6 for observed
cancellation; `main` uses 8 for signal-handler setup failure. The old Ketho guide's
blanket failure code 2 was wrong. New-only output and report-last writing already
exist; do not replace them with pack publication. These gaps belong to #201.

## Generator handoff follow-up at `1a87ce15` (2026-10-21)

The new entrypoint exists, but tracing its real consumers exposes a second
instance of our own "new path added, old path retained" mistake. These findings
come from source review; no builder, Rust test or real-source workflow ran here.

| Boundary | Confirmed mismatch | Required replacement |
| --- | --- | --- |
| `apps/wow-reference-builder/src/lib.rs` → `tools/xtask/src/library.rs::verify` | Producer writes `wow-native-source-build/2`; verifier admits only `/1` and reads the old embedded `library` object. The current-source workflow invokes them consecutively. | #201: align useful file/source checks with the real output or retire the obsolete verifier. A version-string edit alone is insufficient; do not restore embedded generated text or maintain both formats. |
| `wow-annotations/tests/consumer_support/{source,catalog_source}.rs` → `examples/support` | Probes call a separate old driver; `source_report.rs` expects `/1`. Git-I/O and catalog helpers are identical copies of builder helpers. | Move public-entrypoint checks to builder integration; retain emitter/type cases below it, then remove the old driver and duplicate helpers. No lower-owner dependency on the application or new generic driver framework. |
| `wow-annotations/src/native.rs` → `wow-project/src/disk.rs::read_lua_inventory` and `wow-service/src/local/input.rs` | Producer ceilings are 4096 files, 8 MiB/file and 64 MiB/library; ordinary Library acquisition/assembly permits 1024 files, 1 MiB/file and 16 MiB/library. | #200/#201: define a usable bounded output/input contract and expose useful failures. A valid producer result can exceed consumer ceilings; this does not establish that a real corpus currently does. Do not blindly raise every limit. |
| Current-source workflow → `KETHO_TYPE_CATALOG.md` | Workflow selects six catalogs; documentation claimed all nine listed resources. | Name the actual selection. Parser support, workflow selection and executed donor coverage are different facts. |

The public report deliberately omits the embedded Library text but still includes
file hashes, byte lengths, mappings and source metadata. It is not hash-free and
should not be described as such. Keep only what real consumers need; deleting
one duplicated payload does not complete the handoff or remove all bureaucracy.

The old helper is active test code despite living under `examples/support`.
Deleting that directory without moving its useful cases would repeat the earlier
lint-based deletion error. Conversely, keeping it merely because tests use it
would preserve the duplicate producer indefinitely.

For new instructions, trace **command → output → actual reader → observable
result**, including the existing workflow and the test's producer. A green
emitter probe cannot stand in for a different application. Reuse one small
source fixture for the real producer/reader handoff when repairing #201; do not
create a schema certification suite. Partial status also requires inspecting
annotation count and issues: it is not a guarantee of usable Library output.

The [Ketho emitter at the recorded pin](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/luasrc/annotate/init.lua)
was re-read for default/nilability, stride and callback behavior. Those semantics
must survive driver consolidation; this source comparison is not donor parity.

## What to retain from old requirements and donors

The old architecture mandated evidence universes, service-only application access,
immutable publication/readback and independent acceptance gates for nearly every
operation. That explains much of the current plumbing; it is not a reason to
preserve it. Retain the useful user invariants, not that organizational model:

- Exact source path/range and selected flavor; ambiguity, unsupported input and
  incomplete output stay visible. A missing result is not automatically success.
- One input analysis supplies Main diagnostics and Library-aware facts. Querying
  an index does not repeat analysis. Unsaved editor text must not use saved offsets.
- Nonexecuting source reads, configured roots, bounded work/output, cancellation
  and preservation of user files. Old derived data gets a clear rebuild response,
  not silent deletion based on paths found in the old cache.
- Ketho's useful lowering: `Default ~= nil` includes false/zero, InnerType arrays,
  receivers, ordered parameters/returns and selected callbacks. Do not simplify
  unsupported types into `any` or lose the location of an independent valid item.

Re-read donors: [Ketho emitter at d0b5b51](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/luasrc/annotate/init.lua)
and [Emmy engine API at aaaca684](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/lib.rs).
This was source comparison, not full donor parity. Standard declarations initialize
separately; changed-only upstream updates do not establish dependent-file invalidation.

Corrections to Gemini remain important: the cited XML type-shape vector is charged
**after**, not inside, the item loop; this does not prove quadratic work. A module's
`allow(dead_code)` is not proof it has no callers. `generated_api.rs` has a live
purpose. Do not derive `Deserialize` on invariant-bearing owners merely to delete
`Wire` types: plain DTOs may derive it; executable/validated state needs its existing
admission boundary. No 70–80% saving has been established by measurement.

External references checked on 2026-10-21: [Serde unknown-field behavior](https://serde.rs/container-attrs.html),
[Git blob/symlink behavior](https://git-scm.com/docs/git-cat-file),
[SQLite FTS5](https://www.sqlite.org/fts5.html), and the selected
[MCP 2025-11-25 base protocol](https://modelcontextprotocol.io/specification/2025-11-25/basic).
FTS token/phrase matching is not arbitrary substring search. Specify the supported
query semantics before claiming repository text search; do not add an engine to
avoid deciding what the user query means.

## Material to carry into the rewritten documentation

Keep the manifesto short and implementation organized by three workflows:
generate a library; check an addon; index/query/read source context. Each recipe
needs only inputs/defaults, the existing owners, the data passed between them,
observable output/failure and old code to remove. It must say whether behavior
exists, was exercised, or remains work; no new status framework is needed.

| Old material | Retain in the relevant workflow | Retire instead of copying |
| --- | --- | --- |
| `PLATFORM_SOURCE_CONFIGURATION.md`, `PLATFORM_STORE_NAMESPACE.md` | Source/flavor selection, Main/Library roles, isolation and current reopen limitations. | Frozen epochs, migration promises, old-byte preservation and claims the existing CLI is absent. |
| `PLATFORM_DIRECT_PRODUCERS.md`, `PLATFORM_DIRECT_SERVICE.md`, `PLATFORM_RECOGNIZER_ASSERTIONS.md` | Actually consumed TOC/XML/call/state relations, locations and ambiguity. | Mandatory staged proof choreography, version histories, fingerprints as completion obligations. |
| `PLATFORM_RAW_INVENTORY_GRAPH.md`, `PLATFORM_INVENTORY_SPANS.md` | Indexed versus loaded files; whole-file versus byte-range locations. | Raw inventory certificates and legacy replay acceptance. |
| `PLATFORM_XML_SOURCE_MAPS.md` | XML entity/newline mapping, CDATA/comment gaps, virtual-Lua coordinates and ambiguous carets. | Replay8/9/10 narratives and historical gate totals. |
| `wow-recognizers/e2/PARTITIONS_AND_VERSIONING.md` | Replace stale derived relations when source/selection changes. | Frozen report9, shadow/holdout/promotion and per-feature compatibility matrices. |

Do not rewrite these nine notes by swapping strings again. Move each needed fact
once to its workflow/owner note, remove superseded prose and redirect actual links.
Keep the seven already recorded broken Store links in #112's cleanup; do not restore
deleted E2 specifications to satisfy them. A new audit is not a replacement tracker.

Priority: #200 → #201 for generation-to-addon checking; #131/#169/#122 for a reopened
body/symbol query with located text. Attach #194/#107/#112 reductions to those
consumers, not as an unlimited prerequisite phase. Use these concrete observations
when the implementation reaches them: correct/wrong Library argument at the addon
line; a body-only term found after process restart; changed/deleted source no longer
producing stale hits; a missing input reported honestly. Do not expand this into an
all-editor, historical-cache or runtime-WoW qualification campaign.

Remaining uncertainty: no complete semantic audit of every source file, no full
annotation corpus parity, no Windows runtime/protocol session run and no measured
end-to-end performance. The material above prepares a safer rewrite; it does not
claim that the product or future documentation is error-free.
