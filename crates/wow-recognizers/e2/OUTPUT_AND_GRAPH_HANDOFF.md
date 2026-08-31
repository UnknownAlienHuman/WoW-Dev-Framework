# E2-B proposed output and `wow-graph` handoff

**Status:** normative boundary; recognizers never publish graph state directly.

## Output principle

Recognizer output is a fully explainable proposal set bound to one exact producer/rule/input partition. `wow-graph` independently validates registries, semantic keys, endpoints, assertion identity, conflicts, and publication.

## Proposed entity assertion

```text
ProposedEntityAssertion
    proposal_id
    requested graph entity kind ID/version
    exact scope/universe/profile/project/reference generation
    semantic key ingredient map
    typed attribute map
    rule/pack/match/capture IDs
    input fact/source/evidence/coverage IDs
    derivation explanation ID
    confidence = Derived | Possible
    ambiguity refs
    canonical digest
```

## Proposed relation assertion

```text
ProposedRelationAssertion
    proposal_id
    requested graph relation kind ID/version
    direction
    source and target semantic-key ingredient maps
    relation semantic qualifiers/attributes
    exact scope/generation
    rule/pack/match/capture/input/evidence/coverage refs
    confidence = Derived | Possible
    ambiguity refs
    canonical digest
```

## What recognizers may use from `wow-graph`

- immutable registry bundle definitions;
- entity/relation/attribute/axis validation contracts;
- proposal request/result types;
- endpoint-kind and confidence constraints;
- deterministic key-ingredient canonicalization helpers explicitly exported for proposal construction.

## What recognizers may not use

- store/connection/transaction handles;
- graph generation publication APIs;
- raw assertion/entity/relation tables;
- mutable graph actor internals;
- query traversal to discover hidden matcher inputs;
- graph conflict resolution policy beyond validating proposed shape;
- final `GraphGenerationId` or assertion IDs.

## Handoff sequence

```text
RecognizerFactBundle
-> match one rule/input partition
-> build RecognizerOutputPartition
-> caller validates exact project/input generation
-> caller groups output partitions for target graph generation
-> wow-graph validates proposals against exact registry/scope/base snapshot
-> wow-graph derives semantic keys/assertion IDs/conflicts/views
-> wow-store publishes through graph-owned replacement plan
-> caller records accepted/rejected proposal report
```

`wow-project` is expected to own this orchestration in E2-C/E2-D. Recognizers remain pure over immutable inputs.

## Accepted/rejected report

```text
GraphProposalValidationReport
    recognizer output partition ID
    target graph registry/base generation
    accepted proposal -> graph assertion/key mapping
    rejected proposal -> exact graph validation error
    conflicts/coverage impacts
    canonical digest
```

A rejected proposal is not silently omitted from recognizer evaluation. It becomes a contract defect, incompatible profile, or explicit partial outcome.

## Semantic key ingredients

Rules map exact captures into graph-registry-defined identity fields. Examples:

```text
state_root
    project universe + exact TOC-declared variable identity

state_path
    exact root key + ordered literal path segments

custom event
    project universe + exact event key + producer scope policy

function/frame/template
    exact supplied project/XML/analyzer semantic identity
```

Display names, insertion order, source lines, SQL IDs, and repository names cannot substitute for required semantic identity fields.

## Evidence closure

Every proposal retains:

- exact source fact IDs;
- source handles/spans where applicable;
- producer and generation context;
- evidence/provenance/confidence;
- source capability coverage;
- rule/capture/derivation steps;
- competing matches/ambiguity.

A graph assertion cannot claim stronger confidence or provenance than the proposal/input permits.

## Partition identity

```text
RecognizerProducerPartitionKey
    pack ID/version
    rule ID/version
    fact schema/graph registry profiles
    exact input universe/scope/partition
    project/reference/analyzer generation inputs
```

The output partition manifest lists all proposals, outcomes, ambiguities, coverage, and truncation. Empty complete output is valid only for a fully evaluated no-match rule scope.

## Replacement

A pack/rule/input partition update atomically replaces prior accepted graph assertions from that producer partition. It cannot delete assertions from another rule or producer.

If matching is partial/failed/truncated:

- no complete replacement is published by default;
- caller may publish an explicit partial producer partition only under a frozen graph/project policy;
- previous complete partition is not relabeled as the target generation;
- coverage and last-known-good identities remain explicit.

## Rule removal/disablement

Disablement produces an explicit empty replacement plan for that producer partition plus coverage downgrade. It does not emit negative assertions or rewrite graph kind semantics.

## Determinism

Equivalent input/pack/registry profiles yield identical proposal IDs, canonical ordering, output partition digest, and graph validation request, independent of matching parallelism or graph physical storage.
