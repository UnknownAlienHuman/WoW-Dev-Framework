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

The machine-state contract uses metadata schema 3 and `declared_rule_ids` for
the declared core capability inventory. Those identities do not advertise a
runtime rule set: the exact selected pack ID/version/digest owns its rule
versions. A native adapter pack and the shared core example are distinct packs.
`capability_routing` separately records current availability. In particular,
`core.library.embed@1` remains declared but `not-evaluated`, blocked by #159;
the executable shared pack omits the unsupported rule. The explicit xtask owner
map rejects both a false functional embed claim and the old misleading
`active_rule_ids` field. This metadata change accepts no fixture or checksum.

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

## Exact decoded EventRegistry keys

The W2 native bridge and W3 custom-signal adapters preserve the decoded string literal as the event key. W3 producer/subscription matching and graph entity identity use that exact value, including leading or trailing whitespace. A W3 subscription relation describes the exact registration site from its enclosing source function to that event key; it does not identify the callback or owner argument. Unresolved callback/owner values remain unbound and produce no handler-specific edge. Empty and over-limit inputs retain each adapter's existing conservative handling: W3 omits the unsupported site, while W2 fails closed with its budget error.

This interpretation change is identified by W2 profile wow-recognizers/lua-native-event-bridges/5 and fact profile wow-recognizers-lua-native-event-bridge-facts-5, plus W3 profile wow-recognizers/lua-custom-signals/5 and fact profile wow-recognizers-lua-custom-signal-facts-3. The typed rules and graph registry are unchanged, so their versions do not change. The real replacement consumer is wow-service::graph::build::signals::publish_signals: it replaces the stable W2 and W3 graph partitions from the new exact facts. Existing snapshots keep their recorded identities; this adds no legacy alias or reinterpretation path.

The E2-B plan schema `wow-recognizers/semantic-plan/e2-b/2` records dependency-correct order within each original clause scope. Its plan identity changes because clause interpretation and execution order now agree; old `/1` plans are not reinterpreted. The native Service witness consumes the rebuilt plan through `wow-service::graph::build::signals::publish_signals`, which replaces the W3 producer partition under its current profile. Other recognizer-backed producers likewise replace only their own output partitions when rebuilt against the new plan.

The W6 library interpretation is identified by Emmy input profile `wow-emmy/function-call-facts/10`, source profile `wow-recognizers/source-library/4`, fact profile `wow-recognizers-lua-library-facts-4`, core pack version `4`, and require/new rule version `2`. Report10 supplies exact same-file local scalar custody: a typed string at argument 0 can supply the complete major, and a typed integer at argument 1 can supply the independent NewLibrary minor. The source reference key and declaration, initializer, argument-use, and call spans remain in the retained Emmy report. Unproved locals remain unresolved. The real compatibility consumer is `wow-service::graph::build::signals::publish_signals`, which replaces the `wow-recognizers.lua-library` partition and serializes `SourceLibraryRecognition` in `GraphBuildReport.library_recognition`. Prior `/3` recognition outputs and serialized match fields are not reinterpreted. Project global-access replay schema `/13` captures the selected budget, source corpus, and analyzer snapshot identity, then rebuilds the report during hydration. The existing `primary_callable_replay` witness does not select a global-access budget; compatibility of previously captured profile9 global-access replays against report10 remains `NotEvaluated` in this slice. No Project replay version was minted here. Core embed coverage is `NotEvaluated` while no universal donor-backed owner exists.

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
