# Native platform direct producer plan

`build_platform_graph_proposal_plan` projects one held platform `ProjectView`
into four ordered captured-direct stages and, when selected, the existing raw
inventory prelude. It finishes against the exact admitted native graph owner.
This is a library API; additive native recognizer adapters consume its exact
addresses while service and replay publication retain their existing layouts.
The native cases and final workspace gates/build pass on
2026-10-10. Full W17/E3 acceptance remains open.

## Native inputs and API

The APIs are exported from `wow_project::graph`. Construction requires a genuine
`ProjectKind::BlizzardUiPlatformSource` with an existing selected platform graph
profile. The common collector validates the held project, configuration,
packages, retained load plans and same-session analyzer reports. Ordinary or
unselected projects return `DeferredCapability`. No new configuration selector
or serialized admission flag is introduced.

The plan's separate library profile is
`wow-project/platform-direct-producers/1`. `PlatformGraphProposalPlan` exposes
`profile()`, `registry()`, `scope()`, `limits()`, `foundation()`,
`raw_inventory_batch()`, `raw_inventory_producer_version()` and
`producer_order()`. One shared projection supplies either this plan or the
existing flattened builder; the plan adds no parser, analyzer pass or temporary
ProjectView.

| Order | Producer and partition | Currently emitted responsibility |
| --- | --- | --- |
| Optional prelude | Original raw Inventory capability: `wow-project.platform-source-inventory` | Declared Included raw members; its existing exact batch and version remain separate from captured files. |
| 1 | `Inventory`: `wow-project.platform-inventory` | Captured files, packages and package containment. |
| 2 | `TocLoad`: `wow-project.platform-toc-load` | Currently emitted dependencies/loads and TOC state roots/membership. |
| 3 | `AnalyzerStructure`: `wow-project.platform-analyzer-structure` | Actual Lua declarations/functions/state paths and their owned membership. |
| 4 | `XmlStructure`: `wow-project.platform-xml-structure` | Captured XML declarations/handlers, inheritance, ownership and explicit XML mixin relations. |

Ownership is tagged at proposal creation and checked against producer
permissions. A declaration discovered while processing XML remains
AnalyzerStructure-owned. Package-qualified paths, native XML occurrence IDs,
Main receipts, ambiguity and omissions retain their original meaning. Four
classes cover these existing assertions, not every normative E3 direct role.

## Ordered native graph admission

1. Construct the plan from the held project. Create a
   `GraphPartitionSnapshot` with its `registry().clone()`,
   `foundation().clone()` and `scope().source_context_id`.
2. If `raw_inventory_batch()` is present, admit that exact batch first with
   `raw_inventory_producer_version()` and empty raw coverage.
3. Follow `producer_order()`. Call `build_stage(producer, &owner, stop)` against
   the actual admitted predecessor. The result is
   `PlatformGraphProducerProposals`; retain `producer_version()` before
   consuming `into_parts()` for its batch and coverage.
4. Admit each stage through `prepare_replacement` with the exact snapshot and
   previous partition digest. Use the resulting native candidate, or publish it
   through a `GraphPartitionSession`, before building the next stage.
5. Call the consuming `finish(&owner, stop)` only after all four stages are
   admitted. Retain the project and graph for the returned borrowed capability.

Dependent stages are sequential. `prepare_replacements` admits independent
replacement sets against one predecessor; it is not a dependency scheduler.
See [native partition usage](../crates/wow-graph/PARTITION_USAGE.md).

A constructed stage remains pending until its exact native admission. Retrying
the last pending stage returns the same batch; construction alone cannot advance
the plan. Batch, producer version, coverage and full accepted accounting must
match. Cross-stage endpoints resolve exact native Producer references and use
accepted **input-generation** IDs. Final materialized node IDs are not inputs.
Derivations retain Local references within a stage and exact Producer references
to admitted earlier stages.

Finish checks every actual partition. It permits precisely the four stages and
the selected raw prelude, rejecting unrelated partitions/tombstones and a
monolithic overlay. Wrong scope, order, predecessor version or incomplete
admission also refuses. Shared metadata accounting covers the collected
projection, stages, retries, address crosswalk and evidence catalog; inherited
graph/source limits and cooperative cancellation remain required. Failure
cannot produce a finished capability.

## Exact assertion and evidence capability

`finish` returns `PlatformGraphProvenance`, which borrows the original project and
graph. Its getters are `profile()`, `project()`, `graph()`, `scope()`, `source()`,
`assertion(&GraphLocalAssertion)` and `evidence_catalog()`.

`assertion()` maps an original entity/relation proposal key to its exact admitted
Producer address. Finish resolves proposals and accepted receipts, then joins
captured-file support and optional raw-member support to the held native source
handles and evidence. Raw membership remains distinct from captured load/Main
membership. Original source metadata is retained by `source()`; that metadata
alone does not reconstruct this native capability.

The plan and finished capability have private fields and no Deserialize route.
Serializable proposals or metadata do not establish native owner authority.
Evidence closure preserves confidence, coverage, omissions and ambiguity; it
does not establish configured-root completeness, origin authenticity, license
permission, successful loading or runtime behavior. Original inventory remains
Partial. This API adds no byte excerpt/export transport or source acquisition.

## Compatibility and remaining work

`build_source_graph_proposals` still flattens the common projection into its
existing captured-source batch. Source `/19`, package `/20` and raw `/21`
builders retain their recipes. The new plan reuses their genuine input scope,
registry and source context; its different partition layout determines a
different materialized graph. Reusing input scope does not select that layout in
old service or replay channels.

Configuration selectors, service request/result profiles, legacy recognizer
entry points, ordinary replay v1-v4, platform replay v5-v8 and storage/catalog/epoch recipes
remain unchanged. Published package/raw application routes retain their
monolithic captured producer. Their recognizer partitions remain separate. See
[package graph](PLATFORM_PACKAGE_GRAPH.md) and
[raw inventory graph](PLATFORM_RAW_INVENTORY_GRAPH.md).

Additive [native recognizer assertion APIs](PLATFORM_RECOGNIZER_ASSERTIONS.md)
consume this plan's exact Producer addresses for TOC/XML/scripts/state access
and core Read/Write. Finish the direct graph before extending it with recognizers.
Scripts retain endpoint references in their recognition envelope; state/core
retain actual graph derivation records. This native caller composition does not
select the split in configuration, service publication or replay.

Configuration/service/replay selection and remaining recognizer routes, missing
selected-TOC/variant/load-unit, inventory/source-span and XML object/region/parent/span roles,
structural fingerprints, bounded `SkeletonInputView` and source service/CLI
transport remain open. Full W17/E3/package, real Gethe/Ketho corpus, performance,
native fault/exhaustion and WoW runtime acceptance remain NotEvaluated. Gethe
materialization stays deferred; W18-W26 are unchanged.

## Scoped verification

The single [native synthetic test](../crates/wow-project/tests/platform_direct_producers.rs)
`direct_package_stages_bind_native_predecessors_and_preserve_replay` passes. With
source disk absent, it checks four-stage admission, exact input endpoints and
derivations, source-handle/evidence catalog equality, repeated package-local XML
identity, and wrong-scope/order/pending/version/overlay/surplus refusals. The same
fixture also finishes raw prelude plus four stages and verifies original raw
Producer addresses and catalog support. Existing `/6` and `/7` replay hydration
and unchanged monolithic payloads are checked; `/8` raw replay capture and its
monolithic/raw payloads remain unchanged.

The native assertion chain additionally passes with disk absent, exact support
and native endpoint checks, genuine substitution refusal and pre-set cancellation;
see its [scoped verification](PLATFORM_RECOGNIZER_ASSERTIONS.md#scoped-verification-and-remaining-work).

Final workspace gates/build completed at **2026-10-10 09:19:03 UTC**:
`cargo xtask check`, `cargo fmt --all --check`, workspace all-target/all-feature
check, strict Clippy, tests (**941 passed, 0 failed, 1 ignored, 112 targets**),
strict rustdoc under `RUSTDOCFLAGS=-D warnings`, and workspace
all-target/all-feature build all passed. These are local scoped results;
remote publication and CI are not asserted.
