# Repository map

Use one task route, then read its owner contract and actual caller. This is a map
of existing entry points, not a second architecture, status register or file
inventory. [Current status](CURRENT_STATUS.md) owns delivery; the
[queue](REFACTOR_QUEUE.md) owns order; [CONTRIBUTING.md](../CONTRIBUTING.md) owns
checks and update destinations.

## Ordinary check

The current CLI still reaches the legacy local backend. The lower direct checker
exists separately; the public source-preparation-to-result operation remains open.
The arrows below identify handoffs to inspect, not an already connected workflow.

| Task | Existing producer -> consumer | Read / review focus |
| --- | --- | --- |
| TOC metadata (T23) | [TOC parser](../crates/wow-project/src/load/toc.rs), [metadata](../crates/wow-project/src/load/metadata.rs), [package admission](../crates/wow-project/src/load/package.rs) -> [Service limitation mapping](../crates/wow-service/src/addon_check/project_limitations.rs) | [Metadata by requested scope](IMPLEMENTATION.md#requested-scope-limitations). Inspect the existing draft referenced by status before duplicating its classifier. |
| Physical Main and target (T32/T33/T35) | [Loader and `ProjectLoadPlan`](../crates/wow-project/src/load/mod.rs), [inventory reader](../crates/wow-project/src/disk.rs) -> [lower check inputs](../crates/wow-service/src/addon_check.rs) | [Slice 1A](IMPLEMENTATION.md#first-implementation-slice). Change the retained plan and both empty-Main gates, not only the caller signature. Full XML expansion is later work. |
| Safe roots, Libraries and errors (T34/T36/T38) | [Disk admission](../crates/wow-project/src/disk.rs), [Library reader](../crates/wow-project/src/disk/library.rs) -> [configured owner](../crates/wow-emmy/src/configured_session.rs) -> [Service](../crates/wow-service/src/addon_check.rs) | [Authority](IMPLEMENTATION.md#safe-roots-and-selected-path-authority), [structured failures](IMPLEMENTATION.md#structured-failures). Follow role/path/cause through every conversion. |
| Diagnostic configuration/reuse (T13/T16) | [Configuration](../crates/wow-emmy/src/configuration.rs) -> [configured session](../crates/wow-emmy/src/configured_session.rs) -> [diagnostic collector](../crates/wow-emmy/src/syntax/analyzer.rs) | [Configuration semantics](EMMYLUA_UPSTREAM.md#diagnostic-configuration). Check admission, equivalent settings, explicit global-off and resource-refusal behavior together. |
| Chunk requirements (T31) | [Configured owner](../crates/wow-emmy/src/configured_session.rs) -> [lower completeness assembly](../crates/wow-service/src/addon_check.rs) | [Same-owner AST collection](EMMYLUA_UPSTREAM.md#addon-chunk-context). Locate affected Main chunks without another parser; helper injection is later work. |
| Public Service assembly (T37/T38) | [Addon-check module](../crates/wow-service/src/addon_check.rs), [limitation mapper](../crates/wow-service/src/addon_check/project_limitations.rs), [public exports](../crates/wow-service/src/lib.rs) -> [CLI dispatch](../apps/wow/src/main.rs) | [One public operation](IMPLEMENTATION.md#one-public-service-operation). Current package mapping is not the standalone requested-scope mapper; do not copy its global completion flags. |
| CLI/config/output (T01/T02/T39) | [Args](../apps/wow/src/args.rs), [dispatch](../apps/wow/src/main.rs), [StrictJson](../apps/wow/src/json.rs) -> Service -> [output](../apps/wow/src/output.rs) | [Frontend contract](FRONTENDS.md#configuration-and-check-input). Existing [local response](../crates/wow-service/src/local/response.rs) is a replacement boundary, not the ordinary route to extend. Package is `wow-cli`; binary is `wow`. |

The [trace register](IMPLEMENTATION_TRACE_REGISTER.md#source-owners-behind-the-open-boundaries)
provides detailed evidence for a disputed boundary. Read that row, not the entire
audit history. The [owner table](IMPLEMENTATION.md#code-ownership) remains the
responsibility contract.

## Other task routes

| Task | Start here, then follow the consuming owner | Contract / boundary |
| --- | --- | --- |
| Blizzard/Ketho Library generation | [Builder](../apps/wow-reference-builder/src/lib.rs), [Git input/output](../apps/wow-reference-builder/src/io.rs), [catalogs](../apps/wow-reference-builder/src/catalogs.rs) -> [raw ingestion](../crates/wow-reference/src/native.rs), [typed facts](../crates/wow-reference/src/native_model.rs) -> [native emission](../crates/wow-annotations/src/native.rs), [Ketho rendering](../crates/wow-annotations/src/ketho.rs) | [Projection guide](KETHO_RUST_PORT.md), [current command](../apps/wow-reference-builder/README.md). Trace report/maps into the reader; successful writing alone does not close T10. |
| LSP/MCP and unsaved source | [LSP wire](../apps/wow/src/lsp.rs), [MCP wire](../apps/wow/src/mcp.rs) -> [Service frontends](../crates/wow-service/src/frontend/) -> [overlay model](../crates/wow-project/src/overlay.rs), [state](../crates/wow-project/src/overlay/state.rs) | [Protocols](FRONTENDS.md#lsp-adapter), [queued adapter work](REFACTOR_QUEUE.md#3-thin-lspmcp-adapters). These are transitional owners, not a second ordinary checker. |
| Source index/search/context | [Store](../crates/wow-store/), [Search](../crates/wow-search/), [Context](../crates/wow-context/) and their callers in [Service](../crates/wow-service/src/) | [Direct lookup slice](REFACTOR_QUEUE.md#4-direct-source-index-search-and-context). Useful existing code is mixed with legacy replay/federation; do not make it a prerequisite for checking. |
| Shared primitives or legacy removal | [Workspace members](../Cargo.toml), [Core](../crates/wow-core/), [Rules](../crates/wow-rules/), [Graph](../crates/wow-graph/), [Recognizers](../crates/wow-recognizers/) | [Removal boundary](REFACTOR_QUEUE.md#5-remove-displaced-platform-code). A directory name is not permission to delete its still-used behavior or preserve its platform wrappers. |
| Repository checks, instructions or CI | [xtask dispatch](../tools/xtask/src/main.rs) -> [file policy](../tools/xtask/src/repository.rs), [Markdown](../tools/xtask/src/markdown.rs), [skill sync](../tools/xtask/src/skill.rs); [Quick workflow](../.github/workflows/quick-check.yml) | [Maintenance commands](../tools/xtask/README.md), [automation](../.github/README.md). Maintenance is not an alternate product implementation. |

## Read less, verify the right revision

Locate a symbol in its owner, then inspect its callers and error/result conversion:

```sh
git grep -n -F 'ConfiguredAnalyzerSession' -- crates/wow-emmy/src crates/wow-service/src
git grep -n -F 'check_addon' -- crates apps
git grep -n -F 'docs/FRONTENDS.md' -- '*.md'
```

Search results are entry points, not proof of behavior or complete caller coverage.
Read the containing function and the next producer/consumer. Existing tests can
illustrate historical cases but do not override current scope or authorize test
execution. API users must fetch search hits at the selected PR/commit, since code
search covers the default branch.

When moving an entry point, update this row and inbound links in the same change.
Check links with xtask and symbol names against source; link validation cannot
verify a Rust symbol or a statement about its behavior. Keep numeric limits and
protocol rules in their owning guide rather than duplicating them here.
