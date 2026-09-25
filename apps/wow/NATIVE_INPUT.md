# Native documentation input

```text
cargo build --locked -p wow-cli --bin wow
wow status --config workspace/native-project.json --format json --detail capabilities
wow check --config workspace/native-project.json --project my-addon --format json
```

This is the W03 **selected-source functional slice**, not a full Reference Pack
importer or complete WoW profile. `--config` accepts
`wow-service/local-project-native/1` in addition to the existing inline, file and
TOC schemas. The new input does not require a hand-built `ReferenceView`, a full
internal `ProfileIdentity` JSON or a manually assembled annotation Library.

## Configuration

The top-level fields are exactly `schema`, `project_id`, `workspace_id`,
`source_origin_id`, `logical_root`, `profile`, `analyzer`, `main`, `native_source`.
Unknown or duplicate known fields reject. Logical project/workspace/origin/root
identities retain their [existing meanings](LOCAL_INPUT.md#input-wow-servicelocal-project-input1).
Physical paths remain relative to the explicitly selected config directory.

`profile` contains:

| Field | Required value |
|---|---|
| `profile_id` | Valid `profile:wow:<slug>` label for this selection, never the E0 fixture label |
| `flavor` | Explicit core flavor segment, such as `retail` |
| `edition` | Optional core edition segment |
| `interface` | Positive Interface value for the selected source, not an implicit current default |
| `client_version` | Exact semantic version string for this selected source |
| `client_build` | Positive client build for this selected source |
| `revision` | Full lowercase 40- or 64-hex Git object ID; moving selectors reject |
| `environment` | Explicit native environment, such as `Mainline`; it is not inferred from Interface |

The caller supplies these labels from its source manifest. They are **not** proof
that a Git server authenticated the supplied files, that the metadata matches a
client installation, or that the selection is current. Receipt freshness remains
`unverified-current`; source binding is `explicit_digest_pinned_manifest`.

`native_source` is `{ "root": "source", "files": [...] }`. Every entry must have
`path`, `content_digest` (`sha256:` and 64 lowercase hex digits) and `byte_length`.
Select generated API-documentation `.lua` files from the same resolved revision.
Hash and length cover exact raw bytes, including line endings. The reader opens
only these explicitly named files, checks both identities and does not run Lua,
walk directories, clone/update repositories or expand a TOC for the native corpus.
Case-fold collisions, unsafe paths, nonregular files and changed bytes reject.

`analyzer` is the existing [disk analyzer declaration](FILES_INPUT.md#file-manifest-schema):
`compatibility_report` (a pinned JSON file), `accepted_pin_id`,
`configuration_digest`, `contract_id`, `fixture_contract_id`, `library_contract_id`.
The original compatibility report must still match the actual compiled analyzer.
This route neither generates an acceptance pin nor bypasses owner validation.

`main` is the existing explicit addon inventory:
`{ "root": "addon", "files": [{ "path": "Init.lua" }] }`, or the existing
[selected TOC form](TOC_INPUT.md) with `toc` and optional `load_context` instead of
`files`. Exactly one form is allowed. No caller-provided `library` or
`reference_view` field is accepted in the native schema.

## Owner composition and identities

`wow-project` captures the pinned source bytes through its confined read-only
port. `wow-reference::native` parses them as data, then its native callable-view
projection creates the Reference partition. The existing Ketho Rust annotation
projector creates the generated Library; neither raw API documentation nor the
whole Blizzard UI implementation enters the analyzer Main or Library universe.
Main remains only the declared addon files/units. The existing project/analyzer
assembly is reused; no second parser, interpreter or semantic session is added.

The source logical digest includes the revision, environment and sorted exact
source-file identities. The Release-class profile is built from that selection
and explicit metadata with a versioned no-corrections identity. Reference generation
also includes actual emitted partition/conflict content. The effective analyzer
configuration binds the caller's original configuration, profile, ReferenceView
digest, annotation schema/source-map profile and **actual generated Library file
digests and lengths**. Both original and bound configuration digests are retained.
This prevents changed Library output from retaining an old project identity.

## Evidence and conservative results

The callable Reference slice contains exact global/namespace function candidates
only. Duplicate declarations retain their candidates and an explicit conflict,
not a chosen winner. ScriptObject receiver contracts, unsupported normalizations
and environment exclusions remain explicit omissions. Every Reference partition
is **Partial**, including a successfully parsed selected corpus. The route never
creates Secret facets, restriction facts or authoritative absence from partial data.

`LocalProjectInput::native_source_report()` exposes the bounded raw metadata,
normalization/projection omissions, conflicts, annotation sidecars and source maps
as JSON bytes. `native_input_receipt()` exposes its digest/length and input/output
identities. The backend retains both through the operation. Native check owner
analysis uses `wow-service/owner-analysis/2`, `input_mode=native_source_project`,
and includes the receipt; ordinary status has a partial native annotation component
identified by the report digest. Existing non-native owner output remains version 1.
The CLI does not write or automatically export the full report to disk.

The annotation receipt is not LuaLS/Emmy semantic acceptance. **WoW rule dispatch
remains fixture-only pending W04 / PR #72.** A non-fixture check uses the real
analyzer with its generated Library, but reports those WoW rules as NotEvaluated;
partial is not a clean result. There is no runtime or release-gate promotion.

## Bounds and remaining work

Source selection and generated Library each allow at most 1,024 files, 1 MiB per
file and 16 MiB total. Reference projection additionally limits registrations and
examined functions to 65,536 each, payloads to 64 KiB each and cumulative callable
payload bytes to 32 MiB. The retained native report is streamed under a 64 MiB
ceiling with cooperative cancellation. Existing config/analyzer artifact, Main
inventory and command-output bounds remain unchanged. No sources are modified.

Full generated-TOC/source-manifest provenance admission, corrections/alias/custom
catalog selection, prebuilt native artifact import, CLI report export and complete
real-profile/consumer acceptance remain open W03 work. Managed acquisition is W08;
production rule policies are W04. No tests or fixtures are changed by this slice.
