# Exact selection and progressive context expansion

**Status:** normative E3-A request/selection algorithm.

## No implicit search

E3-A starts from exact roots. Acceptable root inputs are:

```text
EntityKey
semantic symbol ID bound to the exact analyzer/project snapshot
SourceHandle + exact SourceSpan
package/load-unit ID
exact graph query result ID
explicit ordered root set
```

A free-text name, partial path, fuzzy symbol, migration question, or natural-language query must first be resolved by an owning later search/service layer. `wow-context` does not guess.

## Normalized request

Normalize and validate:

- exact artifact/input/profile IDs;
- roots and expected scope/kind assertions;
- requested level and map sections;
- relation/axis/direction allowlists;
- confidence/provenance policy bounded by ContextProfile;
- source-excerpt permission;
- hard budgets and optional tokenizer requirement;
- continuation cursor/cancellation.

Equivalent request ordering produces one normalized request ID.

## Candidate fragments

Candidate fragments come only from:

- root L0/L1 skeletons;
- mandatory Project Map header/coverage sections;
- exact direct graph relations/axis members under the profile;
- reason-path nodes needed to explain selected endpoints;
- explicitly requested source excerpts;
- mandatory evidence/coverage/conflict/redaction/truncation closure;
- optional neighborhood sections permitted by the profile.

No candidate is created by raw-source scanning, fuzzy search, model suggestion, popularity, or repository-specific rules.

## Priority classes

```text
P0 request identity and generation coherence
P1 exact root identity/L0 and mandatory uncertainty closure
P2 required owner/load/source/registration/state reason closure
P3 requested L1 fields and direct relations
P4 requested cross-universe bridge endpoints and reason paths
P5 requested Project Map sections
P6 optional one-hop neighborhood
P7 optional excerpts/detail hints
```

Lower-numbered classes are selected first. Within a class, use profile-defined kind/relation order and canonical semantic keys.

## Dependency closure

A fragment can require:

- owning L0 skeleton;
- relation source/target skeletons;
- path predecessor nodes/edges;
- source/evidence/coverage/conflict records;
- a redaction/truncation notice;
- a section header or map index record.

A fragment is included only if required closure fits or the profile defines an explicit minimal/truncated representation. Never include a relation sentence without enough IDs/state to interpret it.

## Expansion algorithm

```text
validate exact request and inputs
-> enumerate bounded candidates from existing artifacts/views
-> assign priority and stable ordering keys
-> compute required dependency closure and cost
-> select mandatory P0/P1 closure
-> add requested P2-P5 groups while budgets permit
-> add optional P6/P7 fragments while budgets permit
-> record every omission/truncation and continuation boundary
-> validate evidence/coverage/conflict and budget closure
-> render from canonical selected fragments
```

The algorithm is deterministic under worker/input/database order.

## Graph expansion

Every graph expansion names:

```text
exact GraphSnapshot
root(s)
axis or relation whitelist
direction
max depth/nodes/edges/paths/bytes
confidence and coverage policy
candidate/possible policy
```

Use bounded graph operations. No whole-graph dump and no unrestricted path enumeration.

## `Possible` and `Candidate`

- Default: include `Proven` and `Derived` only.
- `Possible` requires request/profile opt-in and remains visibly labeled.
- `Candidate` is excluded from E3-A canonical project context.
- A bundle cannot claim proven impact/ownership/callability through a possible path.

## No-new-evidence stopping rule

Expansion stops when:

- no new selected fragment contributes an entity/relation/source/evidence/coverage/conflict ID not already represented;
- requested depth/section is complete;
- a hard budget is reached;
- required upstream coverage is unavailable;
- cancellation occurs.

`no_new_evidence` is distinct from authoritative absence.

## Omission records

```text
ContextOmissionRecord
    candidate/group/section ID
    reason:
        not requested
        profile excluded
        confidence excluded
        candidate excluded
        dependency closure did not fit
        byte/record/node/edge/path/token/excerpt/time budget
        source unavailable/redacted
        upstream NotEvaluated/conflict
        duplicate semantic contribution
    stable next ordering key or continuation ref
```

Optional candidates omitted as duplicate semantic contribution remain traceable in selection metrics without changing artifact truth.

## Continuation

Continuation binds exact artifact set, normalized request, profile/rendering/tokenizer, ordering version, last stable candidate key, and prior truncation/budget state.

A cursor cannot be replayed against another snapshot or enlarged policy silently. A new request with larger budgets gets a new request ID; it may deterministically reuse the same candidate ordering.

## Explain operation

`explain_context_fragment(fragment_id)` returns:

- exact selection reason/priority;
- dependent fragments;
- input entity/relation/assertion/source/evidence/coverage/conflict records;
- derivation/template/profile IDs;
- costs, omissions, redaction, and truncation;
- rendered member mappings.

It does not generate a new narrative explanation.
