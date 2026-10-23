# Verified basis for the WoW linter rewrite

**Review date:** 2026-10-23. **Project source:**
[`c36a0581190a4d08b59c9a19b69ea728b03da5c3`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/c36a0581190a4d08b59c9a19b69ea728b03da5c3).
**Blizzard source inspected:** Gethe `live`
[`e7aa82019dc298e0351c7159d60c2a9b916b9b67`](https://github.com/Gethe/wow-ui-source/commit/e7aa82019dc298e0351c7159d60c2a9b916b9b67),
commit labeled 12.1.5 (70529), 2026-10-22.

This records source-verified findings and their implementation consequences.
The [review register](AUDIT_REGISTRY.md) separately accounts for every baseline
Markdown file, preserved ideas, code traces and remaining questions. Reading a
document is not verification of its implementation. The supplied Claude review
is `AUDIT_REVIEW_2026-10-23.md`, reviewing the same project commit.
Implementation is in [IMPLEMENTATION.md](IMPLEMENTATION.md), ordered work in
[REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## Judgment

The main diagnosis is correct: useful EmmyLua, Ketho and TOC/XML code is buried
under multiple representations, generic rule/graph/storage platforms and frontend
handoffs that lose the result. The solution is a direct **WoW-aware addon check**,
with generation and source lookup serving it. Shrinking files or compiling the
workspace does not establish that this check works.

| User operation | Source-inspected state at the baseline |
| --- | --- |
| Generate a Library | Public generator and useful native projection exist; source metadata/filtering gaps remain. |
| Check an ordinary addon | Shared Main+Library exists; normal setup/stdlib, useful diagnostics and direct frontend wiring remain unfinished. |
| Refresh/editor check | Conservative updates exist; LSP overlay is one-file and some TOC/package update routes remain unsupported. |
| Query source after restart | Useful selection/SQL/excerpt code exists; direct body search without replay reconstruction is not delivered. |

No addon workflow, performance benchmark or test suite was run in this review.
The owner directs implementation before tests. The local shell did not expose
Cargo/Rust; this is an environment observation, not evidence against Claude's
separate reported run or a reason to call the code correct.

## Source-verified findings

Links point to current files; conclusions here are tied to the baseline above.

| Finding | Evidence | Decision |
| --- | --- | --- |
| Shared analysis exists but STD is not loaded | [`references.rs`](../crates/wow-emmy/src/references.rs), [`syntax/analyzer.rs`](../crates/wow-emmy/src/syntax/analyzer.rs); pinned upstream `new`/`init_std_lib` | Reuse the shared owner, configure Lua51 and load embedded STD once; apply WoW availability separately. |
| Most diagnostics and messages are lost | `accepted_code` in the analyzer admits four codes; [`syntax.rs`](../crates/wow-emmy/src/syntax.rs) has no message; normalized severity is Error | Preserve code/message/actual severity/range. Upstream severity is retained as metadata today, so it is the displayed/normalized severity that is wrong. |
| LSP bypasses project context | [`overlay/state.rs`](../crates/wow-project/src/overlay/state.rs) builds one-file input; [`diagnostics.rs`](../crates/wow-service/src/frontend/lsp/diagnostics.rs) emits constant prose | Call the configured Main+Library operation with unsaved buffers. |
| Selected rules run before filtering | [`engine.rs`](../crates/wow-rules/src/engine.rs) runs both providers; [`projection.rs`](../crates/wow-service/src/local/projection.rs) filters evaluations afterward | Select before execution; keep unrelated rule failures out of type-check completeness. |
| Check status aggregates unrelated capabilities | The same projection builds all component capabilities; [`service.rs::derive_status`](../crates/wow-service/src/service.rs) treats any partial capability as a partial check | Derive completeness from requested checks and their inputs, retaining the actual limitations. |
| Git I/O is repeated and blocking | [`io.rs`](../apps/wow-reference-builder/src/io.rs) runs `ls-tree` and `cat-file` for each file | Batch exact regular-file reads and propagate cancellation through child I/O. |
| Compatibility/certification wrappers remain | [`wow-emmy/compatibility.rs`](../crates/wow-emmy/src/compatibility.rs), [`wow-annotations/compatibility.rs`](../crates/wow-annotations/src/compatibility.rs) | Remove through real readers; Cargo pins and useful input validation remain. |
| XML inline context is incomplete | [`xml_lua.rs`](../crates/wow-project/src/xml_lua.rs) explicitly records `not_evaluated_unwrapped_source` for implicit receivers | Preserve mappings and add only supported handler/chunk contexts. |
| Reads rebuild analysis | [`replay/mod.rs`](../crates/wow-project/src/replay/mod.rs) uses `hydrate_owner` and recapture comparison | Store/query current rows; checking itself needs no persistence. |
| SQLite is nested in hex/JSON | [`wow-store/src/search/mod.rs`](../crates/wow-store/src/search/mod.rs) serializes/deserializes the inner SQLite image | One directly queried database. The implementation is in Store, not Search. |
| FTS does not index source bodies | The same file creates FTS on `owner_metadata_label`; document ID is UNINDEXED | Index body text and keep exact symbol lookup separately. |
| Four query families self-measure | [`federation/pagination.rs`](../crates/wow-search/src/federation/pagination.rs), `entities/query`, `reference/query`, `file_set/retrieval` repeat eight-pass sizing | Plain bounded results; no embedded self-size/hash fixed point. |
| Source excerpts are escaped unreadably | [`render.rs`](../crates/wow-context/src/render.rs) Unicode-escapes ASCII punctuation | Render matching source as source, with only transport-appropriate escaping. |
| Old lifecycle and fingerprint code is active | [`wow-store/src/project/mod.rs`](../crates/wow-store/src/project/mod.rs), [`graph/fingerprints`](../crates/wow-project/src/graph/fingerprints), [`xml_fingerprints.rs`](../crates/wow-project/src/graph/xml_fingerprints.rs) | Delete with obsolete consumers; do not claim the subsystem is already removed. |

## Important gaps missed or under-specified by the previous audit

### Flavor is confused with API environment

The builder accepts an arbitrary `environment` string; old examples/workflow use
`Mainline`. The emitter filters only `SystemFacts.environment` with `All` or exact
string equality in [`native.rs`](../crates/wow-annotations/src/native.rs).
`CallableFact` in [`native_model.rs`](../crates/wow-reference/src/native_model.rs)
has no environment field.

In the inspected Blizzard generated corpus, non-`All` occurrences are two
`SecureOnly` systems and the function `GetForbiddenObjectTable` inside the `All`
FrameScript system. The member flag therefore does not participate in the
current filter: this declaration can be emitted into an ordinary-addon Library.
This is a source-traced defect, not an executed generation result. See
[FrameScript](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/FrameScriptDocumentation.lua),
[CombatLogSecure](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/CombatLogSecureDocumentation.lua)
and [PingManagerSecure](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/PingManagerSecureDocumentation.lua).

**Decision:** select flavor and load context independently; preserve/inherit
member API environment and exclude secure-only availability from ordinary-addon
typing. `All` does not mean every client flavor.

### Signatures omit meaningful current Blizzard metadata

At the baseline the normalized field/callable model lacks `MayReturnNothing`,
`Mixin`, callable environment and several restriction fields. Raw values and
metadata sidecars survive; that is not the same as semantics reaching the type
checker. For example,
[`EncounterWarningsDocumentation.lua`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/EncounterWarningsDocumentation.lua)
contains `MayReturnNothing` with a non-nil declared return, mixins, field
`SecretValue` and function-level `SecretArguments` policies.

**Decision:** model these facts explicitly and project only representable
semantics through EmmyLua. Preserve losses. `SecretArguments` is not a ready-made
list of argument positions, and a restriction flag does not prove call-site
illegality. The existing narrow first-return secret/predicate projection in
[`native_view.rs`](../crates/wow-reference/src/native_view.rs) remains useful.

### Stride packs and nilable contents

These are additional source-traced defects, with concrete current-Blizzard
inputs. They were not established by merely counting documents or comparing
the generator to Ketho.

**Repeated groups are rejected.**
[`C_ArtifactUI.GetMetaPowerInfo`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/ArtifactUIDocumentation.lua)
declares three return fields with `StrideIndex` 1, 2 and 3. Its Blizzard UI
consumer, `MetaPowerTooltipHelper` in
[`Blizzard_ArtifactUI.lua`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ArtifactUI/Blizzard_ArtifactUI.lua),
iterates the results in groups of three. The source therefore establishes
repeated ordered triples for this API.

[`native/fields.rs::convert`](../crates/wow-annotations/src/native/fields.rs)
collapses each positive index to `variadic = true`.
[`ketho.rs::validate_fields`](../crates/wow-annotations/src/ketho.rs)
allows that flag only on the last field. The first field therefore fails;
`native.rs` records `renderer_InvalidVariadic` and omits the callable while
retaining independent valid declarations. No generator run is claimed here.

**Nullable contents are lost.**
[`C_EquipmentSet.GetItemIDs`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/EquipmentManagerDocumentation.lua)
declares `MayReturnNothing`, a `table` with `InnerType = number`,
`Nilable = false`, and `NilableContents = true`. The same contents flag appears
on `GetIgnoredSlots` and `GetItemLocations`, and on the argument/return packs
of [`C_AddOnProfiler.MeasureCall`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/AddOnProfilerDocumentation.lua).
`native_model.rs::FieldFact` has no contents-nilability field;
[`ketho/callbacks.rs::field_type`](../crates/wow-annotations/src/ketho/callbacks.rs)
builds the array from `InnerType` without it. The typed projection loses the
distinction even though raw metadata survives. Do not infer an executed
consumer error, or a non-nil `LuaValueVariant` alias, from that loss alone.

**Decision:** preserve numeric pack positions and element nilability in the
existing typed model. Use source-supported Emmy lowering, document any loss of
tuple correlation, and keep unsupported signatures explicit. Callable
zero-result alternatives, nullable containers and nullable elements are
independent. [The projection guide](KETHO_RUST_PORT.md) owns this behavior.

### Retained metadata is not a consumer handoff

`NativeLibrary.sources` retains raw field values. Its `metadata_sidecars`
contains field names and source links, not typed semantic values; the builder
includes them in `source-report.json`. Moreover,
[`local/native_artifact/wire.rs::LibraryBinding`](../crates/wow-service/src/local/native_artifact/wire.rs)
deserializes `metadata_sidecars` as `IgnoredAny`. This is evidence about that
reader, not a claim that every restriction fact is unused: the separate narrow
`native_view` secret projection exists.

**Decision:** pass the selected typed context/restriction/origin fields through
the ordinary Library reader. A generated Lua Library must be usable without
the original checkout; missing optional metadata limits only affected checks.
Keeping a raw sidecar or reparsing the original source is not completion of
this producer-to-consumer path.

### Analyzer settings are not yet shared configuration

[`syntax/analyzer.rs::build_registered`](../crates/wow-emmy/src/syntax/analyzer.rs),
the shared constructor in [`references.rs`](../crates/wow-emmy/src/references.rs),
and [`flow.rs::build_analysis`](../crates/wow-emmy/src/flow.rs) each construct
`Emmyrc::default`, force Lua51 and set their own diagnostics toggle. A caller's
resolved diagnostic policy is not threaded through these constructors.

**Decision:** one configured analyzer owner for CLI, updates and editor checks.
Keep upstream diagnostics, selection of WoW checks and the CLI failure threshold
as separate controls. Reusing Emmy's configuration API is a target requirement,
not a capability established by the current fixed defaults.

### Standard Lua typing is not the WoW runtime

Pinned Emmy has embedded STD declarations and no WoW preset or standard-module
configuration switch. Loading STD fixes missing Lua type facts but can also make
host APIs resolve. **Decision:** keep type information and platform availability
distinct; check resolved symbols against selected WoW facts. Workspace ignore
settings do not filter `init_std_lib`. See [the verified donor APIs](EMMYLUA_UPSTREAM.md).

### Check completeness is not scoped to requested work

[`local/projection.rs::check_context`](../crates/wow-service/src/local/projection.rs)
filters returned rule evaluations, but its `components` call has no selected-rule
argument. That projection includes secret-rule capability state independently
of selection. For either native source or retained-artifact input it also adds
`wow-annotations` as `Degraded` with a `Partial` capability unconditionally.

[`service.rs::derive_status`](../crates/wow-service/src/service.rs) examines every
component and every capability, then chooses `Partial` ahead of findings/clean.
[`local/response.rs`](../crates/wow-service/src/local/response.rs) and
[`apps/wow/src/output.rs`](../apps/wow/src/output.rs) map that to CLI exit 2.
Consequently a successful native check reaches partial status regardless of the
selected rule's result. This follows the source branches; no command was run.
It does not establish that all its analysis is unusable or that findings vanish.

**Decision:** select work before execution and derive completeness from the
checks/files requested and their actual dependencies. Preserve limitations of
enabled checks, lost inputs and cancellation. Do not fix this by declaring every
component complete. The old requirement to retain raw findings alongside
blockers remains useful; optional component health is not a global clean gate.

### A typed local receiver can enter WoW API absence checking

The earlier shadowing hypothesis is now narrowed by the pinned Emmy source.
[`references.rs::reference_resolution`](../crates/wow-emmy/src/references.rs)
records receiver spelling and `Resolved`/`Possible`/`Unresolved`, without the
receiver's lexical declaration identity. A missing member on a concrete `Ref`
or `Def` type becomes `Unresolved`; broad/table-literal types remain `Possible`.
[`engine.rs::api_lookup_key_parts`](../crates/wow-rules/src/engine.rs) admits an
API namespace by that receiver spelling.

At the pinned Emmy revision,
[`infer_name_expr`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/infer_name.rs)
uses a local declaration before global lookup, and the
[local-type lookup](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/narrow/mod.rs)
can return its concrete annotated type.
[`get_index_decl_type`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/mod.rs)
returns `infer_index_expr(...).ok()`; a missing member on that type follows
[`FieldNotFound`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/infer/infer_index/mod.rs).
The separate [semantic-declaration lookup](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/semantic/semantic_info/infer_expr_semantic_decl.rs)
also has no member to return when the class and its bases lack it.

This establishes a conditional false-positive path: a local named like a selected
API namespace, but typed as an unrelated class, has a missing class member;
the adapter labels it `Unresolved`, and the rule queries the same-spelled WoW
namespace. If that API partition proves the key absent and the relevant input
gates are complete, the rule emits an API-absence finding about the local use.
The ordinary Lua member error can be valid while that WoW attribution is wrong.
This is a source-derived control-flow conclusion, not an executed Lua example.
The simple untyped `{}` example does not establish the same path.

The exact lookup itself preserves useful boundaries:
[`ReferenceView::lookup`](../crates/wow-reference/src/deterministic_view.rs)
returns conflicts before records, permits positive records under partial
coverage, and only a complete selected partition can prove absence. Native
projection additionally requires the admitted corpus closure and no blocking
API loss. Preserve those checks; they cannot compensate for a wrong receiver.

**Decision:** carry or query Emmy's lexical binding and selected API origin before
making a WoW claim. Unknown provenance stays unresolved; do not implement another
name-resolution engine. The current producer collects direct static member
calls, and the rule's spelling filter skips namespace aliases. This is not full
global/member/alias availability coverage.

### An unresolved Secret producer is treated as a broken fact graph

The member-call producer in
[`references.rs`](../crates/wow-emmy/src/references.rs) emits a fact for each
supported direct member call, including `Possible` and `Unresolved` references.
[`flow.rs::direct_member_initializer`](../crates/wow-emmy/src/flow.rs) joins a
local initializer to that exact call/reference without requiring `Resolved`.
These are coherent facts about incomplete semantic knowledge.

In [`engine.rs::evaluate_secret`](../crates/wow-rules/src/engine.rs), an exact
matching restriction record first makes the producer spelling relevant. The
next combined guard treats both a non-resolved reference and mismatched
call/reference/path/digest identities as `RuleFactReferenceGraphInvalid`. This
happens before collecting the binding's operations and their capability gates.
Consequently, if the selected restriction metadata recognizes a producer but
Emmy does not resolve its member call, the rule execution can fail instead of
reporting the affected use as not evaluated. This conditional source path was
traced; a selected-corpus example and observed diagnostics were not executed.

The identity checks have a different purpose. The normal constructor joins the
call to the reference it actually emitted, and Project validates the shared
Main/member/flow reports and Library bindings. No normal path producing those
identity mismatches was established.

**Decision:** retain rejection of malformed or stale fact identities. Handle
semantic uncertainty as a limitation of the relevant check/use, preserve other
findings, and establish the producer's actual API binding before applying secret
metadata. A missing row in the intentionally partial restriction partition is
not authoritative evidence that a producer has no restrictions.

### TOC and XML need source-specific context

The builder separately scans document-TOC lines; Project's richer
[`load/toc.rs`](../crates/wow-project/src/load/toc.rs) can reject Interface mismatch
and mark unknown metadata as incomplete. Generated API TOCs may have no Interface;
cosmetic metadata must not invalidate unrelated checks. XML's exact virtual-byte
maps already exist, but implicit handler receivers are explicitly unevaluated.
**Decision:** share the useful selection parser without importing Project/Graph
machinery into generation; make load uncertainty local and add supported chunk/
handler context. Preserve UTF-8 spans, XML mapping pieces and load order.

### XML-only input is refused before its Lua can be analyzed

[`load/mod.rs`](../crates/wow-project/src/load/mod.rs) rejects `TocLoadPurpose::Main`
when `loader.files` is empty, even if the parsed XML index contains inline Lua.
[`package_closure.rs::into_namespaced_main`](../crates/wow-project/src/load/package_closure.rs)
rejects an all-empty physical Lua inventory after selecting reachable packages;
[`inventory.rs::ProjectInputInventory::build`](../crates/wow-project/src/inventory.rs)
also requires physical files. These boundaries precede XML virtual-unit analysis.

The selected Blizzard
[`Blizzard_ChatBubble.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ChatBubble/Blizzard_ChatBubble.toc)
lists only [`ChatBubbleTemplates.xml`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ChatBubble/ChatBubbleTemplates.xml),
which contains an inline `OnLoad` body and no external Lua/include. This is a
real source shape, not an executed failure report; separate Interface admission
can reject that original TOC earlier. A package with no physical Lua can coexist
with other packages supplying physical files, so the gap is not a claim that
every XML-only package is excluded from a larger analysis.

**Decision:** admit selected load facts and the actual physical/virtual language
units. An XML-only addon must not need an artificial `.lua` file. An empty
language scope, an unsupported handler context and failed acquisition are
different states.

### Package path identity does not model addon chunk arguments

`into_namespaced_main` gives files paths such as `packages/<package>/<path>`;
it retains source bytes and isolates file identities. The traced
[`analyzer.rs`](../crates/wow-project/src/analyzer.rs) registers those physical
bytes and XML-derived units without an addon-name/private-table input. No
producer-to-consumer chunk-context binding was found in Project/Emmy.

**Decision:** preserve lexical file scopes and model a shared private table
within each addon, separately for dependencies, through supported Emmy context
or mapped virtual declarations. Path prefixes are not a substitute. Do not infer
that namespace from local variable spelling, turn all globals into per-file
locals, or reinterpret varargs inside unrelated nested functions.

### XML syntax work and load completeness disagree

[`load/xml.rs::parse`](../crates/wow-project/src/load/xml.rs) adds a blocking
`InlineLuaNotAnalyzed` issue for nonempty text, CDATA and entity events.
The same XML index can extract a supported inline handler body, and
[`xml_lua.rs::prepare`](../crates/wow-project/src/xml_lua.rs) passes it into
virtual Lua syntax analysis. The legacy issue is not reconciled with that work:
`ProjectLoadPlan::external_files_complete` remains false, Service exposes
`project.load_files.resolved = Partial`, and package projection can add
`PackageFileClosurePartial`. See
[`load/mod.rs`](../crates/wow-project/src/load/mod.rs),
[`package_closure.rs`](../crates/wow-project/src/load/package_closure.rs) and
[`local/projection.rs`](../crates/wow-service/src/local/projection.rs).

This does **not** by itself block the native XML census. XML records explicitly
set `file_enumeration_complete = true`; its independent consumer uses that
flag and its own source checks, not the legacy issue list. Nor does successful
syntax analysis establish implicit handler receiver or runtime semantics.

The deeper trace confirms that `xml_lua::prepare` handles both single-TOC and
package load plans. For reachable or conditionally reachable packages it collects
inline units and `analyzer.rs` feeds them into the same Main/Library session.
`xml_index/inline.rs` retains entity spellings, XML newline transformations and
discontinuous source pieces around comments/CDATA; mapped spans are validated
against captured XML. Preserve this working mechanism. A partial capability
label alone is not evidence that package inline Lua is skipped.

**Decision:** distinguish file closure, syntax analysis and supported handler
context. Remove the stale blanket blocker for recognized/extracted bodies;
keep actual unsupported context and malformed input visible at the right scope.

### LSP rejects valid version gaps

[`overlay/state.rs::change_document`](../crates/wow-project/src/overlay/state.rs)
requires `current.client_version.checked_add(1) == Some(version)` after its
exact-replay case. The
[`Service LSP adapter`](../crates/wow-service/src/frontend/lsp.rs)
marks any rejected change as requiring resynchronization and clears diagnostics.
Thus a valid change from version 3 to 5 is rejected solely for the gap.

The official
[LSP versioned-document definition](https://github.com/microsoft/language-server-protocol/blob/baf2db31b5fb314a8f358f9a839d3c66e9d98734/_specifications/lsp/3.18/types/versionedTextDocumentIdentifier.md)
requires increasing versions and explicitly allows nonconsecutive values.
This is a source/protocol mismatch; no editor session was executed.

**Decision:** preserve exact text/snapshot guards, ordered edit application and
stale-result rejection while accepting increasing client versions. Internal
revisions and client versions are different values. Do not carry the old
exact-next-version requirement into the replacement frontend.

### Planned behavior is still planned

[`wow-rules/src/lib.rs`](../crates/wow-rules/src/lib.rs) lists only API existence
and a narrow local secret-operation rule as active providers. Callback annotation
generation does not implement event-registration/handler validation or the
broader interprocedural summaries described by older plans.
[`LiveProjectStore::update`](../crates/wow-service/src/live_project/operations.rs)
also explicitly rejects TOC/package-selected load
configurations before updating; fresh project construction and lower-level
update machinery exist. These are bounded current limitations, not evidence
that every load or update operation is absent. Keep them visible in the
[implementation order](REFACTOR_QUEUE.md).

## Reconciliation of the supplied Claude review

| Claim/recommendation | Review decision |
| --- | --- |
| Old producer/verifier, limits and TOC mode claims are stale after #241–#243 | Confirmed. Public builder probes replaced the example copy; `/1` verifier is removed; regular-file checks and Library discovery/shared acquisition ceilings exist. Do not reimplement them. |
| Builder/reader maximum capacity is solved | Only acquisition/output ceilings align. Replay aggregate and graph requalification limits still differ; public maximum-size checking was not demonstrated. |
| Emmy cancellation is absent | Incorrect as a blanket claim. Stop checks exist around files/conversion; pinned synchronous per-file analysis and blocking Git-child I/O have separate limitations. |
| CI runs no `cargo test` | Confirmed from `quick-check.yml`. It runs repository checks, fmt, `cargo check` and Clippy, with extra exact-head/archive machinery. Under current owner policy, tests are deferred; green CI still means build evidence only. |
| 1,085 passed / 13 failed / 23 ignored, about 42 minutes | **Reported by the supplied review**, not independently rerun and no raw run log supplied here. Keep the reported failures visible; do not relabel them current verified results. |
| Five #232 regressions are rejection-phase assertions | Source review supports the decode/hydrate mismatch and early `?` exit examples. See `live_project/tests.rs`, `local/platform_input/tests.rs`, `tests/empty_packages.rs`, and `primary_callable_replay.rs`. No claim that invalid input is accepted at final admission. |
| Delete the other eight failing tests immediately | Rejected as a blanket recommendation after source inspection of all eight. Four Context tests mix frozen bytes with useful identity/bounds/privacy checks; the fifth freezes IDs/progressive behavior. The signature case expects rejection at store open instead of configuration admission; two other cases retain a writer-lock owner across a second open. See the [13-case register](AUDIT_REGISTRY.md#the-13-failures-reported-by-claude). Exact failing instructions remain unverified without logs. No test edits/runs. |
| Earlier reviewers wrongly claimed Rust unavailable | Tool availability differs by environment. Claude having Cargo does not disprove an earlier container limitation. |
| 329 Markdown files, 18 PLATFORM docs | 329 tracked Markdown files confirmed at baseline; the exact `docs/PLATFORM_*.md` glob has 16. Reachability depends on scan roots/link parsing; do not treat the reported reachability count as a semantic review of every file. |

## Documentation disposition

Historical E1/E2/E3/E7/platform contracts, old app recipes, duplicate status
matrices, glossary/gates and migration/proof plans are removed from the active
tree. Their useful ideas and the limits of source verification are accounted
for in the [review register](AUDIT_REGISTRY.md), with links to the exact old
revision. Root/owner instructions route to the manifesto, behavior, implementation,
donors and short owner guides. Package READMEs identify current code/reuse.

Preserved domain knowledge: source/flavor/context distinction, ordered TOC/XML
loading, dependency ambiguity, Main/Library/STD roles, exact UTF-8/virtual ranges,
Ketho type/callback/default/mapping semantics, events versus custom callbacks,
SavedVariables limits and narrow secret-flow constraints. The second reading
also restores explicit configuration ownership, deletion invalidation, semantic
admission after decoding, simple outline/neighborhood operations and query
scope/coverage distinctions. These live once in
[WOW_LINTER.md](WOW_LINTER.md), [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md) and
[EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md).

Third-party notices remain. `wow-annotations/src/aliases.rs` embeds its notice;
`wow-service/e3/native-primary-pack/artifact.md` is also retained unchanged because
existing tests embed it. That fixture is data, not an active specification.
Non-Markdown fixtures and CI workflows are unchanged. CLI help pointers and
source comments that named deleted guides now point to the current documentation;
no analysis or command logic changed.

## Remaining uncertainty

Real addon diagnostics, whole-source type closure, precise consumer compatibility,
all conditional TOC/XML cases, editor sessions, local-flow rule completeness,
maximum-size behavior and performance remain unexecuted. This does not block a
coherent implementation plan, but it does block claims of full working coverage.
The typed-local receiver path is now traced through the pinned analyzer and the
rule consumer; an executed reproduction and coverage of other shadowing forms
remain open. `FallbackType` lowering has no located occurrence in the selected
generated corpus and is not a confirmed current-corpus defect. The package-only
configuration identity omits the XML adapter-profile field, but no stale-result
or cache consequence was established; do not turn that bookkeeping observation
into a new profile/version platform. Keep these limits separate from the
source-traced findings above. Update the relevant register row when evidence
changes; a separate audit framework is unnecessary.
