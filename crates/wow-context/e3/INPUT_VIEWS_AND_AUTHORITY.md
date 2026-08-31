# Input views, universe boundaries, and authority

**Status:** normative.

## Accepted inputs

`wow-context` receives already-opened immutable public views:

```text
ProjectView
    exact ProjectSnapshot/ProjectGeneration
    package/TOC/XML/load/source/analyzer/recognizer manifests
    source-handle resolver capability

GraphView
    exact GraphSnapshot/GraphGeneration
    registry, entity/relation assertions, axes, conflicts, coverage

optional Reference or BlizzardUi GraphView
    exact separately scoped universe/generation
    explicit graph bridge relations only
```

It never receives a SQLite connection, database path, filesystem root, repository checkout, analyzer session, recognizer engine, or raw source catalog.

## Coherence checks

Before projection:

- project and graph views name the same expected project/publication set;
- graph project-universe assertions bind the exact ProjectGeneration;
- analyzer snapshot, profile, reference generation, recognizer packs, and graph registry match the project publication;
- optional reference/UI/dependency views use exact compatible profiles and explicit universe IDs;
- every source handle resolves only within the exact input source snapshot;
- every requested entity/axis/relation exists in the pinned graph registry;
- capability, coverage, conflict, and truncation summaries resolve to exact records.

A mismatch is a typed error, not a best-effort join.

## Authority order

For one context field:

1. direct authoritative project/reference source fact with exact evidence;
2. accepted graph assertion preserving that evidence/provenance/confidence;
3. deterministic context derivation defined by the ContextProfile;
4. deterministic rendering of the canonical field;
5. external candidate/model/search material only in later explicit nonauthoritative lanes.

Context text never outranks its cited input.

## Universe rules

Universes remain distinct:

```text
first_party_project
project_dependency
analyzer_library
reference_api
blizzard_ui_source
external_candidate
historical
runtime
```

Same name, path suffix, symbol label, or signature does not merge universes. A project function and Blizzard UI function may be linked only by an explicit graph relation under an exact profile.

## Cross-universe bridge examples

Allowed when graph-valid and exact:

```text
project function --uses_api--> reference API symbol
project XML object --inherits--> Blizzard UI XML template
project hook --hooks--> Blizzard UI function/method
project library --embeds_library--> dependency library entity
```

The context projection retains both endpoint scopes and the bridge assertion. It never rewrites one endpoint into the other universe.

## Confidence

```text
Proven
    direct structural/source/reference evidence accepted by the owner

Derived
    deterministic graph/context rule over adequate inputs

Possible
    structurally plausible but not established; explicit opt-in only

Candidate
    discovery hypothesis; excluded from default E3-A artifacts
```

A path containing `Possible` cannot be rendered as proven. A context summary cannot upgrade a relation because multiple weak assertions agree.

## Coverage

Keep separate:

- project source/TOC/XML/load coverage;
- analyzer fact/finding coverage;
- recognizer rule/partition coverage;
- graph registry/assertion/publication/query coverage;
- source-excerpt availability/redaction coverage;
- context projection/selection/rendering coverage;
- budget/truncation coverage.

`Complete` context rendering means only that all selected canonical fragments rendered successfully. It does not repair incomplete upstream coverage.

## Negative authority

Context may state that a relation/entity/section is absent only when the underlying exact graph/project query returns authoritative absence for the relevant producer partitions and no conflict/truncation/redaction blocks that conclusion.

Otherwise use one of:

```text
not present in selected bounded view
not found under partial coverage
not evaluated
conflicted
truncated
source unavailable or redacted
```

## Patch-sensitive claims

Current WoW API, Secret Value, protected/forbidden, taint, lifecycle, event payload, and runtime behavior cannot be inferred from a project map or skeleton. Such fields require exact reference/rule/runtime evidence routed through the current external KB and owning framework components.

## Source-reader seam

```text
ContextSourceReader
    validate_source_handle(snapshot, handle)
    read_exact_span(snapshot, handle, span, byte_limit)
    map_virtual_to_physical_span(snapshot, handle, span)
    classify_private_or_redacted_ranges(snapshot, handle, span, profile)
```

The seam is supplied by the caller and is read-only. It does not accept arbitrary paths or URLs.

## No fallback ladder

The following are forbidden:

- graph miss -> scan source directly;
- source miss -> search the repository;
- missing exact entity -> fuzzy name match;
- absent reference graph -> use current online docs;
- partial coverage -> omit the warning;
- conflict -> choose the most popular assertion;
- missing source excerpt -> reconstruct it from a model.
