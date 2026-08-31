# E2-C TOC parsing and package model

**Status:** normative bounded lexical/semantic contract.

## Parser boundary

The TOC parser reads exact UTF-8 bytes from a validated `ProjectSourceFile`. It is line-oriented, nonexecuting, profile/version bound, and preserves every line as one of:

```text
blank/comment
metadata directive
file entry
known suffix/tag token
unknown/unsupported record
malformed record
```

No line is executed or interpreted as shell/Lua/config instructions.

## Dialect profile

```text
TocDialectProfile
    profile_id/version
    accepted encoding/BOM/line ending policy
    metadata key grammar and known normalized keys
    file-entry grammar and recognized suffix/tag tokens
    flavor/variant filename and Interface selection rules
    dependency/SavedVariables/list splitting rules
    comment/escaping rules
    unknown/malformed recovery policy
    budgets
    canonical digest
```

The profile is pinned before implementation. Unknown syntax is preserved and affects only dependent capabilities.

## Variants

A package can contain multiple TOC documents. Each becomes an independent `TocVariant` with exact applicability evidence:

```text
flavor/edition
Interface metadata
filename/profile convention
explicit project selection
```

Selection rules:

- exactly one active variant per package/target;
- selected flavor/interface/profile must agree with project configuration;
- no fallback by newest/highest Interface without policy;
- no union of files/directives from multiple variants;
- ambiguous/no compatible variant is a typed failure or partial candidate under explicit policy;
- unselected variants remain historical/comparison inputs only.

## Metadata directives

Known projections can include profile-reviewed keys such as:

```text
Interface
Title/Notes/Author/Version metadata
Dependencies/RequiredDeps
OptionalDeps
LoadOnDemand
SavedVariables
SavedVariablesPerCharacter
```

Other directives, including `X-*` and patch-specific keys, are retained as raw normalized records. Unknown directives do not silently become ignored if they can affect load behavior.

## File entries

```text
TocFileEntry
    package/variant/document
    semantic ordinal
    normalized referenced path
    kind = lua | xml | other-known | unknown
    known tags/suffixes including explicit Bootstrap when profile supports it
    resolution state against exact source snapshot
    source span/raw record
```

Rules:

- order is exactly source order after comments/metadata handling;
- no filesystem sorting;
- path/root/security policy applied before resolution;
- missing entry is explicit and blocks complete load/files capability;
- duplicate entries retained/classified; they are not silently deduplicated;
- unsupported tag/token preserved and may block exact load semantics;
- `[Bootstrap]` marks the static bootstrap role only.

## Dependencies

```text
TocDependencyDeclaration
    package/variant
    dependency name
    required | optional
    source key/ordinal/span
    exact resolved package identity when configured
    resolution state
```

Rules:

- preserve declaration order and duplicates/conflicts;
- resolve only against declared package universes;
- required missing dependency blocks complete package load plan;
- optional missing dependency remains explicit but does not become a required failure;
- no source download/discovery;
- dependency names do not infer repository or version.

## LoadOnDemand and bootstrap

```text
TocLoadOnDemandRecord
    declared value/raw source
    normalized tri-state = true | false | unknown
```

Bootstrap records identify files explicitly marked by the accepted profile. Static model can state:

```text
bootstrap candidate unit
full package units
load-on-demand metadata
```

It cannot state when/why the client actually loaded the addon, whether initialization succeeded, or whether frames/children exist at `ADDON_LOADED`.

## SavedVariables

```text
TocSavedVariableDeclaration
    exact variable name
    account | character scope
    package/variant
    semantic ordinal/span
    normalization/validation state
```

Rules:

- declarations only; never contents;
- preserve duplicates/conflicts/order;
- invalid/dynamic-looking names are retained/diagnosed, not executed;
- selected variant only seeds active state roots;
- same-named Lua global without declaration is not a persistent root.

## File closure

The selected TOC defines primary static load roots. File closure expands XML includes/script file references under the XML/load model. Source files present but unreachable remain inventory records and can be queried as unreachable; they do not enter analyzer Main workspace by default unless project policy explicitly includes nonloaded support/test files in a separate scope.

## Raw preservation and coverage

Every TOC line maps to a raw record/source span. Normalized projections link back to it. Coverage is partitioned by document/key/file/dependency/SavedVariables/load semantics. One unknown metadata key does not invalidate unrelated known file order, but an unknown suffix on a file entry can block exact phase/order semantics for that entry.

## Budgets

Bound:

```text
TOC files per package
file bytes/lines/line length
metadata/directive/file/dependency/variable records
list items/string bytes
variant count
unknown/malformed records
```

Budget exceed yields Partial/Failed with exact processed/omitted scope, never a complete truncated TOC.

## Required operations

```text
validate_toc_dialect_profile
parse_toc_document
normalize_toc_directive
normalize_toc_file_entry
build_toc_variant
select_toc_variant
resolve_toc_dependencies
resolve_toc_file_entries
build_toc_saved_variable_facts
build_toc_coverage_report
```

## Tests

- LF/CRLF/BOM/profile encoding;
- comments/blanks/metadata/files/source spans;
- one selected variant and no cross-variant union;
- Interface/flavor mismatch/ambiguity;
- required/optional dependencies and duplicates;
- LOD true/false/unknown;
- Bootstrap known/unknown tag;
- SavedVariables scopes/duplicates/invalids;
- missing/duplicate/path-escaping file entries;
- unknown directives/tokens preserved;
- malformed and budget-limited documents;
- no runtime/frame/readiness claim;
- deterministic parse/normalization under worker/input changes.
