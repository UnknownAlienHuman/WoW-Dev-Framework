# Recognizer partitions, versions, activation, and invalidation

**Status:** normative.

## Identity hierarchy

```text
pack ID/version
  -> rule ID/version
     -> input fact schema/graph registry profiles
        -> exact generation/scope/input partition
           -> recognizer output producer partition
```

Every output assertion proposal can be traced to this hierarchy.

## Pack version

Pack version changes when the active rule set, rule order identity, compatibility profile, budgets that can change output, or evaluation gate changes.

## Rule version

Rule version changes when any output-affecting behavior changes:

- clauses/predicates/joins;
- required capabilities/scope;
- capture types/cardinality/identity;
- graph output kind/direction/key ingredients/attributes;
- confidence/ambiguity/no-match policy;
- deterministic canonicalization;
- output limits that alter accepted results.

Documentation wording or additional nonsemantic labels do not force a semantic version change.

## Output partition key

```text
pack/rule/version
fact schema profile
graph registry bundle
exact input universe/scope/partition
exact analyzer/project/reference generation dependencies
```

Same rule over two files, TOC variants, packages, or project generations produces distinct partitions.

## Invalidation

A partition invalidates when any declared dependency changes:

```text
source fact set/digest
analyzer/project generation
TOC/XML dependent partition
pack/rule/profile/version
required graph registry definition
capability/coverage/conflict state
input adapter version
```

Path/mtime alone is insufficient unless it changes the owned project fact identity.

## Replacement

```text
old complete output partition
+ new exact input/rule version
-> match and validate new partition
-> graph validates target proposals
-> atomic graph producer-partition replacement
```

On failure/partial/cancel, the target complete partition is not published. Prior partition may remain last-known-good under its old identity but cannot satisfy a request for the new generation.

## Pack enable/disable

Activation is explicit configuration against exact pack/trust/profile compatibility.

- E2 core pack is repository-owned and reviewed.
- Calibration/experimental packs are not auto-loaded.
- Disabling a pack schedules empty replacements for its producer partitions and emits coverage loss.
- Disabling never changes entity/relation definitions or other producer assertions.

## Rollout state

```text
disabled
shadow       evaluate/report but do not publish default graph assertions
default      eligible for publication after gates
```

Technical correctness and rollout policy are separate. New rule versions return to shadow unless compatibility policy explicitly proves no semantic change.

## Compatibility

A pack declares exact accepted ranges/IDs for:

- `wow-core` contract/schema;
- `wow-emmy` fact schema;
- recognizer fact adapter schema;
- `wow-graph` registry bundle;
- project TOC/XML fact schema;
- canonicalization and error schema.

Unknown/incompatible versions produce `NotEvaluated`, not best-effort field guessing.

## Historical output

Historical producer partitions may be retained for exact generation comparison/debugging. They are never merged with active project facts without an explicit graph/lineage query.

## Corpus versioning

Evaluation corpus manifest pins:

```text
fixture/project/repository revision
fact adapter/profile
graph registry
expected labels and label authority
mutation definitions
license/provenance
```

Changing labels or corpus membership changes corpus version and report identity. It does not automatically change a rule; disagreements require review.

## External/named packs

E5 pack metadata may name Ace3, oUF, WeakAuras, BigWigs, Details, Plater, or user repositories as pinned calibration sources. Names remain provenance only. Production rule conditions cannot inspect them.

## Freeze gate

Before first Rust implementation:

- exact prerequisite implementation commits and fixture digests;
- core pack/rule/profile versions;
- graph registry bundle;
- input fact schema/adapters;
- output partition and replacement vectors;
- rollout/evaluation profile;
- all fixture/member/bundle SHA-256 values.

Required nulls are invalid after implementation state changes from `not-started`.
