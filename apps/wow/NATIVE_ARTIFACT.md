# Native report export and reusable input artifacts

This is a closed, explicit W03 service-native cache format. It is not an E1
Reference Pack, arbitrary `native_library` driver output, a serialized analyzer
session or proof of source/consumer acceptance. The source-producing route remains
[NATIVE_INPUT.md](NATIVE_INPUT.md); both routes use the existing project/analyzer
owners. No Lua execution, directory scan, network lookup or source write is added.

## Export commands

```text
wow native report --config workspace/native-project.json --project my-addon
wow native artifact --config workspace/native-project.json --project my-addon
```

`report` returns the original full native report JSON bytes, including retained
metadata, omissions, conflicts, correction outcomes and source maps. `artifact`
returns a reusable envelope containing the selected ProfileIdentity, conservative
ReferenceView, generated Library files and those original report bytes.

Both commands accept `--max-bytes N` (default and maximum 67,108,864) and optional
`--expect-sha256 sha256:<64-lowercase-hex>`. `--project` is required and must match
the configured ProjectId. The digest guard is checked against the **entire output**
before stdout is written. No `--format`, implicit project selection, file write
or automatic retry is supported. `wow native --help` prints command help.

Output is exact UTF-8 JSON, with **no added newline**. The report's internal
whitespace and byte identity are preserved. The artifact's Library entries are
sorted by exact path and serialize deterministically; the embedded report remains
unchanged. The service checks the same supported report bindings before export
as before import, instead of emitting an unsupported cache.

A source-configured export performs its existing input acquisition/projection to
obtain the report; it does not run project diagnostics. With a prebuilt input,
export does not rerun the source or annotation producers. The CLI invokes one
service export operation, then writes its returned bytes in bounded chunks.

| Exit | Meaning |
|---|---|
| 0 | Exact output was written and flushed; not clean analysis or acceptance |
| 3 | Service rejected the export, target, digest guard or output budget |
| 4 | Output/flush failure, including a broken pipe |
| 64 | Invalid arguments or configuration/input admission failure |
| 130 | Cancellation; discard any output already emitted |

Retain stdout using an explicitly chosen **new temporary destination** and keep it
only on exit 0. Never redirect onto a config, source, report or existing artifact:
shell redirection can truncate a file before the program starts. Discard a partial
stream after any nonzero exit. Hash the exact saved bytes and record their actual
length before importing; a shell/editor encoding conversion changes the digest.
Source-derived content is data, not agent instructions. The report/cache can
contain external catalog text and detailed provenance; export does not grant
redistribution rights. There is no automatic upload or persistent output file.

## Prebuilt configuration

Use `wow-service/local-project-native-artifact/1` in `--config`. Its fields are
exactly `schema`, `project_id`, `workspace_id`, `source_origin_id`, `logical_root`,
`expected_profile_id`, `artifact`, `analyzer` and `main`. Unknown fields reject;
required typed fields and their duplicate decoded names cannot be omitted/repeated.

Keep the existing project/workspace/origin/root labels and explicit `main` and
`analyzer` blocks from a valid local configuration. Replace `profile`,
`native_source` and optional `annotation_inputs` with these fields:

```json
{
  "schema": "wow-service/local-project-native-artifact/1",
  "expected_profile_id": "profile:wow:my-selected-profile",
  "artifact": {
    "path": "native-input.json",
    "content_digest": "sha256:<actual-64-lowercase-hex>",
    "byte_length": 12345
  }
}
```

This fragment is **not a complete configuration**. Replace the sample profile,
digest and length with the selected artifact's actual values. `artifact.path` is
a portable relative JSON path below the configuration directory. Both content
hash and byte length are mandatory. `expected_profile_id` prevents a wrong-label
selection; the pinned artifact hash binds the whole profile, not just its label.
Fixture profiles, including the E0 fixture label, are rejected.

`analyzer` is the existing [disk analyzer declaration](FILES_INPUT.md#file-manifest-schema),
including a separately supplied pinned compatibility report that must match the
compiled backend. Historical analyzer IDs inside the retained report do not
replace this declaration. `main` is the existing explicit addon `files` or
[selected TOC input](TOC_INPUT.md), acquired and checked for this operation.
Original Blizzard/annotation-resource roots are not required or reopened. Only
the prebuilt artifact, current analyzer report and declared Main inputs are read.

```text
wow status --config workspace/prebuilt-project.json --format json --detail capabilities
wow check --config workspace/prebuilt-project.json --project my-addon --format json
wow native report --config workspace/prebuilt-project.json --project my-addon
```

## Envelope and admission

`wow-service/native-input-artifact/1` contains `schema`, `profile`,
`reference_view`, `library_files`, `source_report` and `negative_authority=false`.
Each Library file has exact `path`, `text`, `sha256`, `byte_length`.
`source_report` contains its original JSON **string** and exact SHA-256. The
consumer decodes no executable state and runs no source text.

The importer uses the existing confined, no-follow, cancellation-aware project
reader. The cache has a separate 64 MiB read ceiling; existing generic JSON limits
are not widened. It then validates the structured profile and ReferenceView with
their owners, report digest, exact Library digests/lengths, portable/case-unique
paths and the envelope/report bindings. Library limits remain 1,024 files,
1 MiB/file and 16 MiB total. The closed native cache uses flat renderer-owned
`.lua` paths, not arbitrary nested Library layouts.

Supported original reports are `wow-service/native-input-report/1`, `/2`, `/3`,
containing native annotation Library `/3`, `/4`, `/5` and
`wow-native-field-maps/1`. Core profile, reference and Library bytes must agree
with the corresponding report fields; report source mode and optional manifested/
annotation-resource fields must agree with the report version. Unsupported
schemas, malformed data or default JSON recursion-limit violations reject.
Export escaping/envelope overhead counts toward the limit: not every 64 MiB
report fits inside a 64 MiB cache. A too-large cache fails without truncation.

Detailed producer metadata, correction applications and source maps are retained
as **unverified historical producer claims**, not semantically revalidated.
The importer checks critical typed identities and file bindings; it does not
interpret every opaque sidecar, prove Git membership, reopen original sources or
certify external review. Content hashes provide integrity, not authentication.
A Wasm Library `/6`, arbitrary driver `source-report.json` or E1 pack is not this
cache format and is rejected, not silently converted.

## Current identity and capability reporting

The new analyzer configuration binds the caller's original configuration, exact
artifact SHA-256, full profile and ReferenceView digest under
`wow-service/native-artifact-analyzer-binding/1`. Artifact changes cannot reuse an
old analysis binding. Input host location is not a logical identity. Main inputs
still participate in the normal project generation identity.

`LocalProjectInput::native_artifact_receipt()` returns the current
`wow-service/native-artifact-receipt/1` receipt with the cache/report identities,
Library counts/bytes and original/current analyzer configuration digests. It
explicitly states `source_verification=not_reacquired`,
`producer_verification=not_reexecuted`,
`source_maps_verification=retained_not_revalidated` and
`semantic_consumer_acceptance=not_evaluated`.

The backend retains the report and receipt. Status reports the partial capability
`annotations.retained_native_artifact`; check uses `wow-service/owner-analysis/3`,
`input_mode=prebuilt_native_artifact_project` and the current artifact receipt.
Source-producing inputs keep their existing report/receipt and analysis shapes.
`native_source_report()` exposes the original producer bytes for either route;
those bytes intentionally retain historical analyzer metadata, not current state.

Reference admission permits only the existing single Partial native API callable
partition, no restriction facets and no authoritative absence. Non-fixture WoW
rules still remain NotEvaluated pending W04/#72. Export/import is not full W03,
E0/E1, consumer, runtime or launch acceptance.
