# Verified basis for the WoW linter rewrite

**Review date:** 2026-10-23. **Project source:**
[`c36a0581190a4d08b59c9a19b69ea728b03da5c3`](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/commit/c36a0581190a4d08b59c9a19b69ea728b03da5c3).
**Blizzard source inspected:** Gethe `live`
[`e7aa82019dc298e0351c7159d60c2a9b916b9b67`](https://github.com/Gethe/wow-ui-source/commit/e7aa82019dc298e0351c7159d60c2a9b916b9b67),
commit labeled 12.1.5 (70529), 2026-10-22.

This replaces the competing audit/coverage/completion documents. It records
verified decisions and useful remaining gaps, not a progress diary. The supplied
Claude review is `AUDIT_REVIEW_2026-10-23.md`, reviewing the same project commit.
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

### Standard Lua typing is not the WoW runtime

Pinned Emmy has embedded STD declarations and no WoW preset or standard-module
configuration switch. Loading STD fixes missing Lua type facts but can also make
host APIs resolve. **Decision:** keep type information and platform availability
distinct; check resolved symbols against selected WoW facts. Workspace ignore
settings do not filter `init_std_lib`. See [the verified donor APIs](EMMYLUA_UPSTREAM.md).

### TOC and XML need source-specific context

The builder separately scans document-TOC lines; Project's richer
[`load/toc.rs`](../crates/wow-project/src/load/toc.rs) can reject Interface mismatch
and mark unknown metadata as incomplete. Generated API TOCs may have no Interface;
cosmetic metadata must not invalidate unrelated checks. XML's exact virtual-byte
maps already exist, but implicit handler receivers are explicitly unevaluated.
**Decision:** share the useful selection parser without importing Project/Graph
machinery into generation; make load uncertainty local and add supported chunk/
handler context. Preserve UTF-8 spans, XML mapping pieces and load order.

## Reconciliation of the supplied Claude review

| Claim/recommendation | Review decision |
| --- | --- |
| Old producer/verifier, limits and TOC mode claims are stale after #241–#243 | Confirmed. Public builder probes replaced the example copy; `/1` verifier is removed; regular-file checks and Library discovery/shared acquisition ceilings exist. Do not reimplement them. |
| Builder/reader maximum capacity is solved | Only acquisition/output ceilings align. Replay aggregate and graph requalification limits still differ; public maximum-size checking was not demonstrated. |
| Emmy cancellation is absent | Incorrect as a blanket claim. Stop checks exist around files/conversion; pinned synchronous per-file analysis and blocking Git-child I/O have separate limitations. |
| CI runs no `cargo test` | Confirmed from `quick-check.yml`. It runs repository checks, fmt, `cargo check` and Clippy, with extra exact-head/archive machinery. Under current owner policy, tests are deferred; green CI still means build evidence only. |
| 1,085 passed / 13 failed / 23 ignored, about 42 minutes | **Reported by the supplied review**, not independently rerun and no raw run log supplied here. Keep the reported failures visible; do not relabel them current verified results. |
| Five #232 regressions are rejection-phase assertions | Source review supports the decode/hydrate mismatch and early `?` exit examples. See `live_project/tests.rs`, `local/platform_input/tests.rs`, `tests/empty_packages.rs`, and `primary_callable_replay.rs`. No claim that invalid input is accepted at final admission. |
| Delete the other eight failing tests immediately | Too broad. Retire each with the removed behavior after checking its consumers. A failing fingerprint/XML case is not disposable solely because of its name. No test edits/runs in this documentation change. |
| Earlier reviewers wrongly claimed Rust unavailable | Tool availability differs by environment. Claude having Cargo does not disprove an earlier container limitation. |
| 329 Markdown files, 18 PLATFORM docs | 329 tracked Markdown files confirmed at baseline; the exact `docs/PLATFORM_*.md` glob has 16. Reachability depends on scan roots/link parsing; do not treat the reported reachability count as a semantic review of every file. |

## Documentation disposition

Historical E1/E2/E3/E7/platform contracts, old app recipes, duplicate audit/status
matrices, glossary/gates and migration/proof plans are removed from the active
tree. Root/owner instructions now route to the manifesto, behavior, implementation,
donors and short owner guides. Package READMEs only identify current code/reuse.

Preserved domain knowledge: source/flavor/context distinction, ordered TOC/XML
loading, dependency ambiguity, Main/Library/STD roles, exact UTF-8/virtual ranges,
Ketho type/callback/default/mapping semantics, events versus custom callbacks,
SavedVariables limits and narrow secret-flow constraints. These live once in
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
Recheck only the relevant changed path; do not create another audit framework.
