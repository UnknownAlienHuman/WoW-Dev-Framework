# Native Ketho corrections

The reference-owned implementation in
[`native_corrections.rs`](../crates/wow-reference/src/native_corrections.rs) ports
Ketho's `wowdoc/loader/patches.lua:ApplyPatch` name-addressed field selection and
`doc_widgets.lua` receiver mapping. It applies the additional guards required by
[the E1 correction contract](../crates/wow-reference/e1/CORRECTIONS.md).
It is connected to the existing Rust source-to-library driver, not a second
source producer or an external-interpreter wrapper.

## Run

```sh
cargo run -p wow-annotations --example native_library -- \
  /path/to/wow-ui-source <resolved-ref> \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output --corrections /path/to/reviewed-corrections.json
cargo xtask verify-library /path/to/new-output --require-input-complete
```

Without the optional flag, no pack is discovered and the existing v3 report is
unchanged. With a pack, the library emits v4 including the canonical correction
set/digest and one outcome per record. The source-build envelope remains v1.
Expired, rejected or conflicted corrections make the projection and driver result
partial (exit 3), even when the ordinary projection-issue list is empty.
Malformed packs fail before creating the output directory. Artifact verification
checks set identity and outcome/status consistency, not external review authority.

## Supported operations

- `callable_field`: select the exact registration, function, Arguments/Returns
  lane and member **by name**, then replace `type` or `nilable`.
- `widget_owner`: replace the normalized ScriptObject receiver name, as in the
  donor's system-name-to-widget-name mapping. It cannot turn a namespace/global
  into a widget or merge distinct source receivers.

`Type` changes on a field with `InnerType` reject: compound-shape corrections need
a separately implemented operation, not an ignored or contradictory replacement.
Type unions have 1–16 unique named/primitive alternatives, no arbitrary syntax.
Aliases lower per branch; arrays of unions render `(T|U)[]` rather than `T|U[]`.
Unknown named types are still explicit unresolved type names under the existing
emitter profile; this feature does not establish named-type closure.

## Pack data

`CorrectionSet` is strict JSON (`wow-native-corrections/1`) with these fields:

| Field | Meaning |
|---|---|
| `version` | Positive reviewed set version. |
| `revision` | Exact materialized Blizzard revision for this pack. |
| `environment` | Explicit environment, not inferred from a build number. |
| `normalizer` | `native-model/1` for this projection contract. |
| `records` | Independent correction records; canonicalized by target and ID. |

Every record contains `id`, `target`, `expected_source_sha256`,
`expected_raw_sha256`, `before`, `after`, `reviewer`, `rationale` and a nonempty
`evidence` list. Each evidence entry has public `revision`, relative `path` and
`sha256`; credentials, private locators and private prose must not be supplied.
Review/evidence strings record caller authorization; they are not authenticated
human-review attestations or automatically accepted platform truth.

`target` has an exact source-relative `path`, zero-based `registration`, and one
of the following `projection` objects:

```json
{"kind":"callable_field","function":"SetCVar","lane":"arguments","member":"value","property":"type"}
```

```json
{"kind":"widget_owner"}
```

Values are tagged data, for example `{"kind":"text","value":"string|number"}`,
`{"kind":"boolean","value":true}` or `{"kind":"absent"}`. `absent` is allowed
only as the old nilability state. The donor's string `"true"` patch is not a
Boolean in this contract; a reviewed Boolean replacement must be explicit.

Inspect the normalized target from the validated source document to obtain the
old value and raw observation. `raw_digest(raw)` hashes canonical serialized raw
value **including field order and spans**. `expected_source_sha256` is the file's
validated content hash. For a widget target the raw observation is the whole
system; for a field target it is the field descriptor. Never replace expectations
automatically when new source fails the old guards. No default correction file,
compiled widget inventory or permanently supported Blizzard build is introduced.

The executable schema examples and source-to-renderer checks are in
[`tests/corrections.rs`](../crates/wow-annotations/tests/corrections.rs); Git-driver
regressions also exercise JSON input and final v4 output.

## Guards and outcomes

The reference owner normalizes original validated documents itself; it does not
accept caller-forged normalized target copies. Raw observations remain unchanged.
Every edit is planned against the same unmodified input before matching edits
are applied to the private normalized copy.

- `applied`: exact source/revision/raw value/before/normalizer match.
- `expired`: source, normalizer, value or target changed; original retained.
- `rejected`: target projection or compound shape is unsupported.
- `conflict`: multiple records or source targets compete, or widget names collide.
- `not_applicable`: explicitly different environment; original retained.

Duplicate corrections conflict even with equal replacements. Existing duplicate
source owners cannot be disguised by renaming one. Collision checks include
unmodified receivers, classes, global functions and namespaces in the selected
environment. Reverting a rejected alias triggers rechecking, so chained renames
cannot leave a collision behind. File basenames and input order never select a
winner. Correction set version, review and content participate in its digest.

## Limits and remaining scope

Packs are bounded to 2 MiB/4096 records; names, types, evidence and rationale are
bounded independently. Matching/collision work has explicit budgets and checks
cancellation; cancelled planning never returns a corrected corpus. Source data
remains bounded by the native loader. No Lua, donor code, shell, network, editor
configuration or runtime-global lookup is executed by the correction library.

This is the independent Type/Nilable/widget-receiver subset, not every operation
of E1-B. Dependency-ordered patches, restriction/deprecation corrections,
persistent correction-store/generation publication, automatic donor-pack import,
complete TypeDocumentation/custom annotations and widget inheritance remain
unimplemented. Receiver naming is not class/inheritance/type closure. The current
emitter still requires real EmmyLua/LuaLS semantic probes; parser acceptance and
unit/golden tests are not substitutes.
