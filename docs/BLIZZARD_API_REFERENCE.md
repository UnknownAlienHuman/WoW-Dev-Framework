# Native Blizzard API source input

The supported producer is `apps/wow-reference-builder`; its source selection,
exit codes and limitations are in the [Ketho guide](KETHO_RUST_PORT.md#native-source-to-library-flow).
It reads one resolved local Git revision and generated-API TOC, evaluates the
restricted EmmyLua AST input without executing Lua, and writes ordinary `.lua`
files plus `source-report.json` into a new output directory.

```sh
cargo run --locked -p wow-reference-builder -- generate \
  /path/to/wow-ui-source HEAD \
  Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc \
  Mainline /path/to/new-output
```

## Current handoff gap

The producer writes `wow-native-source-build/2`, with top-level
`annotation_files` and sidecars. `tools/xtask/src/library.rs::verify` still accepts
only `/1` and requires the old embedded `library`, including generated text.
Do not append that verifier to this command or restore its old payload.

The current-source workflow no longer invokes it. Generation exits 0/3 are
accepted with partial-projection warnings; a direct `input_failures == []` check
still rejects omitted source inputs. On success, `native-lua-libraries-<sha>`
contains both `native-annotations/` and `native-aliased/`, including their Lua
files and reports. The separate report artifact is retained even on failure.
This is artifact delivery, not proof that the addon checker loaded the Library.

#201 still must remove the old verifier/test producer after moving useful
source-map/type checks to their actual owners. No report converter, `/1`
compatibility reader or regenerated embedded text is needed.

Generation is not addon-check acceptance. #200 still supplies ordinary root,
flavor, TOC and Library-directory configuration; the existing disk reader requires
explicit file lists and separate profile/reference/analyzer inputs. #201 connects
the emitted files to that actual consumer. The [consumer probes](ANNOTATION_CONSUMER_PROBES.md)
currently exercise another driver and do not establish this public handoff.

Other Reference importers remain separate input/model routes, not readers of the
new source report. Trace their real consumers before removing them; do not treat
their existence as a requirement to retain old generated reports. Partial output
preserves issues and unsupported data; neither a report nor an empty result proves
API absence, full type closure or runtime availability.
