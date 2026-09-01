# E3-C context CLI commands

**Status:** normative command grammar. Concrete parser library is not selected yet.

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
wow context status
  [primary selector options]
  [platform selector options]
  [reference guards]
  [--detail summary|capabilities|profiles|generations]
  [operation/budget profile]
  [--output envelope-json|text]
```

`artifact` output is invalid for status.

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

```text
wow context validate
  --input <PATH|->
  --artifact-kind semantic-pack|rendered-artifact
  --media-type application/json|text/markdown
  (--validation-profile-id <ID> | --validation-profile-alias <ALIAS>)
  --origin-closure structural|exact
  [exact publication/reference guards required by exact closure]
  [operation/budget profiles]
  [--output envelope-json|text]
```

The app reads bounded bytes and sends no host path. Artifact output is invalid.

## `wow context render`

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
