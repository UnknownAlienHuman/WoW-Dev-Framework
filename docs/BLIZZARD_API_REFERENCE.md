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

Reviewed at `1a87ce15` on 2026-10-21: the producer writes
`wow-native-source-build/2`, with top-level `annotation_files` and sidecars.
`tools/xtask/src/library.rs::verify` accepts only `wow-native-source-build/1`
and requires the old embedded `library` object, including generated text.
**Do not append `cargo xtask verify-library` to the command above and call it a
working workflow.** `.github/workflows/current-source-bundle.yml` still does so;
a successful generation reaches an incompatible reader. This is a source-level
finding, not a workflow run performed by this review.

#201 must reconcile that consumer with the supported producer or remove the old
verifier if its remaining checks duplicate the producer. Preserve useful checks
of actual generated files and source locations. Do not fix this by accepting both
versions, restoring the large embedded library, adding a converter, or changing
only the admitted version string: the field layout also differs.

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
