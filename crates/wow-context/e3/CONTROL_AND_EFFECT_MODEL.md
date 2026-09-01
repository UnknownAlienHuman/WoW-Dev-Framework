# E3-B control and effect projection

**Status:** normative closed projection over already-published facts. It is not a parser, CFG, SSA, data-flow engine, runtime tracer, or safety proof.

## Purpose

L1/context packs need a compact way to express exact known control/effect structure without reopening source or inventing a second analyzer. E3-B therefore projects a closed registry of published project, analyzer-derived, recognizer, graph, and ReferenceView records.

## Allowed node classes

```text
Declaration
FunctionOrMethod
CallbackOrHandler
LoadEntry
LifecycleEntry
CallSite
ApiUse
EventRegistration
EventHandler
CustomSignalProducer
CustomSignalSubscriber
CVarCallback
SetScript
HookScript
SecurePostHook
StateRead
StateWrite
StateRootOrPath
ObjectCreation
TemplateReference
InheritanceOrMixin
ExistingFindingEvidence
UnknownRegion
CollapsedRegion
OmittedRegion
```

Every node cites exact input records and retains universe, generation, source handle/span, provenance, confidence, coverage, and conflict state.

## Allowed relation classes

Only existing exact graph relations or typed reason paths are projected, for example:

```text
contains / owns / loads / loads_before
calls / possible_calls
uses_api
registers_event / handles_event
triggers_callback / subscribes_callback
sets_script / hooks
reads_state / writes_state
created_by / references_template / inherits / mixes_in
```

A reason path remains a path. E3-B never persists or renders it as a direct edge.

## Distinct signal/hook systems

Do not collapse:

- native frame events;
- EventRegistry native frame-event bridges;
- custom EventRegistry signals with exact producer/subscriber evidence;
- CVar callbacks;
- `SetScript` handlers;
- `HookScript` handlers;
- `hooksecurefunc` post-hooks.

An event-like string is not enough to join systems. A hook relation proves only the structural hook fact available from the producer; it does not prove taint, combat, protected-action, forbidden-aspect, runtime accessibility, or performance safety.

## State effects

State projection preserves:

```text
exact TOC-declared SavedVariables roots
exact literal state paths
possible dynamic prefixes/paths
read versus write
source owner and confidence
```

No SavedVariables contents are read. Same-name globals are not roots without the exact declaration fact. Dynamic keys stay `Possible` or `NotEvaluated`.

## Call/control limits

- use the published call/possible-call relations only;
- preserve recursion/cycles and cycle-safe paths;
- do not infer branch dominance, reachability, exception behavior, callback delivery, or runtime order unless an exact owner fact already states it;
- do not reconstruct bodies, AST, CFG, SSA, or value flow;
- do not treat static load order as execution success;
- do not turn incomplete call coverage into absence.

## Unknown, collapsed, and omitted regions

```text
UnknownRegion
    owner capability did not evaluate or cannot represent the requested structure

CollapsedRegion
    exact member set is known but intentionally summarized under a profile/budget with member manifest and expansion route

OmittedRegion
    known candidate was excluded by profile, privacy, license, confidence, conflict, duplication, or budget; omission record required
```

These states are not interchangeable. None implies an empty region.

## Compact effect facets

Canonical facets are typed, not prose:

```text
reads_state = [exact path refs]
writes_state = [exact/possible path refs]
registers_native_event = [event relation refs]
subscribes_custom_signal = [producer/subscriber refs]
hooks = [target, hook kind, handler refs]
uses_api = [ReferenceEntity refs]
creates_object = [factory/template/object refs]
```

No model-generated “responsibility,” side-effect, or control-flow summary.

## Selection

Control/effect items are selected only when:

- requested by an intent/profile;
- connected to an exact root through permitted direct relations or bounded paths;
- supported by required origin/evidence/coverage;
- allowed by confidence/privacy/license policy;
- within mandatory/optional budget rules.

Same-name/path proximity is not relevance.

## Nonclaims

Projection never establishes:

- actual runtime invocation or order;
- event delivery/payload accessibility;
- Secret Value access or declassification;
- protected/forbidden legality;
- taint/combat safety;
- performance/hot-path frequency;
- cross-build lineage or impact;
- diagnosis or remediation.

## Validation

- every node/relation resolves to exact published records;
- no parser/analyzer/storage fallback;
- direct/path distinction preserved;
- confidence/provenance/coverage/conflicts retained;
- unknown/collapsed/omitted states complete;
- no signal/hook/state class collapse;
- no runtime/safety upgrade;
- deterministic ordering/bytes under cyclic and shuffled inputs.
