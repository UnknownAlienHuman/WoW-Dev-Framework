# E3-B incremental invalidation, profile versioning and source updates

**Status:** normative.

## Base and target

Every update names exact:

```text
base CurrentBlizzardUiSourcePublicationRecord / StoreGeneration: optional
base source profile/generation/snapshot
base analyzer/recognizer/graph/reference-bridge manifests
target source profile/materialized snapshot/build binding/license state
target parser/analyzer/recognizer/graph/store profiles
```

No update starts from a floating branch or “whatever is currently checked out.”

## Change graph

```text
source provider/revision/content/root/file/license/build/profile input
-> source inventory partition
-> package/TOC/XML/load partition
-> physical/virtual Lua unit
-> analyzer fact/finding/source-map partition
-> fact adapter partition
-> recognizer output partition
-> direct source graph proposal partition
-> reference/source bridge partition
-> graph generation/candidate/publication set
-> store generation/current record
```

Dependencies are explicit and canonical.

## Diff basis

Use exact logical final-state diff:

- root definition and admitted manifest;
- normalized logical path/file kind/content digest;
- TOC/XML/load normalized records;
- Lua unit bytes/source map/profile;
- analyzer/adapter/recognizer/graph/bridge profile IDs;
- build-binding evidence/state;
- license/redistribution evidence/decision;
- capability/coverage/conflict state.

Mtime, provider branch movement, checkout status, watcher event order and cache metadata are not sufficient.

## Source file changes

### Lua file

Invalidate:

- file/source handle assertion;
- containing package/global load partitions affected by order/reachability;
- analyzer unit and dependent semantic fact/finding partitions according to exact analyzer dependency report;
- adapters/recognizers consuming changed facts;
- direct source graph entities/relations for changed declarations/references;
- reference/source bridges using changed source facts/endpoints;
- context/search partitions that declare those exact dependencies later.

### XML file

Invalidate exact XML parse/include/template/object/inheritance/script records, virtual/external Lua units and downstream load/analyzer/recognizer/graph/bridge partitions. Include impact follows bounded direct dependency closure.

### TOC file

Invalidate selected variant, package metadata, dependency/file order/load/SavedVariables declarations and all downstream file reachability/workspace/recognizer/graph partitions affected. Variant/profile change cannot reuse facts from the old variant to fill gaps.

### Nonsemantic inventory/license file

A license/notice change can leave source semantic facts reusable but invalidates license/redistribution decisions, notice manifests, publication eligibility and every externally rendered/packaged artifact that depends on them.

### Unknown/binary/unsupported file

Content or classification change invalidates inventory/coverage/license state and any later adapter partition. It cannot silently stay ignored if the target profile expects coverage.

## Root/package profile changes

Changing root roles, inclusion/exclusion, package boundaries, file-kind policy or logical path normalization can re-key a broad source set. Invalidate the complete affected root or source collection unless a versioned migration proves exact stable identities.

No automatic reinterpretation of existing rows under a new identity schema.

## Build-binding changes

Build-binding evidence/state change invalidates:

- source publication eligibility and current selector mapping;
- reference compatibility and all bridge partitions;
- context/search cross-universe availability;
- release/channel decisions that depend on exact build matching.

It does not change source bytes or source entity keys if the source generation's binding identity model explicitly keeps content and binding records separate; it creates a new source profile/generation/publication decision as defined by the frozen identity schema. No in-place relabel.

## Reference profile changes

A new reference generation/profile invalidates reference/source bridge partitions and any graph capability summaries that bind them. Source-internal graph partitions can be reused under exact proof.

Never combine old reference facts and new source facts in one bridge partition.

## Analyzer update

Changing Emmy implementation/pin/config/annotation library/fact schema invalidates analyzer facts/findings/source maps and all dependent adapters/recognizers/source graph/bridges. TOC/XML/source inventory may remain reusable if exact dependencies match.

## Recognizer update

Changing pack/rule/version/budget/evaluation profile invalidates only its producer partitions and dependent graph/source summaries. Direct source facts remain. Disabling a rule removes its assertions and reduces coverage.

## Graph registry or bridge-rule change

Changing entity/relation/attribute/axis/bridge schemas invalidates affected proposal validation, semantic keys/assertion IDs, conflicts, graph generations, bridges and downstream views. A breaking identity change requires a new registry/profile/version and usually a new store epoch.

## Store/schema/profile change

E2-D rules apply:

- compatible logical schema/operation changes create new source store generations;
- breaking physical/runtime/schema profile changes create a new epoch;
- prior generations keep original identities;
- no in-place semantic reinterpretation.

## Removal closure

For every removed file/root/package/entity/relation/bridge, target state must contain no stale:

```text
source inventory or source handle
TOC/XML/load record
physical/virtual analyzer unit
analyzer fact/finding/source map
adapter fact
recognizer match/proposal/output partition
graph entity/relation/assertion/index/conflict reference
reference/source bridge or ambiguity group
source candidate/publication/map/context/search reference
object/member/license/notice reference
continuation/query handle targeting the removed entity
```

Reverse-dependency tests verify all classes.

## Reuse proof

```text
BlizzardUiSourceReuseProof
    exact base/target profile/generation contexts
    partition key and prior manifest
    identical decisive source/profile/tool/rule/registry dependencies and digests
    identical source handles/maps and endpoint identities
    compatible coverage/conflict/build/license state
    no removal/tombstone dependency
    validation result
```

Same path/name/provider revision without this proof is insufficient.

## No-change

Return canonical `NoChange` only when final source profile/snapshot/content, all semantic/tool profiles, build/license/coverage/conflict states and every logical output partition are equivalent. It creates no new semantic generation or current record.

Operational re-download/re-materialization of identical sealed bytes may produce an operational report without a new semantic source generation.

## Update-order determinism

Independent event sequences reaching the same target sealed snapshot/profile state produce the same:

- source generation;
- package/load/analyzer/fact/recognizer manifests;
- source and bridge graph proposals/assertions;
- candidate/publication/store semantic IDs;
- current-record target contents;
- allowed exported artifact bytes.

## Current selectors and multiple builds

Multiple exact source publications may coexist. Current is selected per explicit build/profile/channel selector. Updating build B cannot replace current for build A.

Retention and GC preserve current/last-known-good/readers/evidence/debug/recovery/license-notice dependencies for each selector.

## Lineage

E3-B records exact generation-scoped source entities and changed/removed manifests. It does not infer cross-build lineage. E4 can consume:

- exact file/entity keys by generation;
- source spans/content/shape signatures;
- graph neighborhoods;
- provider revision relationships;
- build/profile identities;
- add/remove/change evidence.

Lineage output remains a separate relation universe/confidence and cannot rewrite E3-B keys.

## Failure

A failed target update:

- retains target source/profile/build/license identities in the failure record;
- leaves current unchanged;
- does not reuse stale target-dependent partitions as complete;
- can keep validated reusable base partitions under exact proof;
- can recover an inactive store generation under its original identity;
- never relabels last-known-good.
