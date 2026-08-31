# E3-A detail planning and progressive expansion

**Status:** normative exact-root graph/project expansion contract.

## Goal

Select the smallest evidence-bearing context that satisfies the explicit request under exact budgets and proof limits.

## Expansion inputs

```text
validated ContextRequest and ContextPlan
exact input views/generations
current ContextFrontier
allowed relation lanes/directions
confidence/coverage/conflict policy
mandatory and optional inclusion sets
budget and cancellation state
```

E3-A accepts exact roots only. Search/ranking does not occur here.

## Relation lanes

Profiles may expose separate lanes such as:

```text
lexical
ownership
load
object
inheritance
registration
lifecycle
state
call
API-use/reference
source/evidence
```

Each lane declares:

- allowed relation kinds/directions;
- endpoint kinds;
- direct versus bounded-path behavior;
- confidence policy;
- required source/graph capabilities;
- cycle/depth rules;
- inclusion priority and cost model;
- stopping/continuation behavior.

No cross-lane generic expansion.

## Work item

```text
ContextWorkItem
    root and current entity/path
    lane/direction/detail target
    reason and originating route
    priority class/stable key
    required capabilities
    estimated structural/source/token cost
    depth/path state
```

Estimates are profile-labeled and cannot override mandatory evidence or exact policy.

## Expansion algorithm

```text
validate exact frontier/request/input
-> select next work item by frozen priority/stable order
-> validate capabilities/budgets/cancellation
-> issue one exact bounded registered graph/project/reference query
-> validate query snapshot/result/coverage/conflicts
-> classify each result as new, duplicate, rejected, blocked, unsupported, or deferred
-> build/update skeleton/evidence/loss/omission records
-> update visited/included/frontier/budget state
-> emit one deterministic ContextExpansionStep
-> stop or continue under explicit policy
```

No query can silently broaden root, lane, confidence, universe, or generation.

## Priority classes

A profile may use:

```text
0 mandatory identity/evidence/blockers
1 request-explicit roots/fields/lanes
2 direct ownership/load/lifecycle/registration context needed to interpret roots
3 direct source/signature/member/API/state context
4 bounded supporting reason paths
5 optional neighboring detail
```

Within class use stable semantic ordering, not model scores or query completion.

## Direct versus path expansion

### Direct

Returns exact one-hop graph relations and supporting assertions.

### Bounded path

Returns explicit entity/relation sequence with reason/evidence and limits. Path inclusion does not create a direct relation in skeleton or graph.

## Duplicate handling

An item is duplicate only under exact semantic IDs/profile field equality. When duplicate presentation is merged:

- retain all evidence/assertion/source/route refs;
- retain strongest blockers and weakest applicable confidence/coverage ceiling;
- record avoided duplication metrics;
- do not discard a distinct source occurrence when relevant.

## Cycles

Visited state is lane/path/profile-specific. A cycle can be valid (call/state) or conflicting (selected hierarchy/load policy). Expansion:

- records cycle closure/reason;
- does not recurse indefinitely;
- retains conflict if source graph declares one;
- does not infer transitive self-edge;
- emits deterministic stopping/omission state.

## Confidence policy

Default:

```text
Proven and Derived included
Possible opt-in and visibly labeled
Candidate excluded
```

If the request permits Possible/Candidate, their budget and presentation remain separate; they never upgrade due to repeated matches.

## Coverage/conflict boundaries

When required capability is partial/failed/unknown/conflicted:

- include available exact records allowed by policy;
- add blocker/loss/NotEvaluated record;
- do not treat missing neighbors as absence;
- stop or continue independent lanes according to profile;
- preserve exact affected partition IDs.

## Source detail lane

A detail route can request source:

```text
entity/source handle
exact declaration/member/span target
context-line/span expansion profile
license/privacy/security policy
source byte/line budget
```

Source excerpt construction occurs through the separate source-excerpt contract, not by graph text or analyzer prose.

## Budget reservation

Before optional expansion reserve mandatory room for:

```text
input/generation header
root identities
required evidence and blocker records
truncation/stopping/continuation metadata
```

An optional large branch cannot consume mandatory footer/evidence space.

## Partial step

If a query/result exceeds an internal budget:

- only a profile-permitted deterministic partial result may be included;
- exact returned/omitted counts and query truncation remain visible;
- frontier/continuation reflects unprocessed work;
- bundle cannot claim request completion.

## No-change and no-new-evidence

### NoChange

Continuation/request has no new requested profile/input/frontier work and returns the same bundle identity or explicit no-change response.

### NoNewEvidence

A processed frontier branch adds no new evidence/semantic record. Record exact branch/query and stop that branch. It does not mean entity/relationship absence unless domain coverage independently authorizes it.

## Cancellation

Check before and after each bounded query, source excerpt, and serialization step. Cancellation creates no complete bundle and no background continuation. Completed exact steps may be returned only under an explicit partial-cancelled profile.

## Determinism

Equivalent frontier/input/profile/budget yields identical work-item selection, queries, included/rejected/duplicate records, budget deltas, output frontier, and stopping decision under 1/2/N internal execution.

Parallel queries can run only when their merge order/result is proven deterministic and does not let one optional branch starve another nondeterministically.

## Required tests

- each lane direct and bounded path;
- same root in several independent axes;
- direct edge versus reason path;
- cycles by allowed/conflicting axis;
- Possible/Candidate default and opt-in;
- partial/conflicted capabilities and independent-lane continuation;
- duplicate evidence preservation;
- mandatory budget reservation;
- source detail route;
- no-new-evidence versus authoritative absence;
- partial query/truncation/continuation;
- cancellation at every step;
- 1/2/N and randomized query completion determinism;
- exact-generation cursor/request enforcement.

## Hard stops

- no fuzzy/root expansion/search;
- no lane/confidence/generation broadening;
- no full graph dump;
- no direct edge from path;
- no model-scored priority in correctness path;
- no blocker/coverage omission;
- no optional branch starving mandatory records;
- no background continuation.
