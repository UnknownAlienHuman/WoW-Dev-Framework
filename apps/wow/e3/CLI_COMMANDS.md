# E3-C context CLI commands

**Status:** normative command grammar. The implemented `context validate` parser uses the exact private grammar below.

## Common selectors

Primary publication is required for map/inspect/build and optional where artifact-only validation does not require owner closure:

```text
--project-store <ProjectStoreId>
--publication current
--publication store-generation:<ProjectStoreGenerationId>
--publication publication-set:<ProjectPublicationSetId>
--expect-current-record <CurrentPublicationRecordId>       optional with current
--expect-current-digest <ContentDigest>                    optional with current
```

Optional Blizzard UI source:

```text
--platform omitted

or

--platform-store <ProjectStoreId>
--platform-publication current|store-generation:<ID>|publication-set:<ID>
--expect-platform-current-record <CurrentPublicationRecordId>
--expect-platform-current-digest <ContentDigest>
```

Default is `--platform omitted` unless the exact service operation profile/config says the command requires a platform selector. The app does not choose a platform store automatically.

Exact reference guard:

```text
--expect-profile <ProfileId>
--expect-reference-generation <ReferenceGenerationId>
--expect-reference-view <ReferenceViewId>
```

Guards are optional as a complete profile-defined group. Service derives actual exact reference identity from selected publications.

## Profiles

Every profile-valued option uses exactly one of:

```text
--<name>-profile-id <ExactProfileId>
--<name>-profile-alias <ConfiguredAlias>
```

Mutually exclusive. App validates syntax; service resolves aliases.

Common:

```text
--operation-profile-id|alias
--budget-profile-id|alias
--privacy-profile-id|alias
```

Output:

```text
--output envelope-json|text|artifact
```

Default `envelope-json`.

## Exact root token

```text
--root <RootKind>@<base64url-no-pad(canonical UTF-8 ID bytes)>
```

Allowed `RootKind` values are frozen from the service contract, for example:

```text
project
package
file
source-handle
graph-entity
graph-relation
reference-entity
finding
evidence
map-node
l0-skeleton
l1-skeleton
```

The app checks one delimiter, known kind, base64url alphabet, decoded byte limit, UTF-8, and nonempty canonical ID. It does not validate owner existence; service does.

## `wow context status`

```text
wow context status --native-host-config <host.json>
  --project-store <registration-key>
  --publication current|store-generation:<ProjectStoreGenerationId>
  [--detail summary|capabilities|profiles|generations]
  [--platform omitted]
  [--expect-current-record <CurrentPublicationRecordId>]
  [--expect-current-digest <ContentDigest>]
  [--expect-profile <ProfileId>
   --expect-reference-generation <ReferenceGenerationId>
   --expect-reference-view <ReferenceViewId>]
  [--output envelope-json]
```

Detail defaults to `summary`. Only `envelope-json` is implemented for this
route; it writes the finalized canonical Service envelope unchanged followed
by one LF. Current record/digest guards require `--publication current`; they
are checked once by Service and a mismatch is not retried. An exact store
generation remains selected even when Current has advanced. Reference guards
are optional but, when supplied, require the complete profile/generation/view
group. The only accepted platform value is `omitted`; this CLI route does not
acquire platform or composed status.

Status reports Primary retained project/graph/reference metadata and selected
capability/profile/generation detail. Its successful Service result is
`partial`, with context generation and evaluation `NotEvaluated`; it is not a
context-generation pass. It creates no map, skeleton, pack, source excerpt, or
rendered artifact and returns no source body. Service checks and closes its
retained resources before emitting success.

```text
wow context status --platform native --native-host-config <host.json>
  --platform-status-request-input <request.json> [--output envelope-json]
```

This native-only route transports one strict exact candidate metadata request to
`execute_context_platform_status`. It accepts no Primary selector or root flags.
The four details are carried in the request file. Checked Partial envelopes keep
root coverage and generation/evaluation NotEvaluated. Full E3-C acceptance remains pending.

Composed status uses a separate explicit route:

```text
wow context status --platform combined --native-host-config <combined-host.json>
  --combined-status-request-input <request.json> [--output envelope-json]
```

It decodes `wow-service/context-combined-status-request/1` and invokes exactly
one `execute_context_combined_status` call. The request names exact Primary and
platform store generations/publication sets and an exact platform candidate;
the host supplies the shared Reference view. The result remains Partial and
root coverage/generation/evaluation remain NotEvaluated. Both owners remain held
through metadata construction, with checked platform-then-Primary closure.

See [context status](../CONTEXT_STATUS.md) for the host input, result
fields, refusal behavior, and scope limits.

## `wow context map`

```text
wow context map
  <primary selector options>
  [platform selector options]
  [reference guards]
  --map-target primary|platform|combined
  [--root <exact-root>]
  (--map-profile-id <ID> | --map-profile-alias <ALIAS>)
  [operation/budget/privacy profiles]
  [--output envelope-json|text]
```

No artifact mode because `context_map` returns a structured ProjectMap, not a rendered context artifact.

## `wow context inspect`

```text
wow context inspect
  <primary selector options>
  [platform selector options]
  [reference guards]
  --root <exact-root> [--root <exact-root> ...]
  --level l0|l1|both
  [--l0-profile-id|alias]
  [--l1-profile-id|alias]
  [--intent-profile-id|alias]
  [--expansion-profile-id|alias]
  [--facet <closed-facet-id> ...]
  [operation/budget/privacy profiles]
  [--output envelope-json|text]
```

No filesystem/source path root and no direct artifact mode.

## `wow context build`

```text
wow context build
  <primary selector options>
  [platform selector options]
  [reference guards]
  --root <exact-root> [--root <exact-root> ...]
  (--context-profile-id <ID> | --context-profile-alias <ALIAS>)
  [--intent-profile-id|alias]
  [--expansion-profile-id|alias]
  [--selection-profile-id|alias]
  [--tokenizer-profile-id|alias]
  [--source-profile-id|alias]
  [--boundary-profile-id|alias]
  [--renderer-profile-id|alias ...]
  [--continuation-retention-profile-id|alias]
  [operation/budget/privacy profiles]
  [--output envelope-json|text|artifact]
```

`artifact` requires exactly one requested renderer and one eligible returned artifact.

## `wow context continue`

```text
wow context continue
  (--continuation <base64url> | --continuation-input <PATH|->)
  [--expect-request <ContextRequestId>]
  [--expect-pack <ContextSemanticPackId>]
  [--output envelope-json|text|artifact]
```

Forbidden:

```text
publication selectors
new roots
profile/budget/privacy/renderer overrides
```

Artifact mode requires exactly one eligible returned artifact from the original renderer chain.

## `wow context validate`

This route validates one explicit semantic-pack JSON input through the Primary StructuralOnly owner profile. All flags except `--output` are required; singleton duplicates and unknown flags are rejected. Numeric limits are unsigned decimal values greater than zero. `--output`, if present, accepts only `envelope-json`.

```text
wow context validate
  --input <PATH|->
  --artifact-kind semantic-pack
  --media-type application/json
  --validation-profile-id wow-context/primary-structural-pack/1
  --origin-closure structural
  --max-input-bytes <U64>
  --max-metadata-bytes <U64>
  --max-context-work <U64>
  --max-depth <U32>
  --max-tokens <U64>
  [--output envelope-json]
```

All five limits must be positive unsigned integers; `--max-input-bytes` is at most 64 MiB, `--max-metadata-bytes` is at most the context result-envelope ceiling, and `--max-depth` is at most 128. The parser rejects profile aliases, exact-closure guards, text output, and artifact output. It reads one explicit file or stdin and sends only bytes to Service, never the path. CLI arguments are capped at 4096 entries and 1 MiB total; each option value is capped at 32 KiB. The only supported artifact selection is Primary Structural L0 `ContainersV1` with a typed `GraphEntity` `source_file` or `source_package` root. Within that selection, exact local assertion/derivation chains are admitted only when recomputed record IDs and internal assertion references close within one partition. The chain must be complete, bounded, conflict-free, and closed over its witnesses; missing records/references, conflicts, cycles, incomplete chains, and cross-partition references return `not_evaluated`. Declaration links join original producer assertions to accepted graph records and rebind original node IDs to the published graph generation; ownership edges preserve original relation, confidence, and evidence while using published endpoints. Other levels, roots, profiles, expansions, excerpts, platform/combined families, and rendered artifacts return owner state `not_evaluated`. A completed Service envelope has operation status `complete` and carries owner state `valid`, `invalid`, or `not_evaluated`. `valid` is structural validation only; native producer/catalog authority, owner-generation closure, and source/runtime authority remain `not_evaluated`, and producer/catalog authority is not a StructuralOnly positive prerequisite. `--max-tokens` limits syntactic JSON scanner tokens, not model tokens. A genuine positive StructuralOnly witness and affected-owner Tier S are required before publication. The independent Quick gate runs on the published exact head; no Quick result is claimed here.

## `wow context render`

The selected implementation is:

```text
wow context render
  --input <PATH|->
  --artifact-kind semantic-pack
  --media-type application/json
  --validation-profile-id wow-context/primary-structural-pack/1
  --origin-closure structural
  --renderer-profile-id wow-context/primary-pack-json/1|wow-context/primary-pack-markdown/1
  --max-input-bytes <U64>
  --max-metadata-bytes <U64>
  --max-context-work <U64>
  --max-depth <U32>
  --max-tokens <U64>
  [--output envelope-json]
```

Limits and explicit file/stdin admission use the same bounded reader as
`context validate`. The CLI invokes one render Service operation and writes its
canonical envelope plus one LF. Completed validation maps to Valid 0, Invalid 1
or NotEvaluated 2; unsupported inputs have no rendered artifact. Failure,
internal failure and cancellation retain 3, 4 and 130; pre-Service errors use 64.
The selected route renders only default Primary L0 structural metadata. It
accepts no aliases, host/store/source selectors, exact closure, text/artifact
output or tokenizer claims. Structural validity never upgrades owner-generation,
source/catalog or runtime authority.

The broader planned transport remains unimplemented:

```text
wow context render
  --input <PATH|->
  --media-type application/json
  (--renderer-profile-id <ID> | --renderer-profile-alias <ALIAS>)
  [--tokenizer-profile-id|alias]
  --origin-closure structural|exact
  [exact publication/reference guards required by exact closure]
  [operation/budget/privacy profiles]
  [--output envelope-json|text|artifact]
```

Artifact mode emits the one validated rendered artifact exactly.

## Explicit config

```text
--config <PATH>
```

Config use and precedence are in `SECURITY_AND_CONFIG.md`. No implicit config discovery.

## Common parser errors

Exit 64 before service invocation for:

- unknown command/flag/value;
- missing required/mutually exclusive flag;
- duplicate singleton flag;
- malformed selector/root/profile/continuation token;
- excessive arguments/input path length;
- artifact mode unsupported/ambiguous for command;
- explicit config or artifact transport read/size/decode failure.

Semantic existence/compatibility errors are service results, not parser errors.
