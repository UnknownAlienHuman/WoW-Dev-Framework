# `wow-reference` E1-B persistent reference-data contract

**Status:** implementation-ready E1-B contract package; no Rust code yet.

**Contract ID:** `wow-reference/e1-persistent-schema-and-build-plan`

This package extends the existing E0 fixture-backed `wow-reference` contract into the first full persistent ReferenceData build/read slice. It deliberately lives under `e1/` so the proven E0 fixture contract remains stable and agents cannot silently rewrite it while implementing a larger builder.

## Mission

E1-B turns one exact, materialized Blizzard UI/API source snapshot into deterministic, evidence-bearing, profile-isolated reference facts and a typed `wow-store` build plan. It preserves all raw APIDocumentation metadata, lowers supported fields into exact queryable facts, quarantines unknown/unsupported constructs, applies digest-bound reviewed corrections, computes capability/coverage state, publishes one immutable ReferenceStore generation, and exposes a read-only exact `ReferenceView`.

The crate never executes arbitrary Lua and never treats a mirror/provider, annotation projection, community implementation, or fuzzy search result as platform authority.

## E1-B outcome

A future implementation agent must prove:

```text
one exact SourceSnapshotManifest and ReferenceProfile can be validated
all declared APIDocumentation inputs are parsed in deterministic declared order
one restricted evaluator can lower supported declarative tables/calls without executing Lua
all unknown fields and unsupported constructs are preserved with exact source evidence
normalized API/event/table/widget/enum/CVar/predicate/restriction facts are deterministic
reviewed corrections apply only when the expected source digest matches
capability/coverage partitions distinguish Complete/Partial/Failed/NotEvaluated
one static persistent schema/operation/validation bundle is registered with wow-store
one deterministic ReferenceStoreBuildPlan publishes an immutable ReferenceStore
one read-only ReferenceView performs exact lookup and authoritative negative decisions only under complete relevant coverage
profiles and generations never mix
```

## Owned responsibilities

- exact source snapshot/provider provenance manifest validation;
- exact `ReferenceProfile` construction and release/fixture eligibility;
- APIDocumentation file/system/registration ordering;
- source parsing boundary and restricted declarative evaluator semantics;
- canonical raw Lua value preservation;
- normalized reference fact identities and record schemas;
- API systems, functions/methods, tables/structures, events, enums, CVars, widgets/script objects, predicates, deprecations, and restriction facets supported by E1;
- raw unknown-field and unsupported-construct quarantine;
- source spans and stable source-handle construction inputs;
- digest-bound curated corrections and correction-set identity;
- capability/coverage/conflict records and negative-authority prerequisites;
- persistent reference schema/operation/validation bundle owned by `wow-reference` and executed through `wow-store`;
- deterministic reference build plan and object plan;
- immutable ReferenceData generation identity/manifest;
- exact read-only `ReferenceView` and typed lookup/list/raw-metadata/capability operations;
- exact profile/generation isolation;
- build/query budgets, cancellation, and error classification;
- reference fixture/parity inputs and deterministic test corpus.

## Explicit non-responsibilities

E1-B does not:

- acquire/download Blizzard source over the network;
- decide which provider/branch is “current”;
- execute Blizzard Lua or a general Lua interpreter;
- mutate user/editor configuration;
- generate LuaCATS/Emmy annotations (`wow-annotations` owns projections);
- parse/index the complete Blizzard UI graph, TOC/XML object model, function graph, or skeletons beyond exact E1 source/provenance needs;
- index addon projects (`wow-project` owns workspace facts);
- rank fuzzy/semantic search or infer replacements (`wow-search` later);
- call Codebase Memory or external repositories;
- expose SQLite/raw SQL/store connections;
- mutate a sealed ReferenceStore in place;
- convert unknown/partial coverage into absence/safety;
- freeze runtime/hotfix-sensitive spell secrecy as permanent universal truth;
- perform release signing/distribution or CI.

## Required reading

Read in order:

1. [`../../AGENTS.md`](../../AGENTS.md)
2. [`../../DEPENDENCY_GRAPH.md`](../../DEPENDENCY_GRAPH.md)
3. [`../../WORKSTREAMS.md`](../../WORKSTREAMS.md)
4. existing E0 contract files one directory above;
5. [`../../wow-core/CONSUMER_GUIDE.md`](../../wow-core/CONSUMER_GUIDE.md)
6. [`../../wow-store/CONTRACT.json`](../../wow-store/CONTRACT.json)
7. [`AGENTS.md`](AGENTS.md)
8. [`DECISIONS.md`](DECISIONS.md)
9. [`DATA_MODEL.md`](DATA_MODEL.md)
10. [`SOURCE_SNAPSHOT_AND_PROFILES.md`](SOURCE_SNAPSHOT_AND_PROFILES.md)
11. [`APIDOC_EVALUATOR.md`](APIDOC_EVALUATOR.md)
12. [`NORMALIZATION_AND_RAW_METADATA.md`](NORMALIZATION_AND_RAW_METADATA.md)
13. [`CORRECTIONS.md`](CORRECTIONS.md)
14. [`COVERAGE_AND_NEGATIVE_AUTHORITY.md`](COVERAGE_AND_NEGATIVE_AUTHORITY.md)
15. [`STORE_SCHEMA_AND_OPERATIONS.md`](STORE_SCHEMA_AND_OPERATIONS.md)
16. [`BUILD_AND_PUBLICATION.md`](BUILD_AND_PUBLICATION.md)
17. [`REFERENCE_VIEW.md`](REFERENCE_VIEW.md)
18. [`ERROR_MODEL.md`](ERROR_MODEL.md)
19. [`TEST_MATRIX.md`](TEST_MATRIX.md)
20. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
21. [`CONTRACT.json`](CONTRACT.json)
22. current `AGENTS.md` and `INDEX_MINI.md` in the external [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)

Normative repository sources:

- [`../../../docs/REFERENCE_PACK.md`](../../../docs/REFERENCE_PACK.md)
- [`../../../docs/ARCHITECTURE.md`](../../../docs/ARCHITECTURE.md)
- [`../../../docs/PROVENANCE_AND_COVERAGE.md`](../../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../../docs/SECURITY_MODEL.md`](../../../docs/SECURITY_MODEL.md)
- [`../../../docs/TEST_STRATEGY.md`](../../../docs/TEST_STRATEGY.md)

For current live patch/security/source routing, consult the external KB. E1 fixtures pin their own exact snapshot/profile and do not copy floating live guidance.

## Direct framework dependencies

```text
wow-core
wow-store
```

A parser dependency may be a pinned external Rust crate (expected: the exact EmmyLua parser compatibility line selected by the project) but it must not create a framework dependency on `wow-emmy`. The exact parser/version/feature/API probe freezes before code.

## Input topology

```text
MaterializedSourceSnapshot
    SourceSnapshotManifest
    provider provenance
    exact source revision/content digest
    declared flavor/build/Interface
    ordered APIDocumentation input partitions
    generated documentation files
    deprecation/transition inputs
    optional selected resource/index inputs
    source-map entries and licenses

+ CuratedCorrectionSet
+ ReferenceSchema/Operation/Validation bundle
+ Build budgets/policy

-> ReferenceBuild
```

Provider provenance is not authority. The materialized content digest and exact profile define the input.

## E1 source scope

Active E1 partitions:

```text
APIDocumentation systems and functions
structures/tables/fields
systems/tables registered in generated documentation order
events and payloads
enums and selected constants
CVars and metadata where exact source input exists
widgets/script objects/methods where represented by the selected E1 input contract
predicates and restriction/Secret metadata
explicit deprecations/transitions/aliases where source evidence exists
raw unknown fields and unsupported constructs
```

Deferred or partial unless separately activated:

```text
complete FrameXML/UI implementation graph
TOC/XML package/object/inheritance graph
full Lua call graph
source skeleton corpus
historical lineage beyond explicit selected migration/deprecation records
runtime game-data spell secrecy tables
annotation generation/parity output
```

Coverage reports these honestly.

## Restricted evaluator boundary

Allowed declarative semantics include only the exact frozen subset required by the corpus, for example:

```text
literal nil/boolean/number/string
table constructors with bounded array/map fields
local bindings to supported canonical values
field/index access to known local/constants tables
bounded supported constant expressions
known APIDocumentation registration calls
known enum/table reference forms
known helper constructors only after exact semantic contract and tests
```

Forbidden:

```text
arbitrary function execution
loops/recursion/coroutines/metatables
file/network/process/global environment access
load/loadstring/require/dofile
unknown calls with side effects
unbounded computation
runtime/client API calls
```

Unsupported syntax becomes a typed ingestion diagnostic and reduces only the affected capability partition unless completeness dependencies require broader downgrade.

## Raw versus normalized data

```text
raw canonical value/provenance store
    preserves every known and unknown field/value/source span

normalized fact store
    typed queryable projections for supported fields

annotation projection
    separate later wow-annotations output
```

Normalization never destroys raw input. Unknown field names/values remain addressable and affect coverage for dependent consumers.

## Corrections

Corrections are reviewed data:

```text
CuratedCorrection
    correction ID/version
    exact target entity/field path
    expected source/value digest
    replacement canonical value
    evidence/source handles
    rationale/reviewer
    applicability profile/build
```

Mismatch expires/rejects correction. No hidden source-name/product branch. Correction output retains both raw source value and corrected normalized projection with provenance.

## Persistent storage seam

`wow-reference` owns one static schema bundle and prepared operation/validation catalogs; `wow-store` owns application, transactions, file/object lifecycle, validation, sealing, publication, and read-only open.

No raw SQL leaves either crate. Reference adapters encode/decode typed domain records and invoke registered operation IDs.

## Build flow

```text
validate source snapshot/profile/licenses/budgets
-> enumerate deterministic input partitions/files in declared order
-> parse source to syntax facts
-> restricted-evaluate supported registrations/values
-> preserve raw canonical values/unknown fields/unsupported records
-> lower normalized reference facts and source evidence
-> apply digest-bound corrections
-> detect conflicts and compute capability/coverage partitions
-> canonicalize/deduplicate by exact identity
-> build ReferenceStore schema operation plan and object plan
-> ask wow-store to build/validate/seal/publish immutable generation
-> reopen exact published store read-only
-> validate ReferenceView queries, manifests, negative-authority fixtures
-> emit ReferenceDataBuildReport/manifest
```

## ReferenceData artifact boundary

E1-B produces/reference-owns:

```text
ReferenceDataManifest
ReferenceStore generation/manifest reference
raw metadata object set
source-map object set
capability/coverage/correction/conflict manifests
license/provenance manifest
build report/checksums
```

`wow-annotations` later consumes the exact ReferenceView and emits a separate annotation artifact. A higher assembly tool combines them into the final Reference Pack; `wow-reference` does not depend on `wow-annotations`.

## Exact ReferenceView

E1 operations:

```text
get_profile_and_generation
get_capability_and_coverage
lookup_exact_entity
lookup_api_callable
lookup_event
lookup_structure_or_table
lookup_enum_or_cvar
lookup_widget_or_method
lookup_restriction_facets
lookup_deprecation_or_explicit_transition
read_raw_metadata
resolve_reference_source_handle
list_namespace_or_kind_bounded
negative_authority_decision
```

No hidden fuzzy fallback, external search, or replacement inference.

## Negative authority

Authoritative absence requires:

```text
known exact selected profile/reference generation
normalized exact query/entity kind
all relevant declared input partitions ingested
required parser/evaluator/normalizer/correction/store capabilities Complete
no unresolved relevant conflict or unknown dependent field
no truncation/staleness/runtime-only gap
```

Otherwise return partial/failed/conflict/not-evaluated/candidate-only style state, never “does not exist.”

## E1-B hard stops

- No `Cargo.toml` or Rust code in this documentation phase.
- No source acquisition/network.
- No arbitrary Lua execution.
- No hidden source/editor setting mutation.
- No unknown field loss.
- No annotation projection in this crate.
- No full UI graph/skeleton/search implementation.
- No SQLite/raw SQL/storage lifecycle bypass.
- No mixed profile/generation.
- No correction without exact digest/evidence.
- No clean negative from partial/failed/conflicted partitions.
- No runtime spell whitelist/generalized safety claim.
- No in-place sealed ReferenceStore mutation.
- No CI/release automation.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- source snapshot/profile manifest;
- evaluator supported/unsupported/security cases;
- raw/normalized/unknown/conflict facts;
- corrections and expiry;
- persistent schema/operation/build plan;
- exact ReferenceView lookup/negative-authority cases;
- checksum freeze.

Exact parser/source/schema/correction/generation/store/fact/query/report IDs and SHA-256 values freeze before the first E1-B Rust commit.

## Definition of done

E1-B implementation is complete only when:

```text
one exact source snapshot/profile ingests without arbitrary Lua execution
all declared inputs are ingested or explicitly diagnosed and partitioned
raw unknown metadata round-trips exactly
normalized supported facts and source evidence are deterministic
corrections apply/expire only through exact digest-bound rules
coverage/conflicts gate negative authority correctly
one static reference schema/operation/validation bundle builds through wow-store
one immutable ReferenceStore generation publishes and reopens read-only
one exact ReferenceView passes positive/authoritative-negative/partial/conflict/profile-isolation tests
no annotation/search/UI-graph/runtime-whitelist/storage-bypass behavior exists
all TEST_MATRIX cases pass
```

Until then, `e1/` is an implementation-ready persistent reference contract, not a full Reference Pack builder.
