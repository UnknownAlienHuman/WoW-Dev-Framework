# `wow-annotations` E1-C annotation projection contract

**Status:** implementation-ready E1-C contract package; no Rust code yet.

**Contract ID:** `wow-annotations/e1-projection-and-parity`

This package defines the first persistent-reference-to-annotation transformation. It consumes one exact read-only E1 `ReferenceView` and produces one deterministic, profile-bound annotation artifact suitable for EmmyLua/LuaLS consumers and semantic comparison with Ketho-style output.

The artifact is a projection. It is not the canonical source of WoW API, raw metadata, restriction facets, corrections, coverage, or provenance.

## Mission

E1-C renders exact supported reference facts into a small, deterministic, static LuaCATS/Emmy annotation library while preserving enough source mapping and projection-loss metadata for downstream diagnostics and parity review.

A future implementation agent must prove:

```text
one exact ReferenceView/profile/reference generation is required
one versioned semantic annotation model is built before text rendering
one versioned Ketho-compatible layout/rendering profile is selected explicitly
supported callable/table/event/enum/CVar/widget/script-object/type facts lower deterministically
known Secret/restriction facts project only through explicit analysis types/tags/sidecars
all unrepresentable or partially represented reference metadata is reported, never dropped silently
generated source cannot execute source-provided instructions or inject extra LuaCATS declarations
artifact identity includes reference generation, renderer/type-lowering/layout/consumer profiles
output files, source maps, semantic manifests, loss reports, and checksums are deterministic
EmmyLua and LuaLS compatibility/parity probes can validate the artifact without editor mutation
Ketho comparison is semantic, version/profile scoped, and never overwrites platform source truth
```

## Owned responsibilities

- exact `ReferenceView` input validation and capability requirements;
- versioned consumer-neutral annotation semantic model;
- stable declaration/module/file partitioning;
- LuaCATS/Emmy type lowering and declaration rendering;
- Ketho-compatible path/layout/rendering profile where explicitly frozen;
- WoW dialect/global library projection;
- generated source sanitization and injection prevention;
- generated-file source map back to reference entity/fact/raw/source/evidence handles;
- projection coverage and loss classification;
- semantic artifact manifest and content digests;
- deterministic file/declaration/member/doc ordering;
- consumer capability profiles for EmmyLua/LuaLS;
- semantic parity manifests and differential comparison inputs/results;
- compatibility probe contracts and promotion gates;
- bounded artifact size/file/declaration/documentation budgets;
- annotation-specific errors, fixtures, and golden outputs.

## Explicit non-responsibilities

E1-C does not:

- acquire or parse Blizzard source;
- execute Lua or generated annotation files;
- decide platform facts, current profiles, corrections, or negative authority;
- own raw metadata or restriction truth;
- mutate editor/workspace/user settings;
- add generated libraries automatically to an editor configuration;
- include the full Blizzard UI implementation source tree;
- build the complete TOC/XML/UI graph or source skeletons;
- run or own EmmyLua/LuaLS processes inside the library crate;
- use Ketho/Numy/LuaLS/Emmy output as platform authority;
- suppress analyzer diagnostics to imitate an oracle;
- infer aliases, replacements, types, restrictions, or globals from similarity/prose;
- persist SQLite/store state;
- expose filesystem/process/network/shell operations;
- assemble/sign/distribute final releases or add CI.

## Required reading

Read in order:

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../../DEPENDENCY_GRAPH.md`](../../DEPENDENCY_GRAPH.md)
3. [`../../WORKSTREAMS.md`](../../WORKSTREAMS.md)
4. existing crate brief one directory above;
5. [`../../wow-core/CONSUMER_GUIDE.md`](../../wow-core/CONSUMER_GUIDE.md)
6. [`../../wow-reference/e1/README.md`](../../wow-reference/e1/README.md)
7. [`../../wow-reference/e1/REFERENCE_VIEW.md`](../../wow-reference/e1/REFERENCE_VIEW.md)
8. [`AGENTS.md`](AGENTS.md)
9. [`DECISIONS.md`](DECISIONS.md)
10. [`DATA_MODEL.md`](DATA_MODEL.md)
11. [`SEMANTIC_MODEL.md`](SEMANTIC_MODEL.md)
12. [`TYPE_LOWERING.md`](TYPE_LOWERING.md)
13. [`LAYOUT_AND_RENDERING.md`](LAYOUT_AND_RENDERING.md)
14. [`DIALECT_AND_GLOBALS.md`](DIALECT_AND_GLOBALS.md)
15. [`SOURCE_MAP_AND_LOSS.md`](SOURCE_MAP_AND_LOSS.md)
16. [`PARITY_AND_CONSUMER_PROBES.md`](PARITY_AND_CONSUMER_PROBES.md)
17. [`SECURITY_AND_SANITIZATION.md`](SECURITY_AND_SANITIZATION.md)
18. [`ERROR_MODEL.md`](ERROR_MODEL.md)
19. [`TEST_MATRIX.md`](TEST_MATRIX.md)
20. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
21. [`CONTRACT.json`](CONTRACT.json)
22. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

Normative repository sources:

- [`../../../docs/REFERENCE_PACK.md`](../../../docs/REFERENCE_PACK.md)
- [`../../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../../docs/PROVENANCE_AND_COVERAGE.md`](../../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../../docs/SECURITY_MODEL.md`](../../../docs/SECURITY_MODEL.md)
- [`../../../docs/TEST_STRATEGY.md`](../../../docs/TEST_STRATEGY.md)
- [`../../../docs/RESEARCH_BASELINE.md`](../../../docs/RESEARCH_BASELINE.md)

## Direct framework dependencies

```text
wow-core
wow-reference
```

No dependency on `wow-store`, `wow-emmy`, `wow-project`, `wow-service`, or application crates.

Consumer probes and Ketho/LuaLS/Emmy comparisons run through external test/tool adapters, not hidden runtime dependencies of `wow-annotations`.

## Input contract

```text
AnnotationBuildRequest
    exact ReferenceView ID
    exact ProfileId / ReferenceGenerationId
    exact required capability set
    annotation semantic model version
    type-lowering profile
    layout/rendering profile
    dialect/global profile
    consumer capability profiles
    documentation/sanitization policy
    source-map/loss policy
    file/declaration/output budgets
```

The view must be immutable and profile/generation exact. Missing required reference capabilities produce typed `NotEvaluated`/partial artifact state; they are never inferred from another profile/source/oracle.

## Output contract

```text
AnnotationArtifact
    artifact ID/version
    exact ProfileId / ReferenceGenerationId
    semantic model manifest
    rendered file manifest and file bytes/digests
    source map manifest
    projection coverage/loss report
    consumer profile manifest
    semantic parity baseline/result refs
    licenses/provenance/tool identities
    artifact capability/eligibility state
    canonical digest
```

Generated files are static analysis stubs. They are not runtime implementations and must not be loaded by a WoW addon.

## E1 semantic scope

Active projections:

```text
API systems/namespaces and functions/methods
parameters and returns
named tables/structures and fields
events and payload types/aliases under the frozen profile
enums and enum values
CVars where supported
widgets/script objects and methods where supported by ReferenceView
named type aliases/unions/collections/callbacks in the frozen type model
selected explicit deprecation metadata
WoW dialect/global declarations
nominal Secret analysis types and explicit supported restriction metadata projections
```

Deferred/sidecar-only unless explicitly supported:

```text
all raw unknown metadata
complete restriction predicate logic
runtime/hotfix-sensitive secrecy state
complete source documentation fidelity
full UI implementation bodies
TOC/XML/package/object/call graph
source skeletons
search/lineage/replacement candidates
```

Loss/sidecar records remain mandatory.

## Semantic model before text

The renderer consumes a normalized `AnnotationSemanticModel` rather than directly formatting ReferenceView rows. This layer:

- validates exact input closure;
- resolves declaration kinds/owners/member order;
- lowers reference types into consumer-neutral annotation types;
- classifies unsupported/lossy constructs;
- defines semantic declaration IDs independent of file layout/text;
- keeps docs, deprecation, restrictions, and source evidence as separate fields;
- enables semantic parity independent of whitespace/file partitioning.

## Type lowering

Every source type/fact lowers to one of:

```text
Exact
ExactWithSidecar
LossyDeclared
Unsupported
NotEvaluated
```

`any`, `unknown`, broad union, omitted return, or prose-only fallback cannot hide a loss. The projection-loss report identifies the original reference fact, lowering rule/version, consumer limitation, emitted form, and affected capabilities.

## Ketho compatibility

Compatibility means a versioned profile of:

```text
logical declaration/type semantics
expected namespaces/systems/globals
consumer-recognized LuaCATS constructs
familiar deterministic artifact layout where useful
selected event/enum/CVar/widget/script-object conventions
```

It does not mean:

```text
copying editor setting mutation
auto-injecting libraries/globals
suppressing diagnostics
executing Ketho build/runtime
blind byte equality
accepting oracle output over pinned ReferenceView
```

Parity discrepancies create differential records; they do not silently rewrite the artifact.

## No editor mutation

The crate returns artifact bytes/plans/manifests only. A higher application may write a selected artifact directory under a configured output root and generate a separate analysis configuration under `.wow/generated/...`, but `wow-annotations` does not discover or edit VS Code, LuaLS, Emmy, or user/workspace settings.

## Generated source safety

Reference documentation, names, string values, and raw metadata are untrusted data. Rendering must prevent them from:

```text
closing comments/strings unexpectedly
creating new annotation directives
defining executable code outside fixed inert stubs
changing file/module boundaries
injecting filesystem paths or editor commands
producing invalid/ambiguous Lua identifiers without explicit safe rendering
```

Only renderer-owned templates/directives/code shapes are emitted. Source prose is escaped/sanitized and can be omitted with a loss record.

## Source maps

Every generated declaration/member/type/documentation fragment can map to:

```text
ReferenceEntity/Fact ID
raw observation/correction application IDs
source handles/evidence IDs
coverage/conflict state
lowering/rendering rule IDs
```

Generated source spans are computed after final deterministic rendering and stored in a separate source-map artifact. A generated line is not platform authority by itself.

## Artifact eligibility

### Fixture

Synthetic/partial annotation corpus.

### Candidate

Real ReferenceView but one or more consumer/parity/loss/coverage gates pending.

### Release-ready annotation artifact

All declared mandatory projection, source-map, loss, deterministic rendering, consumer compatibility, and parity gates pass for the exact profile/reference generation.

Final Reference Pack release eligibility is assembled later and may additionally require ReferenceData/store/license/signing gates.

## Hard stops

- No Rust code or `Cargo.toml` in this documentation phase.
- No raw source parsing or platform fact derivation.
- No direct SQL/store/process/network/editor access.
- No full Blizzard UI source in output.
- No runtime function bodies or executable source-provided code.
- No editor setting/global/library mutation.
- No silent `any`, omission, doc drop, restriction loss, or unknown metadata loss.
- No oracle-over-source overwrite.
- No cross-profile/generation artifact.
- No nondeterministic file/declaration/member order.
- No generated annotation/source injection.
- No final release/CI automation.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- annotation semantic model;
- type-lowering cases;
- deterministic layout/rendering files;
- sanitization cases;
- source-map/loss cases;
- Ketho semantic parity and Emmy/LuaLS consumer probe results;
- artifact manifest and checksum freeze.

Exact reference input, renderer, layout, consumer, declaration, source-map, loss, parity, artifact, file, and SHA-256 values freeze before the first E1-C Rust commit.

## Definition of done

E1-C implementation is complete only when:

```text
one exact ReferenceView transforms into one deterministic semantic model and rendered artifact
all supported declarations/types/members retain exact profile/reference/source mapping
all unsupported or lossy facts produce explicit loss/coverage records
source-controlled text cannot inject directives/code or alter artifact topology
artifact loads in the frozen EmmyLua and LuaLS consumer probes without editor mutation
semantic parity results against the pinned Ketho oracle are classified and reviewed
repeated 1/2/N worker runs produce identical semantic/file/source-map/loss/artifact digests
no annotations become platform authority and no raw metadata is silently lost
all TEST_MATRIX cases and checksum vectors pass
```
