# `wow-reference`

`wow-reference` admits and normalizes selected Blizzard API documentation and
related source data. It owns raw and typed API/type/event/restriction facts,
source-bound corrections, coverage, and read-only views consumed by annotation,
search, and project operations. Source-sensitive work selects one exact
revision and client flavor. Inputs are parsed through existing bounded,
nonexecuting owners; source hooks and generated code never run.

The current Ketho annotation route is `wow-reference` normalization followed by
`wow-annotations` projection. The native library driver and Service/builder
handoff are described in the [Ketho Rust port map](../../docs/KETHO_RUST_PORT.md).
This crate does not own annotation rendering, project/addon indexing, search
ranking, editor settings, or general Lua language-server behavior.

## Useful native APIs

- The native APIDocumentation loader and model live in `src/native.rs`,
  `src/native_model.rs`, and `src/native_corrections.rs`.
- `src/native_aliases.rs` admits explicit annotation-only alias resources as
  bounded `AliasDocument` inputs. These overlays do not create Blizzard facts
  or negative authority.
- `src/native/strings.rs` extends EmmyLua's token decoding for selected Lua 5.1
  decimal and long-bracket string forms while preserving original source spans.
- `NativeViewProjection::callable_signature(record, stop)` provides a bound
  read of source-normalized callable parameters and returns for Search consumers.
- `project_callables_with_type_resolution` attaches the explicit exact-name and
  correction overlay; the original `project_callables` behavior remains its own
  route.

Unsupported fields, conflicting declarations, unresolved types and partial
coverage stay visible. Never coerce unknown input into a successful empty or
`any` declaration. Keep reference facts bound to the selected profile and
generation; static source does not establish runtime, Secret, combat, or taint
behavior.

## Reading by task

Begin with [`AGENTS.md`](AGENTS.md) and the selected owner contract/code. For
annotation semantics and source-to-library flow, use the [Ketho map](../../docs/KETHO_RUST_PORT.md).
For callable type overlays, see [native callable type resolution](NATIVE_CALLABLE_TYPE_RESOLUTION.md).
For alias input behavior, see the [catalog scope](../wow-annotations/KETHO_TYPE_CATALOG.md).
For current Rust string handling, see `tests/native_string_forms.rs` and the
focused implementation in `src/native/strings.rs`. The [architecture decision](../../docs/ARCHITECTURE.md)
states the current project scope.

Run a focused owner-chain check for the changed behavior. Use representative
positive and negative cases when semantics change; full-source campaigns are
for a task that claims release-wide coverage, not every refactor.
