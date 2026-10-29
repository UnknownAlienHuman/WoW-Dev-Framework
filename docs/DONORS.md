# Donors and source boundaries

Use this as a short map from an existing upstream source to the Rust owner or
bounded behavior that can be reused. Read the source at the pinned revision
before copying behavior. Blizzard UI source is the authority for the selected
client's facts; Ketho is an annotation-semantics donor; Wowless is a comparative
implementation and data source. None of these makes runtime behavior statically
provable.

## Pinned sources and useful surfaces

| Donor | Revision and path | Reuse / owner | Boundary |
|---|---|---|---|
| EmmyLua analyzer | [`aaaca68425d9362876228649b0b8d92f07654daa`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/tree/aaaca68425d9362876228649b0b8d92f07654daa) | Public parser and semantic analysis in `wow-emmy`; use for Lua syntax, types, diagnostics and configuration. | Language analysis belongs upstream. Do not build a parallel Lua parser/type checker or depend on its private LSP internals. |
| Ketho annotation generator | [`d0b5b51fac4c52c493371b9b18e66ce604ea4326`](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326), `luasrc/annotate/init.lua`, `wowdoc/init.lua` | `wow-annotations/src/ketho.rs` and callbacks: typed projection, ordered params/returns, callback signatures and receiver conventions. | Semantics/output donor, not a requirement to preserve VS Code activation or runtime. |
| Ketho source loader and patches | Same Ketho revision: [`wowdoc/loader/init.lua`](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/init.lua), [`patches.lua`](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/patches.lua), [`doc_widgets.lua`](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/doc_widgets.lua) | Rust equivalents in `wow-reference/src/native*.rs`: normalize source facts, retain raw input, apply guarded named-field corrections and map documented receivers. | Do not execute the Lua loader or patch code. Port narrow, inspected semantics into bounded Rust over source data. |
| Ketho annotations | Same revision: [`Annotations/Core/Type`](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326/Annotations/Core/Type), [`Annotations/Core/Widget`](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326/Annotations/Core/Widget) | Existing native alias, structure, namespace, mixin, widget, literal and navigation modules under `wow-annotations` / `wow-ketho-literals`. | Preserve only understood, representable annotations. Keep losses and unsupported types visible; never silently widen them to `any`. |
| Blizzard UI source mirror | [`e7aa82019dc298e0351c7159d60c2a9b916b9b67`](https://github.com/Gethe/wow-ui-source/tree/e7aa82019dc298e0351c7159d60c2a9b916b9b67) | Selected API documentation tables, Lua/XML/TOC source and flavor-specific API availability; consumed by `wow-reference`, project loading and annotations. | Primary facts are revision- and flavor-bound. This reviewed source revision represents build 12.1.5 / 70529; it does not identify a permanently supported build. Respect Blizzard's source terms; do not assume a code license from the mirror. |
| Wowless UIObject model | [`c09acebd9be7d52e41154dff5276f4d62f3e4970`](https://github.com/wowless/wowless/tree/c09acebd9be7d52e41154dff5276f4d62f3e4970), [`uiobjecttypes.lua`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/uiobjecttypes.lua), [`uiobjects.yaml`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/data/products/wow/uiobjects.yaml) | Compare inheritance/script lookup algorithms and bounded type/script facts against selected source; potential design input for XML script/object checks. | Comparative model/data, not Blizzard authority or a runtime dependency. Verify any finding against selected source and represent uncertainty explicitly. |
| Wowless XML and TOC logic | Same Wowless revision: [`xmleval.lua`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/xmleval.lua), [`loader.lua`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/loader.lua), [`toc.lua`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/toc.lua) | Compare XML template/script binding, ordered TOC entries, dependency aliases, product-channel gating, the `LoadWith` reverse-trigger model, exact `Bindings<suffix>.xml` selection and binding callback wrapping with `wow-project`. | Comparative behavior is not Blizzard authority. Do not import the interpreter, execute addons or promote donor ordering/security assumptions into client facts without selected-source evidence; preserve unsupported semantics explicitly. |
| quick-xml | Declared `0.38`; lock resolves `0.38.4` in [`Cargo.lock`](../Cargo.lock); [0.38.4 docs](https://docs.rs/crate/quick-xml/0.38.4) | Existing XML event reader for source-located XML traversal. | Parse as data; XML callbacks/scripts are not executed. |
| rusqlite / SQLite FTS5 | `rusqlite 0.37.0` with `bundled`, `serialize`, `backup` in [`crates/wow-store/Cargo.toml`](../crates/wow-store/Cargo.toml); [rusqlite 0.37 docs](https://docs.rs/rusqlite/0.37.0/rusqlite/); [SQLite FTS5 reference](https://www.sqlite.org/fts5.html) | SQLite persistence and optional full-text indexing/search in `wow-store` / `wow-search`. FTS5 is SQLite's virtual-table full-text module. | Search is auxiliary; addon checking must not require a populated search index. Use direct SQLite docs for FTS semantics, not an assumption based on crate feature names. |
| LuaLS library consumer | [Libraries](https://luals.github.io/wiki/libraries/), [annotation compatibility](https://luals.github.io/wiki/annotations/) | External consumer for generated annotation libraries via workspace library configuration; useful interoperability target. | A `.lua` extension alone does not establish compatibility. LuaLS warns its LuaCATS annotations are no longer cross-compatible with EmmyLua annotations since v3. Validate emitted constructs with target consumers or provide a tested profile. |

Wowless's own code is MIT-licensed at the pinned
[`LICENSE`](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/LICENSE);
that notice excludes separately licensed `vendor/` and `elune/` components.
Ketho records MIT terms in its pinned
[`LICENSE`](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/LICENSE);
this project retains attribution in
[`THIRD_PARTY_NOTICES.md`](../crates/wow-annotations/THIRD_PARTY_NOTICES.md).
EmmyLua's pinned
[`LICENSE`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/LICENSE)
governs that upstream code. Review each donor's license and data provenance
before copying code or source-derived datasets; a software license does not
settle rights in Blizzard-derived data.

The crate workspace pins EmmyLua through `Cargo.toml`; its revision is also
the parser and semantic API contract for this checkout. The Ketho revision is
the reviewed comparison for annotation behavior, not a promise of byte-for-byte
output parity. Gethe's revision is the primary input for source-sensitive WoW
facts. Keep these roles distinct when a donor and current client data disagree.

## WoW source facts and limits

Use Gethe's selected Blizzard API docs and authored Lua/XML/TOC as the source
of truth. Do not treat Ketho's annotations or Wowless's models as newer or
more authoritative facts. The exact source revision, client flavor and selected
TOC control what is in scope.

The pinned
[`AddOnsDocumentation.lua`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/AddOnsDocumentation.lua)
explicitly describes the addon table as the second result of chunk `...` and
states that `AllowAddOnTableAccess: 1` only opts that table into retrieval through
`C_AddOns.GetAddOnLocalTable`. The pinned
[`Blizzard_CombatAudioAlerts.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlerts.toc)
uses that directive. This is stronger evidence for ordinary chunk context than a
runtime-emulator convention, and it keeps table injection separate from external
retrieval permission.

In the pinned API docs, observed `Environment` values include `All` and
`SecureOnly`. These name UI environments, not client flavors such as `Mainline`
or `Classic`; `All` is not a statement that an API is present in every client
flavor. The current model preserves and filters on system-level `Environment`
through [`SystemFacts`](../crates/wow-reference/src/native_model.rs) and the
Reference/annotation projections ([`native_view.rs`](../crates/wow-reference/src/native_view.rs),
[`native.rs`](../crates/wow-annotations/src/native.rs)). It does not preserve a
callable-level `Environment` override in [`CallableFact`](../crates/wow-reference/src/native_model.rs),
so that member-level availability remains unmodeled. At this source revision,
for example, `FrameScript.GetForbiddenObjectTable` is documented `SecureOnly`.

Ketho-specific compatibility gaps to check against the current Rust owner:

- Repeated `StrideIndex` groups and `NilableContents` occur in the selected
  source. The Blizzard consumer of `GetMetaPowerInfo` demonstrates repeated
  triples; preserving a single terminal vararg does not cover that shape.
- `Mixin` annotations describe composition; do not substitute a guessed single
  parent relationship.
- `MayReturnNothing` is distinct metadata about a callable's return behavior.
- `SecretArguments` is a function-level policy string, not a list of argument
  positions. Do not reinterpret it as positional facts.

Consult [WoW behavior](WOW_LINTER.md) for rule priorities and static-analysis
limits. Dynamic loading, runtime-created objects, taint, combat behavior and
arbitrary Secret-value flow cannot be certified by a partial static model.
When extending the native path, trace the selected source field into its Rust
fact, projection and consumer before deciding that the behavior is supported.
Retain raw evidence and report unsupported forms as partial or unknown rather
than silently dropping their meaning.

For annotation syntax, use the pinned Emmy
[parser](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_parser/src/grammar/doc/types.rs),
[type lowering](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/compilation/analyzer/doc/infer_type.rs)
and [return guide](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/docs/emmylua_doc/annotations_EN/return.md).
Nullable element types and homogeneous variadic returns have source support;
an exact repeated heterogeneous tuple form was not established. The
[projection guide](KETHO_RUST_PORT.md#what-the-pinned-emmy-consumer-can-represent)
records the verified forms and remaining limit. Do not invent syntax or a new
type engine to fill that gap.

## Wowless

For Wowless comparisons specifically, `uiobjecttypes.lua` precomputes ancestry
for `InheritsFrom`/`HasScript`; `uiobjects.yaml` lists type inheritance and
scripts. `xmleval.lua` binds XML script functions/methods and inline bodies,
while `toc.lua` exposes source-order and conditional parsing. These are useful
ideas and comparison points only. The `loader.lua`/`xmleval.lua` execution path
models a client-like runtime, including addon chunk arguments `(addonName,
privateTable)`; the linter must read source and model only the statically
supported namespace relationship. Do not run Ketho or Wowless Lua loaders.

The inspected loader gives `SuppressLocalTableRef` its own branch and allocates
bootstrap environments separately. It sets the addon environment to `nil` for
that directive but still calls external Lua with both `addonName` and the
`addonEnv` value. Gethe's selected
[`Blizzard_RecruitAFriend.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_RecruitAFriend/Blizzard_RecruitAFriend.toc)
confirms that the restricted/internal metadata still occurs in selected Blizzard
source, but neither source establishes its current client effect. Do not turn the
donor's `nil` convention into a supported client suppression rule. Preserve the
raw directive as unsupported context and limit only conclusions that depend on
chunk ellipsis/private-table semantics.

The loader also compares product gates, records `LoadWith` reverse triggers and
searches a finite bindings-suffix list; its XML evaluator wraps binding bodies
with `keystate`, compiles known handlers with their script parameters, and loads
free inline Script bodies through a different call. These are reviewed donor
behaviors, not an open Blizzard loader specification. They justify narrow
adapters and scoped limitations, not universal runtime claims. The required
[context adapter](EMMYLUA_UPSTREAM.md#addon-chunk-context) remains open work and
must keep unsupported special environments explicitly limited.
