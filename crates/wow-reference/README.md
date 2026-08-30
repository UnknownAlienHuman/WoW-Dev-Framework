# `wow-reference` implementation contract

**Status:** E0-B implementation-ready contract; no Rust code yet. Full Reference Pack production remains an E1 responsibility.

## Mission

`wow-reference` owns the canonical, profile-bound view of World of Warcraft platform facts. It turns pinned reference inputs into exact normalized facts, preserves raw and unknown metadata, reports partition coverage honestly, and exposes narrow read operations to rules and later query layers.

The E0-B slice proves only one deterministic fixture-backed `ReferenceView`. It does not attempt to ingest the complete Blizzard UI tree or build a distributable Reference Pack.

## E0-B outcome

A future implementation agent must be able to implement this exact vertical seam:

```text
closed fixture bundle
    -> validate fixture profile and ordered inputs
    -> parse restricted APIDocumentation-shaped records
    -> preserve raw canonical values
    -> lower two fixture API symbols and one restriction facet
    -> publish one immutable ReferenceView
    -> exact lookup hit / authoritative miss / partial miss / conflict
    -> evidence, coverage, and negative-authority outputs using wow-core
```

The fixture is synthetic and minimized, but it is bound to an explicit Retail build context. It is not a release-grade profile and must never be presented as one.

## Owned responsibilities

- fixture and release profile identity on the reference side;
- ordered reference-input inventory and content identity;
- restricted declarative APIDocumentation evaluation;
- raw canonical Lua-value preservation;
- schema-aware lowering into normalized symbols and restriction facets;
- duplicate and conflicting registration classification;
- exact `ReferenceView` operations;
- reference-side source handles and evidence records;
- per-producer, per-partition capability coverage;
- negative-authority decisions for exact misses;
- deterministic fixture/model serialization;
- E1 pack assembly, corrections, source maps, checksums, and validation when that milestone activates.

## Explicit non-responsibilities

`wow-reference` does not:

- parse or index addon project files;
- create project source handles or diagnostic findings;
- generate LuaCATS/Emmy annotation text;
- rank search candidates or infer replacements from similarity;
- execute arbitrary Lua, build scripts, repository hooks, or Blizzard UI code;
- download a floating `latest` profile;
- hard-code hotfix-sensitive spell secrecy as permanent truth;
- persist E0 data in SQLite;
- hide unknown fields, unsupported constructs, duplicates, or conflicts;
- summarize an entire operation's coverage for `wow-service`;
- attach platform evidence to an addon source location.

## Required reading

Before implementing this slice, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`DECISIONS.md`](DECISIONS.md)
6. [`DATA_MODEL.md`](DATA_MODEL.md)
7. [`OPERATIONS.md`](OPERATIONS.md)
8. [`LOOKUP_AND_COVERAGE.md`](LOOKUP_AND_COVERAGE.md)
9. [`FIXTURE_PROFILE.md`](FIXTURE_PROFILE.md)
10. [`ERROR_MODEL.md`](ERROR_MODEL.md)
11. [`TEST_MATRIX.md`](TEST_MATRIX.md)
12. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
13. [`CONTRACT.json`](CONTRACT.json)
14. current routes in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb)

Normative repository sources:

- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)

## E0-B fixture profile

The contract uses one closed fixture identity:

```text
profile_id: fixture-retail-120100-e0-v1
profile_kind: fixture
flavor: retail
interface: 120100
client_build: 12.1.0.69497
source_revision: 027d26c3406d3de2cbd2b1f67d468fe033a1bcd4
fixture_catalog: C_E0Fixture
```

The source revision/build are provenance context for the minimized fixture. The fixture contents are project-owned synthetic records. This profile cannot be loaded where a release-grade profile is required.

The fixture defines:

```text
C_E0Fixture.KnownApi
    ordinary function used for an exact lookup hit

C_E0Fixture.SecretText
    function with one first-class secret.return facet

C_E0Fixture.RemovedApi
    intentionally absent query key used for exact miss cases
```

See [`FIXTURE_PROFILE.md`](FIXTURE_PROFILE.md) and [`examples/fixture-bundle.json`](examples/fixture-bundle.json).

## Reference pipeline

```text
ReferenceInputInventory
    -> RestrictedEvaluationResult
    -> RawCanonicalRecord set
    -> LoweringResult
    -> ReferenceModel
    -> ReferenceView
```

Every transition retains:

- one exact profile/reference generation;
- ordered input identity and digest;
- producer identity and version;
- source handles for accepted and rejected records;
- raw unknown fields;
- diagnostics or typed gaps for unsupported constructs;
- exact capability/partition coverage;
- conflicts without silent winner selection.

## Restricted evaluator boundary

E0-B supports only the fixture subset needed to prove the seam:

```text
nil / boolean / integer-safe number / string literals
array and keyed table constructors
local bindings to supported canonical values
field access to known local constants
one allow-listed documentation registration call shape
bounded deterministic expressions explicitly listed in the fixture contract
```

It rejects or quarantines:

```text
arbitrary calls
IO or environment access
dynamic loading
metatable execution
loops or recursion
function bodies with side effects
unknown registration targets
unbounded expressions
```

An unsupported construct affects only the declared partition unless it prevents profile/model identity from being established.

## Exact lookup contract

`ReferenceView` exposes exact operations only in E0-B:

```text
profile_identity()
reference_generation()
capability_records(capability, partitions)
lookup_symbol_exact(entity_key)
lookup_restriction_facets(entity_key)
resolve_reference_source_handle(record_or_entity)
```

An exact lookup returns a typed result, never a bare optional:

```text
Found
AuthoritativeAbsent
AbsentWithoutAuthority
Conflict
ProfileMismatch
CapabilityUnavailable
```

`AuthoritativeAbsent` requires complete relevant coverage, the selected profile/reference generation, no unresolved conflict for the query domain, and no truncation blocker.

No alias, fuzzy, lineage, semantic, or replacement fallback is hidden inside `lookup_symbol_exact`.

## Restriction facet contract

`secret.return` is first-class fixture data, not annotation prose. The normalized facet retains:

```text
facet kind
subject entity
return slot/index
applicability
raw canonical source value
evidence/source handle
producer and generation
coverage partition/status
unknown sibling fields
```

The E0-B fixture uses an unconditional synthetic facet solely to exercise the local Secret rule seam. It does not assert a permanent real-game spell or API classification.

## Coverage variants

The same logical fixture has three declared variants:

### `complete`

- input inventory complete;
- supported records evaluated;
- symbol and restriction partitions complete;
- no unresolved conflict;
- exact miss may be authoritative.

### `partial`

- catalog remains readable;
- one declared input/construct is unsupported or omitted;
- affected symbol partition is `Partial`;
- exact miss is non-authoritative;
- unaffected complete partition remains usable.

### `conflict`

- declared records are fully read;
- two source records disagree on the same normalized contract;
- source ingestion may remain `Complete`;
- the affected capability is blocked by an explicit conflict;
- dependent evaluation is `NotEvaluated` or returns `Conflict`.

Complete ingestion is not permission to ignore a conflict.

## Public operation inventory

Concrete Rust names may change only with a matching contract update. Required semantics are defined in [`OPERATIONS.md`](OPERATIONS.md):

```text
inventory_fixture_inputs
validate_fixture_profile
parse_raw_value
restricted_evaluate_fixture
preserve_unknown_fields
lower_system_record
lower_function_record
lower_restriction_facets
classify_duplicate_registration
assemble_fixture_reference_model
open_reference_view
lookup_symbol_exact
lookup_restriction_facets
resolve_reference_source_handle
build_reference_coverage_records
validate_reference_model
canonicalize_fixture_bundle
```

## Failure behavior

Reference errors are typed and partition-aware. Key classes include:

```text
fixture_profile_invalid
reference_generation_mismatch
input_inventory_invalid
input_digest_mismatch
unsupported_declarative_construct
registration_shape_invalid
raw_value_invalid
lowering_contract_invalid
duplicate_registration_conflict
unknown_field_preserved
reference_model_invalid
reference_view_profile_mismatch
lookup_capability_unavailable
lookup_conflict
negative_authority_unavailable
```

`unknown_field_preserved` is normally a structured gap/notice, not data loss or an automatic fatal error.

## E0-B hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No full Blizzard snapshot ingestion.
- No SQLite or object store.
- No annotation generation.
- No network/download logic.
- No correction engine pretending to succeed.
- No aliases, migration, lineage, fuzzy matching, or replacement recommendation.
- No runtime spell whitelist.
- No platform evidence attached to addon/project spans.
- No clean miss under partial/failed/unknown/conflicted coverage.
- No current/latest profile inference.

## Required fixtures

The closed bundle under [`examples/`](examples/README.md) includes:

- exact fixture profile and input inventory;
- complete catalog/model;
- partial coverage variant;
- conflicting restriction-facet variant;
- exact hit;
- authoritative exact miss;
- non-authoritative partial miss;
- restriction-facet lookup;
- checksums over every normative fixture file.

Fixtures are data contracts. An implementation must not rewrite them merely to match an easier internal representation.

## Definition of done

E0-B is implementation-complete only when:

```text
one exact fixture profile validates as fixture-only
ordered input and content identities are deterministic
raw values and unknown fields round-trip
KnownApi resolves exactly
RemovedApi is authoritative absent only in complete coverage
partial and conflict variants never produce clean absence
SecretText returns the exact secret.return facet
reference-side evidence never claims a project source location
all records share one profile/reference generation
randomized input enumeration produces identical canonical model bytes
all TEST_MATRIX cases pass
no arbitrary source code executes
```

Until Rust implementation exists, this directory remains an implementation-ready contract pack, not a functioning Reference Pack builder.
