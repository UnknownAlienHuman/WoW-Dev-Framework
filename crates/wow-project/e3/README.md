# `wow-project` E3-B pinned platform UI source producer

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `wow-project/e3-b/pinned-platform-ui-source-corpus`

## Mission

Turn one closed, provider-labeled, exact materialized World of Warcraft UI source snapshot into a separately identified `pinned_platform_ui_source` corpus with package/TOC/XML/Lua inventory, static load structure, analyzer facts, recognizer proposals, graph assertions, source handles, coverage, license state, and an exact published read view consumable by E3-A context.

```text
MaterializedPlatformSourceSnapshot
+ exact flavor/build/profile and corpus policy
+ pinned parser/analyzer/recognizer/graph/store contracts
-> validate provider provenance, build observations, roots, bytes, paths, license, and security
-> classify implementation, generated API documentation, inventory-hint, and ignored metadata roles
-> inventory all selected UI packages without executing source or repository automation
-> parse selected TOCs/XML through wow-project E2-C contracts
-> build the global static package/file/load model
-> materialize physical and XML-virtual Lua units
-> analyze one logical platform-source workspace through wow-emmy
-> adapt normalized facts and run universal E2-B recognizers
-> validate graph proposals under the dedicated source universe
-> assemble a deterministic PlatformSourceCorpusCandidate
-> publish a separate coherent ProjectStore/GraphSnapshot generation through E2-D
-> expose an exact PlatformSourceCorpusView and SourceUniverseManifest
```

## Separation from E3-A and Reference Pack

- `wow-context` consumes an already published exact view; it never acquires or indexes source.
- `wow-reference` owns generated API documentation evaluation and API/restriction authority.
- E3-B owns implementation-source structure and source pointers.
- Generated API documentation files in the mirror are classified separately and are not silently treated as implementation code or re-parsed as API authority by E3-B.
- A source call/use and a Reference Pack entity can be linked only by an explicit evidence-bearing graph relation; neither record overwrites the other.

## Direct dependencies

```text
wow-core
wow-store
wow-emmy
wow-graph
wow-recognizers
```

The existing `wow-project` dependency set is sufficient. No direct dependency on `wow-reference`, `wow-context`, `wow-search`, `wow-rules`, or network libraries is activated in the library contract.

## Acquisition boundary

E3-B consumes only [`MaterializedPlatformSourceSnapshot`](MATERIALIZED_SOURCE_INPUT.md). Network/repository acquisition is owned by [`../../../tools/wow-ui-source-materializer/`](../../../tools/wow-ui-source-materializer/README.md) or another reviewed materializer implementing the same output contract.

The project library never:

- resolves floating branches/tags/releases;
- invokes Git, GitHub, CASC, package managers, installers, hooks, filters, workflows, or source tools;
- follows unreviewed symlinks/reparse points/submodules/LFS filters;
- downloads dependencies or missing files;
- executes Lua, XML, TOC, generated source, or repository automation.

## Initial pinned fixture

```text
provider trust class: pinned-community-mirror
repository: Gethe/wow-ui-source
branch label: live (provenance only)
commit: 027d26c3406d3de2cbd2b1f67d468fe033a1bcd4
git tree: b95256b3ebce23fbbef3603d0b5550f7d90cd013
version.txt: 12.1.0.69497
Interface tree: 3463d02eb9687da15ad17477fef5823e9628ff43
Interface/AddOns tree: cbe625ad3120fc3ee178d163012b0c62e9361e92
ui-code-list blob: b1ea1dd8389edb3cc298750cff60ffff7d13f79b
ui-toc-list blob: 3d438f56d9bfc00e4d72986b2fd72cfe776b17a5
```

This is a documentation/implementation fixture, not a floating production default. The exact materialized content manifest and SHA-256 remain to be produced by the materializer implementation.

## Required reading

1. repository and `crates/` agent instructions;
2. E2-C `wow-project` package and E2-D publication boundary;
3. E2-A graph and E2-B recognizer contracts;
4. E3-A context input/universe contract;
5. every E3-B file in this package;
6. current KB `AGENTS.md`, `INDEX_MINI.md`, `BlizzardUI_DevWorkflow.md`, subsystem router, TOC/XML/lifecycle/security routes;
7. the exact materializer contract and provider fixture.

## Active E3-B operations

```text
validate_platform_source_request
validate_materialized_platform_source_snapshot
classify_platform_source_roles
build_platform_source_package_inventory
select_platform_source_toc_variants
build_platform_source_global_load_model
build_platform_source_analyzer_plan
validate_platform_source_analyzer_snapshot
build_platform_source_recognizer_fact_bundles
validate_platform_source_graph_proposals
plan_platform_source_incremental_update
assemble_platform_source_corpus_candidate
validate_platform_source_corpus_candidate
build_platform_source_publication_bundle
validate_published_platform_source_corpus
open_platform_source_corpus_view
```

## Explicit non-responsibilities

E3-B does not:

- declare a mirror official Blizzard provenance;
- use repository branch labels as immutable identity;
- infer exact client installation/runtime state from source version metadata;
- evaluate generated APIDocumentation into Reference Pack authority;
- prove API legality, event payload readability, Secret state, taint, combat, protected, forbidden, managed-object, performance, or runtime behavior;
- create search ranking, lineage, patch impact, diagnostics, fixes, context prose, or model summaries;
- redistribute source where license/policy does not permit it;
- add CI.

## Completion gate

E3-B code is complete only when an exact materializer snapshot can be validated without executing repository/source content; provider/build/license conflicts remain visible; selected package/TOC/XML/Lua inventories and global load facts are deterministic; generated API docs remain segregated; analyzer coverage and any sharding loss are explicit; package/file/recognizer/graph partitions replace atomically; removed source has no target facts or handles; corpus publication remains separate from the user's addon ProjectStore; E3-A can bind the exact auxiliary universe without name/path joins; patch updates reuse only exact unchanged partitions; 1/2/N workers and shuffled acquisition/parse/analyzer completion produce byte-identical logical manifests; and all closed fixtures/checksums pass.
