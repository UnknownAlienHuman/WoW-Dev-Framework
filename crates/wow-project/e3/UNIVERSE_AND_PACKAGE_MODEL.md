# Blizzard UI universe, package, and variant model

**Status:** normative.

## Universe identity

```text
universe_id = blizzard_ui_source:<source-profile-id>:<source-snapshot-id>
```

The exact canonical representation freezes before implementation. It must distinguish product/flavor/build/profile/source content and must not depend on repository display name or local path.

## Project kind

```text
ProjectKind::BlizzardUiPlatformSource
```

This project kind enables only reviewed E3-A profile behavior. It does not alter TOC/XML/Lua grammar or graph semantics based on known package names.

## Separation rules

Never merge identities across:

```text
blizzard_ui_source
first_party_project
declared_dependency_source
analyzer_library
reference
external_candidate
runtime
historical build/source generation
```

Cross-universe relations are separate assertions with exact endpoints and evidence.

## Package discovery

Package roots come from the source profile's configured package-root policy, not from a hard-coded `Blizzard_` prefix. Discovery records:

- every candidate directory/file under package roots;
- all TOC candidates and their normalized variants;
- package root identity;
- package metadata;
- missing/duplicate/conflicting TOCs;
- unknown package layout;
- excluded records and coverage.

A directory without a valid selected TOC may remain an inventory/source container; it is not silently promoted to a loadable addon package.

## TOC variant selection

Exactly one compatible TOC variant is selected per package for one target profile. Selection inputs are explicit:

```text
client product/flavor/channel
build and Interface compatibility
variant filename/metadata policy
caller selection override under reviewed policy
```

Forbidden:

- combining facts from multiple variants;
- filling a missing file/dependency/directive from another flavor;
- choosing newest-looking filename/date;
- defaulting silently when multiple variants are compatible;
- using provider path conventions as universal semantics.

Ambiguous selection is `Conflict` or `NotEvaluated`.

## Package dependency classes

Keep distinct:

```text
required dependency
optional dependency
load-on-demand relationship
bootstrap/static phase role
same-source package containment
library/embed structure
```

Dependency names are exact TOC facts, not proof that the dependency is present, loaded, compatible, or initialized at runtime.

## Package identity

Package identity uses exact source universe, normalized package root/TOC identity, selected variant, and package semantic metadata defined by the profile. Display title/localization does not become the sole identity.

A package rename between source generations yields a new scoped key; E4 lineage may later link it.

## File ownership

One source file may be:

- directly listed by one or more package TOCs;
- included by XML;
- referenced as an XML script;
- shared/embedded by multiple packages;
- inventory-only/unreachable in the selected profile.

Ownership, containment, and load membership remain separate graph relations. No single generic parent field.

## Generated and shared source

Generated-looking filenames/directories do not alter trust or ownership automatically. The materialization profile may mark generated provenance when exact evidence exists. Shared files retain all package/load memberships rather than being assigned to the first package encountered.

## Package coverage

Per package, report independent coverage for:

- inventory;
- variant selection;
- TOC parse;
- referenced-file resolution;
- dependencies;
- XML expansion;
- Lua analyzer units;
- recognizer inputs;
- graph proposals/publication;
- license/provenance.

A package is not `Complete` if a decisive partition is missing, conflicted, truncated, or unsupported.
