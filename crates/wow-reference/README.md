# `wow-reference` implementation contract

**Status:** E0 fixture slice active; full implementation deferred to E1.

## Mission

`wow-reference` owns the canonical normalized representation of one exact Blizzard API/UI reference profile and the build/read pipeline for immutable Reference Packs. It converts pinned source inputs into evidence-bearing facts while preserving unknown metadata and explicit coverage gaps.

## Owned responsibilities

- Reference Pack manifest and profile identity;
- input inventory and provenance;
- restricted declarative APIDocumentation evaluation;
- raw canonical Lua value preservation;
- schema-aware lowering of systems, functions, methods, tables, events, enums, CVars, widgets, predicates, and restriction metadata;
- digest-bound curated corrections;
- exact `ReferenceView` lookup interfaces;
- source maps and capability/coverage report;
- pack logical assembly and validation coordination;
- profile isolation and selected historical presence/migration inputs;
- differential input records from Ketho/Numy without treating them as canonical.

## Explicit non-responsibilities

`wow-reference` does not:

- generate editor annotation text; that belongs to `wow-annotations`;
- analyze addon project files; that belongs to `wow-project`/`wow-emmy`;
- own graph search/ranking;
- execute arbitrary Lua or Blizzard UI code;
- download a floating "live" profile without a pinned manifest;
- hard-code current spell secrecy as permanent source truth;
- silently repair unknown fields or parser gaps;
- treat an acquisition mirror as platform authority.

## Input classes

```text
pinned Blizzard UI snapshot
Blizzard_APIDocumentation
Blizzard_APIDocumentationGenerated in declared TOC order
Blizzard_Deprecated / transition records
Interface/AddOns TOC/XML/Lua source
optional interface resource metadata
reviewed correction records
Ketho/Numy outputs for differential comparison
```

Every input has provider, revision/build, normalized path, digest, and license/provenance metadata.

## Internal responsibility slices

### Manifest/profile

Build exact profile identity, schema/tool versions, input digests, correction-set digest, capability report, and pack artifact inventory.

### Declarative evaluator

Allow only documented literal/table/local-binding/known-registration/bounded-expression forms. Unsupported calls, loops, IO, dynamic loading, metaprogramming, or side effects are rejected or quarantined.

### Raw value model

Preserve all known and unknown fields in a canonical value tree before lowering. Raw metadata remains retrievable even when no public typed field exists yet.

### Lowering

Convert supported raw records into normalized domain facts with source spans, producer identity, profile applicability, and restriction facets.

### Corrections

Apply only reviewed corrections whose expected source digest matches. Expired corrections become explicit validation failures/gaps.

### Reference view

Expose narrow exact read operations needed by rules, search, annotations, and context. Consumers never query internal SQLite tables directly.

## Required operations

| Operation | Required behavior |
|---|---|
| `inventory_reference_inputs` | Produce the exact ordered input set and identify missing/duplicate/ambiguous partitions. |
| `evaluate_apidoc_file` | Evaluate only the allow-listed declarative subset and preserve unsupported constructs as diagnostics. |
| `lower_raw_apidoc` | Convert raw canonical values to typed facts without dropping unknown fields. |
| `extract_package_inputs` | Record TOC/XML/Lua package/source inventory for later structural extraction. |
| `apply_curated_corrections` | Require target, field, source digest, evidence, reviewer, and deterministic ordering. |
| `build_capability_report` | Mark each partition Complete/Partial/Unknown/Failed with reasons. |
| `assemble_reference_model` | Produce one profile-isolated normalized model from validated partitions. |
| `open_reference_view` | Return a generation/profile-bound read interface. |
| `lookup_symbol_exact` | Resolve exact canonical/alias identity with evidence and negative-authority status. |
| `lookup_event_or_widget_contract` | Return signature/payload/member facts and coverage, never a bare optional. |
| `lookup_restriction_facets` | Return raw and normalized facets with build applicability and unknown-field status. |
| `resolve_reference_source_handle` | Map an entity/fact to a stable source handle within the pack/source map. |
| `validate_reference_pack` | Verify profile/schema/digests/coverage/artifact consistency. |
| `compare_logical_pack_inputs` | Support provider/differential comparisons without changing canonical facts automatically. |

## E0 fixture slice

E0 implements only:

- one explicit non-floating fixture profile;
- one small normalized APIDocumentation fixture;
- exact lookup for a known valid API and an absent API;
- one producer fact carrying a Secret/restriction facet needed by the local rule;
- complete and partial capability variants;
- an in-memory or fixture-file `ReferenceView`;
- deterministic fixture serialization.

E0 must **not** implement:

- full Blizzard snapshot ingestion;
- SQLite pack storage;
- acquisition/download;
- correction engine beyond a fixture stub with no fake success path;
- annotation generation;
- profile lineage;
- Ketho/Numy execution.

## Full E1 rules

1. Arbitrary Lua is never executed.
2. Unknown fields are round-tripped and downgrade only dependent capabilities.
3. Generated docs are read in declared TOC order.
4. Duplicate/conflicting registrations are retained and classified.
5. A failed partition does not invalidate unrelated complete systems.
6. PTR/beta/current/historical profiles remain physically and logically separate.
7. An exact miss is authoritative only under complete relevant coverage.
8. Corrections expire on source digest change.
9. Pack output is deterministic across worker counts and input enumeration order.
10. Provider identity is provenance; logical content digest defines the snapshot.

## Required tests

### E0

- known API exact hit;
- authoritative miss under complete coverage;
- non-authoritative miss under partial coverage;
- Secret facet retrieval;
- profile mismatch rejection;
- deterministic fixture output.

### E1

- allow-listed evaluator positives and rejected arbitrary execution;
- nested unknown field preservation;
- duplicate/conflicting registration;
- correction apply/expiry;
- TOC order sensitivity;
- partial partition isolation;
- provider-equivalent logical output;
- pack checksum/manifest validation;
- semantic annotation input completeness;
- profile isolation matrix.

## Documentation sources

- [`../../docs/REFERENCE_PACK.md`](../../docs/REFERENCE_PACK.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECRET_VALUES_AND_RESTRICTIONS.md`](../../docs/SECRET_VALUES_AND_RESTRICTIONS.md)
- [`../../docs/RESEARCH_BASELINE.md`](../../docs/RESEARCH_BASELINE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [Current WoW KB agent rules](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md)
- [Current Blizzard subsystem/source router](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_SubsystemRouter.md)
- [Current Secret/taint guidance](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/KB/core/BlizzardUI_security.md)

## Definition of done

The E0 slice is complete when rules can query one exact fixture profile with honest complete/partial negative authority. Full E1 is complete only when a pack can be rebuilt deterministically from pinned inputs, unknown metadata survives, arbitrary Lua cannot execute, and consumers never need to inspect builder internals.
