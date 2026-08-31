# E3-A incremental updates and future-lineage inputs

**Status:** normative update contract; cross-build lineage remains deferred.

## Final-state comparison

Updates compare exact base and target source/profile states. Filesystem watcher order, commit chronology, mtime, archive order, or cache presence does not define invalidation.

```text
base source snapshot/profile/tool set
+ target source snapshot/profile/tool set
-> canonical final-state diff
-> dependency-closure invalidation
-> exact reuse proofs
-> target candidate/publication
```

## Dependency graph

Track dependencies from:

```text
source profile/root/package selection
file bytes/kind/encoding/path identity
TOC variant/directives/order/dependencies
XML include/template/object/script/virtual-source mapping
load model
analyzer pin/config/library/unit graph
fact adapters
recognizer pack/rule/version
E3 graph registry/profile
fingerprint profile
skeleton-input projection profile
E2-D store/schema/runtime profile
```

to every derived partition and manifest.

## Invalidation examples

### Lua-only content change

Invalidate changed unit analyzer facts/findings, dependent fact adapters, affected recognizer partitions, graph proposals/assertions, fingerprints, and skeleton-input records. Preserve unrelated TOC/XML/package partitions with exact proof.

### TOC file order/dependency/variant change

Invalidate selected TOC, load model, unit load membership, analyzer workspace plan where unit set/class/order semantics change, dependent recognizer/graph/fingerprint/skeleton-input partitions.

### XML include/script/template/object change

Invalidate affected XML expansion, virtual Lua units/source maps, load edges, analyzer partitions for changed virtual/physical units, recognizers, graph, fingerprints, and skeleton inputs.

### Analyzer/annotation pin change

Invalidate all affected analyzer and downstream adapter/recognizer/graph/fingerprint/skeleton-input partitions. Source inventory/TOC/XML raw facts may remain reusable if exact parser/source profiles are unchanged.

### Recognizer rule update/disable

Replace only that rule's producer partitions and dependent graph views/skeleton inputs; reduce coverage explicitly. Project/direct source partitions remain unchanged.

### Graph registry change

Revalidate all affected proposal partitions. Do not preserve previously accepted assertions merely because source facts are unchanged.

### Source build/profile/flavor change

Create a new source/project generation. Reuse may occur only for exact compatible partition content/profile identities; no cross-flavor fact merge.

## Removal closure

For every removed file/package/unit/entity/partition, the target must prove absence of:

- source/package/TOC/XML/load records;
- analyzer facts/findings;
- fact-adapter records;
- recognizer matches/proposals;
- graph assertions/index rows/conflict references;
- structural fingerprints;
- skeleton-input records/source handles;
- target generation membership/object references.

Historical prior generations retain original records under their identities.

## Unknown impact

If dependency impact cannot be proven, widen invalidation to the smallest safe ancestor partition. Unknown impact never means reuse.

## No-change

A canonical no-change result requires identical source/profile/tool/schema inputs and all expected manifests/coverage. It produces no new source/project/graph/store generation and performs no expensive rebuild.

## Structural fingerprints

E3-A exports exact-generation fingerprints for E4 candidate generation:

```text
signature
normalized declaration shape
direct typed neighborhood
package/load role
source content/span
```

Rules:

- fingerprints do not identify the same entity across builds;
- no similarity threshold or lineage assertion in E3-A;
- no file/name match as proof of continuity;
- collisions and low-information fingerprints are retained/classified;
- every fingerprint cites exact source/graph evidence and coverage;
- profile/version changes invalidate fingerprints.

## Determinism

Independent update sequences that reach the same target bytes and profiles produce the same target candidate, graph proposal set, fingerprints, publication set, and logical query results.
