# E3-B context requests and reviewed profiles

**Status:** normative request/control-plane schema. All profiles are nonexecutable repository-owned inputs.

## Context request

```text
ContextRequest
    exact ContextUniverseSetId
    exact root selector(s)
    intent profile ID
    requested facets/axes/relations
    required capabilities/fields
    confidence/provenance/coverage policy
    source excerpt and reference enrichment policy
    privacy/consumer trust policy
    semantic budget/tokenizer profile
    renderer profile(s)
    continuation cursor: optional
    cancellation
    opaque audit metadata: optional and nonsemantic
```

The normalized request receives a stable ID/digest. Unknown fields or profile versions are rejected rather than ignored.

## Exact root selectors

E3-B selectors are closed and exact:

```text
ProjectId / ProjectSnapshotId
PackageId
FileId / SourceHandleId / exact source span
Graph EntityKey / EntityId
Graph RelationKey / RelationAssertionId
ReferenceEntityKey / ReferenceEntityId
Existing FindingId / EvidenceId
ProjectMapNodeId
L0SkeletonId / L1SkeletonId
```

A selector may include a bounded ordered set of exact IDs. It cannot contain a fuzzy name, regex, SQL, source expression, natural-language query, model prompt, filesystem path, or callback.

If an application receives user text, search/service resolves it before E3-B and submits explicit chosen root IDs plus optional nonsemantic audit text.

## Intent profiles

Intent affects which typed facets and axes are useful; it never changes facts. Initial reviewed intent families:

```text
NavigateProject
InspectContainer
InspectEntity
TraceLoad
TraceCall
TraceSignal
TraceHook
TraceState
InspectObjectTemplateMixin
ExplainApiUse
ReviewExistingFindingEvidence
PrepareChangeContext
```

`PrepareChangeContext` supplies relevant exact structure only. It does not generate a plan, fix, or edit.

Each intent profile defines:

- allowed root kinds;
- required map/L0/L1 layers;
- required and optional facet sets;
- allowed graph axes/relation kinds/directions;
- source/reference enrichment rules;
- confidence/provenance policy;
- stage/depth/fanout maxima;
- mandatory/optional priority tiers;
- stop conditions;
- permitted render formats;
- completeness semantics.

## Map profile

```text
ProjectMapProfile
    profile ID/version
    active node/edge/facet/group classes
    root enumeration policy
    mandatory closure
    per-kind limits
    grouping rules
    ordering version
    pagination/continuation policy
```

No path substring, addon/provider name, popularity, model score, or executable rule.

## L0 profile

```text
L0Profile
    scope-kind permissions
    typed section ordering
    declaration/member inclusion rules
    direct relation/facet sets
    counts/member-page limits
    source documentation handle policy
    mandatory closure
    ordering/pagination
```

## L1 profile

```text
L1Profile
    root-kind permissions
    exact signature/type/source fields
    facet/axis/relation sets
    depth/fanout/path limits
    API/reference enrichment
    source excerpt candidate rules
    mandatory closure
    ordering/pagination
```

## Expansion profile

```text
ContextExpansionProfile
    ordered stage definitions
    allowed owner operations
    stage prerequisites
    candidate kinds
    required evidence additions
    per-stage budgets
    max rounds/depth/frontier
    stop/no-new-evidence policy
    dependency and dedup policy
```

The schema is closed and non-Turing-complete. No loops other than profile-bounded framework expansion rounds; no user/source code.

## Selection/pruning profile

```text
SelectionProfile
    mandatory item classes
    priority tiers
    stable tie-break keys
    dependency closure rules
    duplicate/equivalence rules
    per-class reservation/minimum/maximum
    omission reason mapping
```

It cannot inspect free source text for semantic relevance or use model/embedding scores.

## Budget profile

```text
ContextBudgetProfile
    hard semantic byte limit
    hard rendered byte limit(s)
    optional token limit(s)
    minimum mandatory allocation
    map/L0/L1/relation/reference/excerpt/metadata pools
    per-item/per-section/per-source maxima
    path/depth/fanout/item limits
    overflow/failure policy
```

See `BUDGETS_TOKENIZATION_AND_PRUNING.md`.

## Tokenizer/estimator profile

```text
TokenizerProfile
    accounting class
    implementation ID/version
    vocabulary/model/config digest where exact
    normalization/special-token policy
    deterministic test vectors
    maximum input/output constraints
```

A model display name alone is insufficient.

## Source excerpt profile

```text
SourceExcerptProfile
    allowed universes/source classes
    declaration/body/adjacent-context rules
    exact range expansion policy
    per-item/total byte and line limits
    encoding/source-map policy
    redaction/transformation permissions
    continuation behavior
```

No arbitrary file read or path-based trust.

## Privacy and consumer trust profile

```text
PrivacyConsumerProfile
    consumer trust class
    allowed source privacy labels
    metadata/source/excerpt permissions
    absolute-path and private-field policy
    external transmission permission
    license/redistribution requirements
    redaction policy ID
    audit requirements
```

Unknown privacy state defaults to the profile's safest explicit behavior; it never silently permits source inclusion.

## Source boundary profile

Defines canonical JSON string encoding, deterministic Markdown source line-prefixing, boundary notices, transformation records, and validation tests. Source text cannot define its own boundary.

## Renderer profile

```text
RendererProfile
    format/media type
    semantic pack schema compatibility
    section ordering
    label/template catalog
    line endings/encoding
    source boundary profile
    rendering loss policy
    exact tokenizer profile: optional
    hard output limits
```

A renderer cannot request facts omitted from the semantic pack unless an explicit new semantic request is created.

## Profile trust and evolution

- Profiles are compiled/static or loaded from repository-owned signed/reviewed canonical data.
- User/project/source-provided profiles are not active in E3-B.
- Unknown versions fail.
- Changing selection, ordering, canonicalization, budgeting, tokenization, privacy, or rendering semantics creates a new profile version and output IDs.
- Additive fields are accepted only under explicit schema compatibility.
- Tests pin every profile and canonical digest before implementation.

## Request validation

Reject:

- unresolved/fuzzy roots;
- incompatible root and intent;
- missing required universe/capability;
- impossible/zero/unlimited budgets;
- unknown relation/facet/profile IDs;
- confidence/authority upgrade request;
- source excerpt request denied by privacy/license policy;
- executable predicates or source-controlled configuration;
- continuation cursor for another request/universe/profile;
- renderer/tokenizer combination without validated compatibility.
