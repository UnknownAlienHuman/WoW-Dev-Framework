# Native documentation input

```text
cargo build --locked -p wow-cli --bin wow
wow status --config workspace/native-project.json --format json --detail capabilities
wow check --config workspace/native-project.json --project my-addon --format json
```

This is the W03 **selected-source functional slice**, not a full Reference Pack
importer or complete WoW profile. `--config` accepts
`wow-service/local-project-native/1` in addition to the existing inline, file and
TOC schemas. For explicit full-report export and the separate reusable cache
input, see [NATIVE_ARTIFACT.md](NATIVE_ARTIFACT.md). The source input does not require a hand-built `ReferenceView`, a full
internal `ProfileIdentity` JSON or a manually assembled annotation Library.

## Configuration

The top-level fields are exactly `schema`, `project_id`, `workspace_id`,
`source_origin_id`, `logical_root`, `profile`, `analyzer`, `main`, `native_source`,
with optional `annotation_inputs` as described below. Unknown or duplicate known fields reject. Logical project/workspace/origin/root
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
`unverified-current`. The explicit-file mode uses source binding
`explicit_digest_pinned_manifest`; the manifested mode below verifies the source
manifest and selected TOC closure without claiming remote Git attestation.

The original `native_source` form is `{ "root": "source", "files": [...] }`. Every entry must have
`path`, `content_digest` (`sha256:` and 64 lowercase hex digits) and `byte_length`.
Select generated API-documentation `.lua` files from the same resolved revision.
Hash and length cover exact raw bytes, including line endings. The reader opens
only these explicitly named files and checks both identities. This mode does not
expand a native TOC; neither mode runs Lua, scans directories or mutates repositories.
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

## Manifested TOC input

Instead of a hand-selected `files` array, `native_source` accepts:

```json
{
  "root": "source",
  "manifest": {
    "path": "source-manifest.json",
    "content_digest": "sha256:<exact-64-lowercase-hex-digest>",
    "byte_length": 12345
  },
  "toc": "Interface/AddOns/Blizzard_APIDocumentationGenerated/Blizzard_APIDocumentationGenerated.toc"
}
```

The digest/length are placeholders to replace with the actual file identity.
`manifest.path` is relative to the configuration directory; `root` identifies the
materialized source directory; `toc` is an exact path inside that source manifest.
Optional `load_context` uses the existing [TOC context](TOC_INPUT.md). It is never
inferred from a build, flavor name or installed game. Exactly one of `files` or
`manifest` + `toc` is allowed; `load_context` is forbidden in the files-only mode.

Use the existing [xtask source manifest](../../tools/xtask/README.md) v1, produced
by `cargo xtask manifest <checkout> <resolved-ref> <selector> <new-manifest.json>`.
The owner admits the entire manifest document and checks its exact selected
extensions, source kind/acquisition policy, schema, self-digest, ordered unique
member index, Git-ID shape, file kinds and all coverage counts/byte totals.
Unknown fields, duplicate decoded field names, nulls, malformed digests and
case-colliding/nonportable paths reject before the source root is opened.

The manifest revision must equal `profile.revision`. Its source version and the
pinned `version.txt` bytes must equal `<profile.client_version>.<profile.client_build>`.
The current Gethe version file uses that explicit dotted form; an unsupported
version format rejects instead of guessing a build. Flavor/edition/environment
remain caller-selected labels, not independently attested client properties.

The existing project TOC parser selects direct Lua entries and retains original
order, source byte spans, metadata issues and explicit condition exclusions.
Every included entry must exist as a `generated_api` member of this exact manifest.
Unknown or unresolved entries, duplicate/case-colliding loads and included XML
reject; they do not become an incomplete corpus reported as complete. The
manifested TOC itself must be inside `Blizzard_APIDocumentationGenerated`.
Required addon dependencies are retained as unresolved load metadata, not opened
or executed. Missing Interface metadata remains an issue; an explicitly present
incompatible Interface rejects through the existing parser.

The manifest, TOC, version file and included Lua bytes are pinned by exact raw
SHA-256 and length. One source-root handle is retained throughout acquisition;
all descendants use the existing no-follow, bounded, cancellation-aware reader.
Unselected manifest members are validated as inventory records but **not read**.
No whole-tree, Git blob-membership or network/currentness proof is inferred from
the caller-supplied manifest. Receipt fields retain `git_membership=not_attested` and
`unconsumed_source_bytes=non_api_source_bytes_not_verified`; use the independent
Git-backed `cargo xtask verify-manifest` operation when stronger repository
verification is required. This does not weaken the narrower generated-API
closure proof below.

The manifest owner compares the complete declared `generated_api` member set with
the exact TOC-selected set. `generated_api_closure=complete_manifest_toc_closure`
is emitted only when both sets and their cardinalities match. The reference owner
then independently requires every selected document to be admitted and rejects
negative authority after any in-domain normalization, payload or record loss.
Environment-excluded systems and ScriptObject methods are outside the selected
global/namespace callable partition and remain explicit omissions. Conflicts stay
key-scoped: the conflicted key is never clean or absent, while an unrelated exact
miss may use complete partition authority.

When all of those conditions hold, `reference.native.apidoc.api` uses
`CoverageStatus::Complete` and the production `wow.api.exists@1` policy may report
an unresolved member as authoritatively absent **within that exact pinned
generated-API corpus and environment**. Explicit-file input, incomplete TOC
closure, any failed document or any in-domain projection loss remains Partial, so
missing records remain `NotEvaluated`. This is API-documentation authority only:
it does not prove currentness, Git membership, runtime availability, signatures,
hotfix state, protected-state behavior or client acceptance.

The source selection identity additionally binds the manifest/TOC/version bytes,
ordered selection and explicit load context. Manifested input now uses
`wow-project/source-manifest-admission/2`, `wow-reference/native-callable-view/2`,
`wow-service/native-input-report/4` and
`wow-service/native-input-receipt/4`; the report/receipt retain declared and
selected generated-API counts, closure state, the final authority decision and
`negative_authority_scope=reference.native.apidoc.api` when that decision is true.
The original explicit-files mode retains its v1 identity/report/receipt profiles.
Production Secret/restriction evaluation remains unavailable.

## Explicit annotation inputs

Both native source modes accept an optional top-level `annotation_inputs`:

```json
{
  "annotation_inputs": {
    "corrections": {
      "path": "reviewed-corrections.json",
      "content_digest": "sha256:<exact-64-lowercase-hex-digest>",
      "byte_length": 1234
    },
    "alias_catalogs": {
      "root": "annotation-resources",
      "revision": "<exact-external-resource-revision>",
      "files": [
        {
          "path": "Annotations/Core/Type/BlizzardType.lua",
          "content_digest": "sha256:<exact-64-lowercase-hex-digest>",
          "byte_length": 2345
        }
      ]
    }
  }
}
```

Replace all example identities and lengths with actual values. Either resource
lane may be omitted; an empty `annotation_inputs`, explicit null at either lane
or the top-level field, empty catalog list and unknown fields reject. Omit the
whole field to keep the original no-resource behavior. No defaults, directory
scan, Git/network lookup or automatic donor-pack selection are introduced.

`corrections.path` is relative to the retained configuration directory. It uses
[the existing guarded correction contract](../../docs/KETHO_NATIVE_CORRECTIONS.md#pack-data),
including supported v1 field/receiver and v2 inheritance records. Its JSON bytes
are pinned and capped at the owner's 2 MiB limit before allocation; canonical
set validation, raw-value guards and application remain in `wow-reference`.
Stale source/normalizer/value guards produce `expired`, a different environment
produces `not_applicable`, and competing or unsupported changes retain the owner's
conflict/rejection outcomes. No expectation is refreshed and no fallback pack is
chosen. Review labels/evidence remain caller-declared, not authenticated review.

`alias_catalogs.root` is relative to the configuration directory. All explicitly
listed resources share one independently selected external revision; it need not
match the Blizzard revision. Exact file digests/lengths, portable paths and case
uniqueness are admitted through one retained root handle. The existing
`ingest_alias_catalog` and `project_with_alias_catalogs` owners handle named and
string aliases plus their supported structure, namespace, function-container and
global-color data profiles. This is not permission to execute arbitrary custom
Lua annotations or admit unsupported forms. Files are sorted canonically, and
alias dependencies/collisions are resolved across the entire admitted set.
Resources cannot replace source declarations or turn unresolved types into `any`.
Original external text, revision, digest and source spans remain in the Library
report, separately scoped from Blizzard source even when relative paths match.
Only explicitly authorized redistributable annotation resources should be supplied.

Catalog limits are 32 files, 256 KiB per resource, 2 MiB total and 4,096 aggregate
admission units under the existing owner counting profile. Per-file/aggregate byte
ceilings are enforced by the confined reader before parsing; declared counts are
checked while admitting resources, before projection. Cancellation, malformed or
unsafe input returns an error rather than silently disabling the selected lane.
Declaration-level unsupported outcomes remain visible while independent entries
can still be emitted. No caller source/resource files are modified.

For explicit-file input, an enabled resource lane selects
`wow-service/native-input-report/3` and
`wow-service/native-input-receipt/3`. Manifested input uses the authority-aware v4
report/receipt regardless of whether resources are present, so its source-closure
contract does not change with annotation configuration. `annotation_inputs` retains
the exact resource selection and compact producer outcome counts by family,
including unresolved structure fields, callback returns and global-color types.
Full correction applications, external source text and maps stay in the bounded
native report, not the compact command receipt.

The canonical correction-set digest participates in ProfileIdentity and hence
Reference generation selection. The ReferenceView still retains **raw source
callable records**, not corrected signature or alias authority; changes apply to
the private normalized annotation projection. The effective analyzer configuration
uses `wow-service/native-analyzer-binding/2`, including exact correction input,
canonical set ID, external revision, canonically ordered resource identities and
actual emitted Library bytes. Changing resources cannot reuse an old analysis
identity even when they happen to emit the same Library text. Moving an unchanged
host root does not change logical input identity.

Neither content pins, external revision labels nor correction review strings
attest Git membership, freshness, consumer correctness or runtime safety. Resource
selection does not create Secret/restriction authority. The native API partition
is Complete only under the manifested closure and loss checks above; every other
mode remains Partial. Production Secret/restriction evaluation remains unavailable.

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
and explicit metadata with the selected canonical correction identity, or the
original versioned no-corrections identity when no set is selected. Reference generation
also includes actual emitted partition/conflict content. The effective analyzer
configuration binds the caller's original configuration, profile, ReferenceView
digest, annotation schema/source-map profile and **actual generated Library file
digests and lengths**. Both original and bound configuration digests are retained.
This prevents changed Library output from retaining an old project identity.

## Evidence and conservative results

The callable Reference slice contains exact global/namespace function candidates
only. Duplicate declarations retain their candidates and an explicit conflict,
not a chosen winner. ScriptObject receiver contracts and environment exclusions
remain explicit out-of-domain omissions. Explicit-file or lossy manifested input
produces a **Partial** partition. Only the exact manifested closure described above
can produce a Complete callable partition and API-documentation absence authority.
The route never creates Secret facets or restriction facts.

`LocalProjectInput::native_source_report()` exposes the bounded raw metadata,
normalization/projection omissions, conflicts, annotation sidecars and source maps
as JSON bytes. `native_input_receipt()` exposes its digest/length and input/output
identities. The backend retains both through the operation. Native check owner
analysis uses `wow-service/owner-analysis/2`, `input_mode=native_source_project`,
and includes the receipt; ordinary status has a partial native annotation component
identified by the report digest. Existing non-native owner output remains version 1.
The CLI does not write or automatically export the full report to disk.

The annotation receipt is not LuaLS/Emmy semantic acceptance. A non-fixture check
uses the real analyzer and the native production policy: exact API records may
establish presence, a Complete manifested callable partition may establish exact
member absence, and partial/conflicted/unsupported cases remain `NotEvaluated`.
The Secret rule remains `NotEvaluated`. There is no runtime or release-gate
promotion.

## Bounds and remaining work

Source selection and generated Library each allow at most 1,024 files, 1 MiB per
file and 16 MiB total. Reference projection additionally limits registrations and
examined functions to 65,536 each, payloads to 64 KiB each and cumulative callable
payload bytes to 32 MiB. The retained native report is streamed under a 64 MiB
ceiling with cooperative cancellation. Existing config/analyzer artifact, Main
inventory and command-output bounds remain unchanged. No sources are modified.

The manifest document allows at most 64 MiB and 200,000 declared tracked files,
32 MiB per inventoried member and 256 MiB inventoried bytes, matching xtask v1.
Consumed TOC/version/Lua bytes still fit the narrower 1 MiB per-file and 16 MiB
aggregate input limits; no large whole-source-tree acquisition is introduced.

Explicit corrections and the existing supported multi-catalog profiles are wired
into this native input. Additional custom-resource forms are not inferred.
[Native artifact import and CLI report export](NATIVE_ARTIFACT.md) use a closed
service-native envelope, not arbitrary driver or Reference Pack output. Complete
real-profile/consumer acceptance remains open W03 work.
Manifest/selected-TOC admission is implemented, not independently executed semantic
or Git provenance acceptance. Managed acquisition is W08;
the W04 native policy covers exact API presence and, only for a Complete manifested
generated-API corpus, exact member absence. Production Secret policy remains
unavailable. No tests or fixtures are changed by this slice.
