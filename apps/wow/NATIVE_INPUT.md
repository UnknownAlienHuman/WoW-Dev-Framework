# Native source input

`wow-service/local-project-native/1` reads a selected Blizzard API-documentation
source, projects the existing Reference and Ketho annotation owners, then sends
the generated Lua Library through the ordinary project/analyzer flow. It is a
current reader format, not the ordinary addon setup configuration tracked by
#200. #200 should resolve source roots, flavor, TOC and Library from actual
inputs; it need not retain this manifest shape or require generated annotations
when a developer already has a Library directory.

## Configuration

The schema requires
`schema`, `project_id`, `workspace_id`, `source_origin_id`, `logical_root`,
`profile`, `analyzer`, `main` and `native_source`. These are strict fields;
unknown fields reject. Project/workspace/origin IDs and logical root have the
meanings described for [materialized input](LOCAL_INPUT.md#input-wow-servicelocal-project-input1).
Physical paths are relative to the selected config directory.

`profile` requires `profile_id`, `flavor`, `interface`, `client_version`,
`client_build`, `revision` and `environment`; `edition` and `reference_target`
are optional. `revision` is a full lowercase 40- or 64-hex Git object ID;
`interface` and `client_build` are positive and `environment` is explicit. These inputs identify the selected source and client flavor; they
do not establish that a remote repository or installed client authenticated it.

`analyzer` uses the existing disk analyzer declaration from
[FILES_INPUT.md](FILES_INPUT.md#file-manifest-schema): a pinned
`compatibility_report`, `accepted_pin_id`, `configuration_digest`, `contract_id`,
`fixture_contract_id` and `library_contract_id`. The report is checked against
the compiled analyzer. These are current reader requirements; #200 should derive
the necessary binding from real inputs and the compiled owner.

`main` selects either a nonempty explicit file inventory or one selected TOC,
with optional `load_context`; see [TOC_INPUT.md](TOC_INPUT.md). Main addon files
remain separate from generated Library files. `native_source` selects exactly
one of:

- Explicit pinned Lua files under `root`. Each file has `path`,
  `content_digest` and `byte_length`.
- A pinned `manifest` file reference, `root` and a `toc` path string, with
  optional `load_context`. `manifest.path` is relative to the config directory;
  `toc` is relative to the source root and selects a manifest member. The
  manifest revision/version must match the profile. `load_context` is not
  allowed in explicit-files mode.

Both modes read source as data and do not scan, execute or modify source trees.
They use bounded, cancellable, no-follow acquisition. Source and generated
Library inventories each allow up to 1,024 files, 1 MiB per file and 16 MiB
total. Manifested TOC input can establish complete API-documentation coverage
only when the manifest's declared generated-API set exactly matches the selected
TOC set and all selected documents are admitted without projection loss.
Explicit-file input and incomplete or lossy inputs remain partial; they do not
establish API absence.

## Optional native inputs

All options below are omitted unless selected. Unsupported values and explicit
nulls reject.

| Field | Accepted input and purpose |
|---|---|
| `annotation_inputs` | Selects at least one of a digest-pinned `corrections` file or explicit `alias_catalogs` (`root`, one exact `revision`, and pinned `files`). Corrections use the [native correction contract](../../docs/KETHO_NATIVE_CORRECTIONS.md); alias parsing stays with the existing Reference annotation owners. |
| `native_api_uses` | `reference_v1`; enables exact native API use relationships for graph publication. See [context build](CONTEXT_BUILD.md). |
| `state_projection` | `dynamic_prefixes_v1`; enables the bounded literal-prefix SavedVariables projection. Dynamic suffixes remain possible observations. |
| `library_witnesses` | `matcher_v1`; retains selected Library matcher witnesses for graph publication. |
| `signal_witnesses` | `matcher_v1`; retains selected event, custom-signal and CVar matcher witnesses for graph publication. |
| `compatibility_evidence` | `parity_report` plus 1–8 `consumer_probe_results`, each a pinned JSON file. The annotation compatibility owner validates the reports; this is optional evidence, not an ordinary check prerequisite. |

Corrections are limited to 2 MiB; catalogs to 32 files, 256 KiB each, 2 MiB
and 4,096 aggregate admission units. Use the [Ketho port map](../../docs/KETHO_RUST_PORT.md)
for supported forms. Corrections affect annotation projection; raw Reference
records retain their source meaning. Malformed selected input fails; unsupported
declarations remain visible while independent valid entries can be emitted.

The regular check consumes the produced Library through existing Main/Library
roles. A second native-library rebuild is unnecessary for this check.
`native_api_uses`/`state_projection` select static facts; witness and
compatibility fields are optional machinery tracked by #195/#118, not setup
requirements to carry into #200.

## Manifested TOC input

This uses the existing [xtask source manifest](../../tools/xtask/README.md)
and Project TOC reader. The TOC must lie in `Blizzard_APIDocumentationGenerated`;
`version.txt` must match `<client_version>.<client_build>`. The manifest, selected TOC,
version file and included Lua files must match their declared digest and length;
included files must be `generated_api` members of that manifest. Only direct
Lua entries are admitted by the current native TOC path. Unsupported, unresolved,
duplicate or XML entries reject instead of being treated as a complete API
corpus. Load context is explicit and never inferred from a flavor, build or
installed game. The exact closure rule above is the only path here that can
support API-documentation absence claims; it says nothing about runtime behavior.

For native artifact import and report export, see
[NATIVE_ARTIFACT.md](NATIVE_ARTIFACT.md). Those are separate reader operations,
not extra requirements for ordinary addon checks.

## Results and owner boundaries

The generated Lua, not raw API-documentation source, enters Library. Main is the
selected addon. `LocalProjectInput::native_source_report()` retains raw metadata,
omissions, conflicts and source maps; the regular check returns a compact receipt.
Annotation output alone does not establish that addon type diagnostics work;
#193/#201 check that integration.

Native WoW checks use exact API presence and, only with complete callable
coverage, member absence. The selected Secret rule uses manifested first-return
metadata and a direct local concatenation; a source-backed `canaccessvalue`
guard must dominate that operation. Unsupported flow/facets remain
`NotEvaluated`. See the [rule owner](../../crates/wow-rules/README.md); these are
static observations, not general runtime safety.

The native report is capped at 64 MiB. Manifest and projection limits remain in
[Project disk acquisition](../../crates/wow-project/DISK_INPUTS.md),
`wow-project/src/disk/source_manifest/mod.rs` and `wow-service/src/local/native_reference.rs`.
Use those existing bounded readers; do not invent another acquisition framework.
