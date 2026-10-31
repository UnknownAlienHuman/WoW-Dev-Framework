# Donors and source boundaries

Choose the source used by the assigned task, not a tour of every donor. These are
reviewed baselines, not claims about the latest upstream revision or delivered
product support. Read the selected code before reusing its behavior; the
[repository map](REPOSITORY_MAP.md) locates the existing Rust consumer.

## Pinned sources and useful surfaces

| Source | Reviewed revision | Use / owning contract |
| --- | --- | --- |
| EmmyLua Analyzer Rust | [`aaaca68425d9362876228649b0b8d92f07654daa`](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/tree/aaaca68425d9362876228649b0b8d92f07654daa), compiled dependency | Public parser, semantic model, settings and diagnostics through [wow-emmy](EMMYLUA_UPSTREAM.md). No second Lua engine or private language-server handlers. |
| Ketho vscode-wow-api | [`d0b5b51fac4c52c493371b9b18e66ce604ea4326`](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326), behavior/data comparison | Annotation semantics and explicit supplemental declarations through [the native projection](KETHO_RUST_PORT.md). Not a runtime dependency or promise of output parity. |
| Blizzard UI source mirror | [`e7aa82019dc298e0351c7159d60c2a9b916b9b67`](https://github.com/Gethe/wow-ui-source/tree/e7aa82019dc298e0351c7159d60c2a9b916b9b67), reviewed build 12.1.5 / 70529 | Selected API tables and authored TOC/XML/Lua establish revision-bound [game facts](WOW_LINTER.md). The build is a baseline, not a permanent supported target. |
| Wowless | [`c09acebd9be7d52e41154dff5276f4d62f3e4970`](https://github.com/wowless/wowless/tree/c09acebd9be7d52e41154dff5276f4d62f3e4970), comparative implementation/data | Compare bounded load/XML/object behavior; never substitute emulator behavior for Blizzard authority. See [Wowless](#wowless). |

For Ketho, inspect only the relevant surface:
[emitter](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/luasrc/annotate/init.lua),
[source loader](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/init.lua),
[corrections](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/patches.lua),
[widget mappings](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/wowdoc/loader/doc_widgets.lua),
[type catalogs](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326/Annotations/Core/Type)
or [widget declarations](https://github.com/Ketho/vscode-wow-api/tree/d0b5b51fac4c52c493371b9b18e66ce604ea4326/Annotations/Core/Widget).
Reuse the existing Reference/Annotations pipeline; donor Lua is data to inspect,
not code to execute. Projection rules, known losses and supported Emmy syntax
have one home in [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md).

## Existing primitives before new code

| Need | Existing implementation | Reuse boundary |
| --- | --- | --- |
| Safe files and directories | `cap-std` / `cap-fs-ext` in [Project disk](../crates/wow-project/src/disk.rs) and [Library traversal](../crates/wow-project/src/disk/library.rs) | Expose the narrow Project entry points specified by [source admission](IMPLEMENTATION.md#safe-roots-and-selected-path-authority). Reuse component-wise no-follow acquisition, not the ambient root opener or legacy artifact prerequisites. |
| Strict JSON input | Serde-based [StrictJson](../apps/wow/src/json.rs) | Duplicate-key/depth admission already exists. It does not establish config byte bounds, permitted keys or diagnostic semantics; use [the config contract](FRONTENDS.md#configuration-and-path-bases). Do not write another JSON parser. |
| Bounded result encoding | Append-bounded buffer in [local response](../crates/wow-service/src/local/response.rs) | Reuse the buffer algorithm, not its legacy envelope/error mapping. App `json::encode` counts then serializes a `Value`; [ordinary output](FRONTENDS.md#check-results-and-command-status) needs a single bounded typed encoding. |
| XML | [Project XML reader](../crates/wow-project/src/load/xml.rs), `quick-xml 0.38.4` in [Cargo.lock](../Cargo.lock); [versioned docs](https://docs.rs/crate/quick-xml/0.38.4) | Extend existing source-located parsing and maps; never execute callbacks or build another XML parser. |
| SQLite / FTS | [Store manifest](../crates/wow-store/Cargo.toml), [rusqlite 0.37 docs](https://docs.rs/rusqlite/0.37.0/rusqlite/), [SQLite FTS5](https://www.sqlite.org/fts5.html) | Later [direct lookup](REFACTOR_QUEUE.md#4-direct-source-index-search-and-context), not a checking prerequisite. Existing `serialize`/`backup` features do not prescribe the new storage design. |

This table identifies code to reuse, not new wrappers to create. Extract a shared
primitive only when real callers need it; keep product orchestration out of the
helper. Package versions come from manifests/lock, not this navigation table.

LuaLS is an external interoperability target, not another engine for the checker:
[library configuration](https://luals.github.io/wiki/libraries/) and
[annotation compatibility](https://luals.github.io/wiki/annotations/).
A `.lua` extension does not establish consumer compatibility; the
[projection guide](KETHO_RUST_PORT.md#what-the-pinned-emmy-consumer-can-represent)
records syntax supported by the pinned Emmy source, separately from executed runs.

## Updating a reviewed revision

A checkout update or generation run does not update the compiled engine or prove
compatibility. Use [the guarded source updater](../tools/xtask/README.md#updating-an-explicitly-selected-source-checkout)
only for the explicitly selected checkout; do not advance every donor for an
unrelated implementation change.

- **EmmyLua:** align both pins in [Cargo.toml](../Cargo.toml), [Cargo.lock](../Cargo.lock)
  and revision/tree/crate-version constants in [syntax.rs](../crates/wow-emmy/src/syntax.rs).
  Review the used parser/config/diagnostic APIs with the adapter. While the
  [compiled probe](../crates/wow-emmy/src/compatibility/compiled_probe.rs) remains a
  caller, its measured source digest must match too; the probe is not an ordinary
  check prerequisite.
- **Blizzard/Ketho/Wowless:** inspect changed source needed by the task, its typed
  projection and actual reader. Update this baseline and the affected owner guide
  only for facts rechecked. Keep exact revisions in generated reports; a moving
  branch name is not the revision used.
- **Dependencies and evidence:** inspect the relevant manifest/lock diff, use the
  [affected-caller checks](../CONTRIBUTING.md#choose-the-checks), and distinguish
  source review from an observed run. Preserve historical citations and notices;
  updating a baseline does not change where previously copied code came from.

## WoW source facts and limits

Selected Blizzard source outranks a comparative donor for client facts. API
`Environment` values such as `All`/`SecureOnly` are contexts, not flavors; `All`
does not assert cross-flavor availability. `SecretArguments` is a function-level
policy string, not a list of parameter positions. Required interpretation lives
in [WoW semantics](WOW_LINTER.md); typed/emission gaps live in
[missing projection semantics](KETHO_RUST_PORT.md#missing-semantics-the-linter-needs).

For chunk context, the pinned
[AddOns API documentation](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_APIDocumentationGenerated/AddOnsDocumentation.lua)
and [AllowAddOnTableAccess example](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_CombatAudioAlerts/Blizzard_CombatAudioAlerts.toc)
support the distinction between ordinary chunk arguments and later table retrieval.
Follow [the game contract](WOW_LINTER.md#2-reuse-lua-analysis-model-the-wow-environment)
and [the same-owner adapter](EMMYLUA_UPSTREAM.md#addon-chunk-context), not a second
implementation described here. Static evidence never certifies runtime safety.

## Wowless

Use [UIObject ancestry/script lookup](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/uiobjecttypes.lua),
[object data](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/data/products/wow/uiobjects.yaml),
[XML evaluation](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/xmleval.lua),
[loader](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/modules/loader.lua)
and [TOC parser](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/wowless/toc.lua)
as comparisons only. Do not import their interpreter or execute addons.

Bootstrap/private-table behavior, product gates, `LoadWith`, bindings and handler
arguments follow the reviewed/unsupported distinctions in
[the load model](WOW_LINTER.md#4-toc-and-xml-form-the-load-model).
The [Blizzard SuppressLocalTableRef occurrence](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_RecruitAFriend/Blizzard_RecruitAFriend.toc)
proves the metadata exists, not that Wowless's `nil` convention describes the
current client. Preserve that uncertainty rather than making a guessed adapter.

## Notices

Review the pinned [EmmyLua license](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/LICENSE),
[Ketho license](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/LICENSE)
and [Wowless license](https://github.com/wowless/wowless/blob/c09acebd9be7d52e41154dff5276f4d62f3e4970/LICENSE)
before copying code/data; Wowless's notice excludes separately licensed vendor
and elune components. A mirror or donor code license does not settle permission
to redistribute Blizzard-derived data. Preserve the local
[annotation](../crates/wow-annotations/THIRD_PARTY_NOTICES.md),
[literal](../crates/wow-ketho-literals/THIRD_PARTY_NOTICES.md) and
[reference](../crates/wow-reference/THIRD_PARTY_NOTICES.md) notices.
