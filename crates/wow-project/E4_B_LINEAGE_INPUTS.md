# E4-B project lineage input producer seam

**Status:** normative supporting contract for [`wow-graph/e4`](../wow-graph/e4/README.md); implementation has not started.

`wow-project` owns project/source/analyzer/recognizer generation facts that can support cross-generation lineage candidate and change analysis. It does not own accepted lineage assertions, review promotion, replacement authority, migration recipes, or static impact traversal.

## Producer operations

```text
build_project_stable_identity_partition
build_project_source_fingerprint_partition
build_project_structural_change_partition
validate_project_lineage_input_partition
```

These operations are exact-generation-bound and produce typed `LineageInputPartition` values for E4-C orchestration and independent `wow-graph` E4-B validation.

## Exact inputs

```text
before exact ProjectStore/ProjectSnapshot/GraphSnapshot publication
before exact source/analyzer/recognizer manifests

after exact ProjectStore/ProjectSnapshot/GraphSnapshot publication
after exact source/analyzer/recognizer manifests

exact E4-B producer/profile/canonicalization IDs
finite budgets/cancellation
```

`wow-project` does not resolve current inside the producer. E4-C supplies retained exact before/after publications.

## `project_stable_identity`

Eligible records require an owner-controlled stable semantic identity independent of display name, path, source line, SQL row, insertion order or repository popularity.

Examples can include an analyzer/project entity identity that the project generation contract explicitly preserves across an incremental update or an explicit reviewed project identity transition.

Every record contains:

- exact before/after entity refs and generations;
- stable identity schema/profile/version;
- owner fact/source/evidence records;
- identity uniqueness/collision checks;
- coverage/conflicts;
- canonical digest.

The producer may request a `Proven` continuity ceiling only when the E4-B relation profile recognizes that exact owner identity and all coverage/collision gates pass.

It never automatically emits `moved_from`, `renamed_from`, `replaced_by`, `removed_after` or `introduced_in`.

## `project_source_fingerprint`

Produces explainable bounded features for Candidate discovery, such as reviewed combinations of:

```text
entity kind and structured signature shape
canonical semantic fact manifest
bounded declaration/control/effect feature set already published by owners
owner/load/registration/state/API relation feature set
source-map-stable normalized structural feature records
```

Hard rules:

- features and every origin are typed and versioned;
- no opaque embedding/model output;
- no full source body by default;
- unknown/unsupported/lossy regions are explicit;
- repository/owner/provider/path/popularity values cannot act as hidden semantic features;
- generated/vendor/copied/duplicate source hard negatives are mandatory;
- maximum lineage ceiling by itself is `Candidate`.

A digest without a feature manifest is not explainable enough for correctness-path promotion.

## `project_structural_change`

Produces exact before/after typed facts that assist change classification:

```text
canonical name
source file/span/container/owner
package/TOC/load role
signature/type facets
registration/event/callback/hook/state/object/template relations
API/reference use relations
entity/source presence under exact closed scopes
```

For paired fields/relations, `wow-project` preserves:

```text
Known
ExplicitNull
Missing
Unknown
Unsupported
Omitted
Conflict
NotEvaluated
```

The producer does not decide that a pair is the same lineage. Pair-specific change records are accepted only after `wow-graph` validates the governing lineage relation.

## Presence and negative-authority inputs

Project input can provide closed source/entity inventory and owner negative-authority records needed by E4-B removal/introduction evaluation. It cannot itself declare `Removed` or `Introduced` from an unmatched entity.

Coverage must identify the exact:

- root/package/TOC flavor;
- source inventory and excluded members;
- entity kind/producer partitions;
- analyzer/recognizer/graph coverage;
- privacy/license exclusions;
- failure/cancellation/truncation/conflicts.

## Native WoW structure distinctions

Do not collapse:

- native frame events;
- EventRegistry native frame bridges;
- custom EventRegistry producers/subscribers;
- CVar callbacks;
- `SetScript`, `HookScript`, `hooksecurefunc`;
- TOC dependencies/load phases;
- XML parent/inheritance/mixin/template/factory relations;
- SavedVariables roots and dynamic state paths.

Static changes in these facts do not prove runtime delivery, readiness, taint, combat, protected-action, Secret Value or performance behavior.

## Partition ownership

Conceptual keys:

```text
project_stable_identity:<profile>:<entity-kind/scope>
project_source_fingerprint:<profile>:<entity-kind/scope>
project_structural_change:<profile>:<facet/relation/scope>
```

Updating one project generation or producer profile replaces only its exact partition in the E4-B input set. Prior generation publications remain immutable.

## Removal closure

When a source/entity/fact disappears from the after project generation, the target producer partitions must contain no stale:

- stable identity record;
- fingerprint feature;
- structural change input;
- source/evidence handle;
- coverage manifest member;
- graph/search handoff mapping.

E4-B then evaluates whether the absence is authoritative.

## Security and privacy

- consume published exact views only;
- no repository/network/client discovery;
- no source execution or new parser;
- no arbitrary filesystem path/read;
- no raw SQL/store connection;
- no model/embedding;
- no private source/body/absolute path/credentials in public records;
- source comments remain untrusted evidence data;
- exact privacy/license profile controls feature/source export;
- all entities/features/partitions/output/time/memory are bounded and cancellable.

## Evaluation

Required project producer fixtures include:

- stable identity continuity;
- stable identity collision;
- rename/move with accepted continuity;
- copy versus move;
- split/merge candidates;
- identical copied/vendor/generated fingerprints;
- same-name unrelated entities;
- source/package/load/signature/state/event/hook changes;
- complete and partial presence coverage;
- removed source with stale-output mutation;
- repository/owner/path/popularity rename invariance;
- 1/2/N workers and shuffled source/fact order.

## Nonclaims

Project producer records do not by themselves establish:

```text
accepted lineage
replacement or deprecation
migration recipe
runtime impact or breakage
platform API/restriction authority
successful edit or release readiness
```
