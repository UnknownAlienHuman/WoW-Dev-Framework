# E2-C XML parsing, structure, includes, and embedded Lua

**Status:** normative bounded static XML contract.

## Parser security profile

```text
XmlDialectSecurityProfile
    profile_id/version
    accepted encoding/BOM/namespace policy
    known UI/template/object/script/include elements and attributes
    unknown preservation policy
    DTD/external entity policy = disabled
    entity expansion policy = bounded predefined/internal subset only if explicitly supported
    path/include rules
    depth/node/attribute/text/bytes budgets
    embedded Lua extraction policy
    canonical digest
```

No external entity, network, catalog, schema download, XInclude, processing instruction, extension, handler, script, or source code executes.

## Streaming model

The parser emits bounded source-mapped records without retaining an unrestricted DOM. It maintains only the validated stack/state needed for:

- namespace/element/attribute identity;
- object/template ownership;
- parent/child relationships;
- inheritance references;
- include/script references;
- inline script content spans;
- source order;
- unknown/malformed diagnostics.

A full document tree may be materialized only under explicit bounded fixture profiles.

## Document records

```text
XmlDocument
    document/file/package/universe/generation IDs
    selected XML profile
    root element identity
    ordered records
    include/script/embedded-unit manifests
    unknown/malformed/conflict/coverage records
    canonical digest
```

Every normalized record links exact UTF-8 byte/line span and raw element/attribute observation.

## Includes and script files

Known reviewed forms produce `XmlIncludeRecord` or external `XmlScriptRecord`:

```text
source document/span
referenced normalized path
reference kind = include_xml | script_lua | other_known
semantic ordinal
resolution state
```

Rules:

- resolve only within declared source snapshot roots/universes;
- reject absolute/traversal/device/URI/tokenized paths;
- preserve duplicate references and order;
- recursively expand XML includes under cycle/depth/file/byte budgets;
- repeated include semantics follow the frozen profile; no silent deduplication;
- missing/unsupported references block exact closure for affected load partition;
- source resolution does not authorize host filesystem reads beyond the snapshot.

## Templates and objects

```text
XmlTemplateRecord
    template identity/name/object kind/virtual state
    owning document/package
    inheritance refs
    source span/evidence/coverage
```

```text
XmlObjectRecord
    object identity/type/name
    owner/document/package
    explicit parent reference
    template/inheritance refs
    ordered child/object regions
    source span/evidence/coverage
```

Rules:

- identity uses project/XML semantic schema, not line number or insertion order;
- anonymous objects use deterministic structural/source identity scoped to the exact generation;
- same display name in different owners/documents does not merge;
- unresolved parent/template refs remain unresolved/Possible;
- object parent and inheritance are distinct relations;
- no runtime object/frame existence or accessibility claim.

## Inheritance

```text
XmlInheritanceRecord
    child object/template identity
    ordered parent template/reference keys
    exact source occurrence
    resolution status per target
```

Multiple inheritance entries remain ordered where source semantics require. Cycles and conflicting definitions are retained for graph/project validation; parser does not choose a winner.

## Scripts

```text
XmlScriptRecord
    owning object/template/document
    script kind/name
    source kind = external_file | inline_body | reference_only
    append/prepend/inherited flags when explicitly represented
    handler/reference identity when exact
    source span/order
```

Script records describe static binding only. They do not execute handlers or prove lifecycle, protection, taint, combat, payload readability, or runtime delivery.

## Inline Lua source units

For an inline script body:

```text
XmlEmbeddedLuaUnit
    virtual ProjectLuaUnit ID
    owning XML document/object/template/script record
    exact parent XML byte/line span
    canonical extracted UTF-8 bytes/digest
    deterministic virtual URI/logical path
    load ordinal/phase
    source-map translation profile
```

Rules:

- extraction normalizes only the profile-declared XML text/entity representation needed to recover exact Lua bytes;
- extracted bytes/digest and XML span are frozen/tested;
- virtual identity is scoped to document/script/source occurrence and generation;
- `wow-emmy` parses the unit; XML code never tokenizes/resolves Lua;
- analyzer spans map back through a validated source-map to XML source;
- malformed/unrepresentable content produces explicit failure/NotEvaluated;
- no synthetic wrapper may change Lua semantics without a versioned adapter and source-map/loss record.

## Unknown elements/attributes

Unknowns are retained as bounded raw records with:

```text
qualified name
normalized attributes/text digest
parent/path/source span
profile applicability
potential impact classification
```

Unknowns cannot create graph facts, code, file paths, or agent instructions. Only dependent capabilities downgrade.

## Namespaces and entities

- namespace prefixes and URIs are data under the profile;
- no remote namespace resolution;
- predefined XML entities handled by the parser;
- custom entity/DTD use rejected or explicitly unsupported unless a future safe profile defines a closed internal subset;
- entity expansion and text size strictly bounded.

## Coverage

Partitions include:

```text
project.xml.document:<id>
project.xml.includes:<id>
project.xml.templates:<id>
project.xml.objects:<id>
project.xml.inheritance:<id>
project.xml.scripts:<id>
project.xml.embedded_lua:<id>
```

A failed embedded script does not erase independently valid template/object facts, but script/analyzer/recognizer capabilities become partial/NotEvaluated.

## Required operations

```text
validate_xml_profile
parse_xml_document_streaming
normalize_xml_include
normalize_xml_template
normalize_xml_object
normalize_xml_inheritance
normalize_xml_script
extract_xml_embedded_lua_unit
resolve_xml_references
build_xml_coverage_report
```

## Tests

- known template/object/parent/inherits/script/include structures;
- external script and inline script mapping;
- UTF-8 spans/entity decoding/source-map round trip;
- unknown element/attribute preservation;
- missing/duplicate includes/scripts;
- include cycles/depth/fanout;
- DTD/external entity/XInclude/network attempts;
- entity-expansion/text/attribute/node bombs;
- traversal/absolute/URI paths;
- anonymous/duplicate/colliding object identities;
- unresolved/multiple inheritance;
- inline Lua never executed or parsed twice;
- deterministic records/units under streaming chunk/worker order.
