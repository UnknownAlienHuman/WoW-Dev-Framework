# Blizzard UI source compatibility profile — `wow-reference` owner seam

**Status:** normative E3-B supporting contract; `wow-reference` does not ingest implementation source.

## Ownership

`wow-reference` owns exact public API/reference profiles, ReferenceGeneration, canonical reference entity keys, restrictions, coverage and negative authority.

For E3-B it additionally defines a read-only compatibility/bridge input contract that states whether one exact Blizzard UI source profile can be evaluated alongside one exact reference profile.

It does not:

- fetch, materialize, parse, analyze or store Blizzard UI implementation source;
- copy source entities into the reference universe;
- use implementation source to add/remove public API entities;
- infer restrictions, Secret Value, protected/forbidden, taint, combat or runtime behavior from source;
- publish source/project graph assertions;
- depend on `wow-project` or `wow-graph` implementations by reversing the dependency graph.

## Compatibility record

```text
ReferenceUiSourceCompatibility
    compatibility_id/version
    exact ReferenceProfileId / ReferenceGenerationId
    exact BlizzardUiSourceProfileId / source generation candidate
    game family/flavor/build/interface selectors
    source build-binding ID/state
    compatibility evidence IDs
    compatible reference entity-kind and alias profiles
    allowed bridge capability IDs
    conflicts and unresolved checks
    state:
        CompatibleExact
        CompatibleLimited
        Incompatible
        NotEvaluated
    canonical digest
```

`CompatibleLimited` names exact capabilities that remain usable. It is not promoted to exact compatibility.

## Authority separation

```text
ReferenceData / ReferenceView
    authority for the exact modeled public API contract and restrictions

BlizzardUiSourceView
    source-structure evidence for exact implementation bytes

ReferenceUiSourceCompatibility
    permits bounded cross-universe resolution; does not change either authority
```

Examples:

- Source calls `C_X.Y`: source evidence can support a bridge request; ReferenceView decides whether exact `C_X.Y` is modeled under that profile.
- Source does not contain `C_X.Y`: reference negative authority is unchanged.
- Source comment says an API is restricted: comment remains source text, not a reference correction.
- Reference marks a value Secret/restricted: source usage does not declassify it.

## Public reference bridge view

A bridge builder receives a minimal exact view:

```text
ReferenceUiBridgeView
    exact compatibility/reference profile/generation IDs
    canonical entity lookup by exact key
    exact versioned alias records
    exact namespace/member/type/event/widget keys
    capability/coverage/conflict/negative-authority records
    evidence/source handles for returned reference facts
```

No fuzzy, nearest-build, current/latest or implementation-source fallback.

## Aliases

Reference aliases are explicit versioned facts with provenance and scope. They may participate in exact bridge resolution only when:

- the source analyzer fact identifies the alias use exactly;
- the alias record is valid for the reference profile;
- target uniqueness/ambiguity is evaluated;
- coverage/conflicts permit the conclusion.

A convenience lowercase/suffix/name similarity is not an alias.

## Event and callback keys

Keep separate reference capabilities for:

```text
native frame events
reference-modeled EventRegistry native bridges when available
custom registry signals
CVar callbacks
other callback/event classes
```

The compatibility profile states which event/entity classes can be bridge targets. A source string alone cannot switch classes.

## Build binding

`wow-reference` can contribute exact reference build/interface evidence to the E3-B build-binding decision. It cannot certify the source revision alone. The final source build binding combines provider/source/reference evidence in `wow-project` E3-B.

## Corrections

Reference corrections remain digest-bound reference corrections. Implementation source observations cannot mutate reference data automatically. A source discrepancy may produce a research/conflict record for review; any accepted correction follows the existing reference correction contract with its own evidence.

## Coverage

Compatibility coverage is independent from reference ingestion coverage and source ingestion coverage. Missing alias/entity/event capability returns `NotEvaluated` or scoped negative authority according to the exact ReferenceView records.

## License

The bridge view exposes reference data under its own artifact/license policy. It does not authorize redistribution of implementation source or source-derived artifacts.

## Tests

- exact compatible source/reference profile;
- provider-declared but not exact build binding;
- cross-flavor/build mismatch;
- exact canonical API lookup;
- exact versioned alias lookup;
- nearest/case/suffix match rejected;
- missing endpoint complete vs partial coverage;
- source absence does not alter reference negative authority;
- source comment/restriction claim cannot become correction;
- reference Secret/restriction remains authoritative despite source usage;
- native/custom/CVar event classes remain separate;
- compatibility update invalidates only dependent bridge partitions.
