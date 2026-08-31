# Blizzard UI source universe and semantic identity

**Status:** normative E3-B universe, key and source-coordinate contract.

## Universe

```text
UniverseId = blizzard_ui_source:<BlizzardUiSourceGenerationId>
```

The universe contains implementation-source entities and assertions for exactly one source generation. It is never an alias for `reference_api`, `first_party_project`, `project_dependency`, `runtime`, `external_candidate` or `historical`.

## Source collection identity

```text
BlizzardUiSourceCollectionId
    provider class and stable collection namespace
    source collection identity schema version
```

A source collection can have many generations. Provider owner/repository/branch display names are provenance, not entity identity.

## Generation identity

`BlizzardUiSourceGenerationId` binds:

- exact source profile;
- materialized snapshot/content manifest;
- root/package/file manifests;
- build-binding decision;
- parser/analyzer/fact-adapter/recognizer/graph profiles;
- source canonicalization and license policy versions;
- capability/coverage/conflict manifests.

It excludes store generation, current pointer, database layout, checkout path, mtime, clock and worker count.

## File identity

```text
BlizzardUiSourceFileKey
    source generation
    logical root ID
    normalized root-relative logical path
    file-kind identity profile
```

Content digest belongs to the generation/file assertion and change manifest. A changed file at a new generation is a new scoped file key. Later lineage may connect file generations.

## Source handle

```text
BlizzardUiSourceHandle
    source generation
    source file key
    canonical logical bytes digest
    encoding/newline profile
    virtual-source mapping profile: optional
```

No absolute host checkout path. Human relative paths are presentation fields.

## Symbol/entity identity

Entity kind definitions come from the graph registry. Typical source identities use exact analyzer/XML/TOC semantic keys:

```text
Lua function/method
    source generation + analyzer semantic declaration/symbol identity

namespace/global/table member
    source generation + exact resolved semantic owner/member key

XML template/object/frame/region
    source generation + XML semantic declaration identity

package/load unit
    source generation + package/TOC/global-unit identity

callback/event registration/state path
    source generation + producer-defined semantic key ingredients
```

Line/column, display name or source text alone is insufficient.

## Named globals

A global name can support a source-universe symbol key under the exact analyzer/global-namespace contract. It does not merge with:

- a public reference API symbol of the same string;
- a user project global of the same string;
- a runtime global value;
- a historical generation's global.

Bridges are explicit relations.

## XML identity

XML template/object identity uses exact source generation, declaration identity and registry-defined name/ownership qualifiers. Duplicate names within incompatible scopes produce ambiguity/conflict records.

A user project XML template with the same name remains a project entity. `inherits` or `references_template` connects exact endpoints only after profile-compatible resolution.

## Generated and embedded source

Generated API glue, embedded libraries and other source classes remain in the `blizzard_ui_source` universe but retain exact root role, package, generator/provider provenance and license class.

They are not reclassified as reference APIs or third-party dependencies by path alone.

## Virtual source units

XML inline Lua or other admitted virtual units use:

```text
virtual source handle
owning physical source handle/span
virtual byte digest
source-map profile/version
semantic unit identity
```

Both virtual and physical mappings remain exact and generation-bound.

## Reference entity identity

Reference API entities keep their existing `reference_api` scope and `ReferenceEntityKey`. A source graph bridge points to that exact key; it does not clone the reference entity into the source universe.

## User project entity identity

A user project keeps `first_party_project:<ProjectGenerationId>`. E3-B publication contains no user-project entity assertions. Per-project bridge partitions name exact source and user project generations later.

## Cross-build identity

Same path/name/signature across source generations is not identity equality. E3-B exposes exact generation-scoped entities; E4 lineage may later assert `same_lineage_as`, `replaced_by`, `moved_to` or other relations with evidence.

## Collision handling

Reject or conflict:

- normalized logical path collision;
- duplicate incompatible package/TOC identity;
- analyzer semantic key collision;
- XML declaration key collision;
- one provider object mapped to conflicting root/file identities;
- same proposed graph key with incompatible identity ingredients;
- reference/source endpoint scope mismatch.

Never resolve by first/last traversal, case-fold convenience, popularity or nearest source location.

## Identity mutation suite

Mandatory mutations:

```text
rename provider/repository/branch/display metadata
move the sealed snapshot to another host path
shuffle materialization/file enumeration
change mtimes and executable bits without content/profile meaning
rename irrelevant local identifiers
change one decisive public semantic identifier
change one source file content byte
change build-binding evidence only
change license/redistribution decision only
```

Expected:

- provider/host/order/mtime changes do not change semantic outputs where excluded;
- irrelevant local rename changes only exact local entities/relations and dependent outputs;
- decisive semantic/content/profile changes create the documented new identities;
- build/license changes update their independent axes and dependent publication eligibility without rewriting source bytes.
