# E4-B candidate matching and ambiguity resolution

**Status:** normative bounded deterministic matching contract.

## Purpose

Candidate matching proposes plausible cross-generation endpoint relations. It does not itself establish graph truth.

```text
exact endpoint sets
+ explicit/stable/structural/search candidate signals
+ relation and matching profiles
-> bounded candidate bipartite/hypergraph components
-> hard compatibility filtering
-> deterministic feature vectors and proof ceilings
-> globally constrained candidate solutions per component
-> unique proposals or explicit ambiguity/conflict
```

## Candidate generation lanes

### Explicit-transition lane

Consumes exact owner transition records naming old/new endpoints and relation scope. Highest available static ceiling, subject to profile/coverage/conflict validation.

### Stable-owner-ID lane

Pairs endpoints carrying the same exact owner-defined stable ID under a frozen identity contract. Stable IDs are namespaced by owner/profile/kind.

### Exact structural-continuity lane

Uses exact compatible discriminators such as:

- declaration identity components preserved by the owner;
- package/file/load-unit/source-map continuity;
- canonical signature/type/member identities;
- exact owner relation neighborhoods;
- exact content/semantic fingerprints declared by profile.

No single mutable label is sufficient unless the owner contract defines it as stable identity.

### Shape/fingerprint lane

Generates `Possible` or `Candidate` endpoint pairs from structured feature overlap/difference. It is useful for renamed/moved candidates but cannot bypass ambiguity gates.

### E4-A search lane

Imports exact `SearchCandidateSignal`/explanation records as `Candidate` inputs. It preserves query, lane, shard, generation, rank, matched fields, evidence, and coverage. Raw search score is not a lineage score and is never compared as proof.

### External/model lane

Deferred by default. When later activated, always Candidate-only and isolated by provider/run/profile identity.

## Hard compatibility filter

Reject pair/group candidates before scoring when any applies:

- wrong generation side/direction;
- incompatible universe/owner/entity kind;
- profile/build/flavor incompatibility not allowed by comparison mode;
- explicit distinct stable IDs under an authoritative identity contract;
- impossible relation cardinality/schema;
- source/reference/project authority mismatch for the requested relation;
- privacy/license policy forbids required evidence;
- required endpoint/evidence record unresolved;
- candidate exceeds budgets.

Filtered candidates remain in diagnostics when useful, with reason codes.

## Feature vector

A candidate feature vector is closed, typed, integer/ordinal, and versioned. It may include:

```text
explicit transition class
stable-ID equality/conflict
exact canonical identifier equality/change
exact source location equality/change
package/owner/load continuity
signature/type/restriction facet equality/difference
content/semantic fingerprint class
relation-neighborhood exact overlap/difference
source declaration continuity class
E4-A candidate authority band and lane ordinals
coverage/conflict penalties
```

Raw source text, arbitrary embeddings, floating scores, repository popularity, model prose, file timestamps, and database order are excluded.

Features carry origin and proof ceiling. Aggregation cannot raise the candidate above the strongest valid lane ceiling.

## Component construction

Candidates sharing endpoints form a bounded connected ambiguity component. Matching is solved per component after hard filtering.

Profiles bound:

```text
from/to endpoints per component
candidate edges/groups
split/merge group size
feature records
solution count retained
CPU/memory/time
```

Over-budget components become `BudgetTruncated`/`NotEvaluated`; they are not greedily approximated as authoritative.

## Constraints

Depending on relation kind:

- one-to-one continuity relations enforce at most one accepted target/source unless split/merge evidence exists;
- split/merge use explicit hyperedge/group candidates;
- endpoint kind/profile compatibility is mandatory;
- explicit mutually exclusive transitions cannot both be accepted;
- accepted rename/move/signature-change requires an accepted continuity basis;
- replacement does not consume identity matching unless the profile declares exclusivity for the capability scope;
- removal/introduction candidates are evaluated only after continuity components close.

## Deterministic objective

The objective is lexicographic, not an opaque floating score. A typical order:

```text
1. satisfy compatible explicit transition/stable-ID constraints
2. minimize authoritative conflicts
3. maximize count/coverage of highest-ceiling uniquely supported relations
4. maximize reviewed integer structural feature vector
5. minimize unsupported assumptions and ambiguity
6. use stable endpoint/proposal IDs only to order equivalent output, not to choose among semantically indistinguishable solutions
```

A lexical ID tie-break can order multiple valid solutions for reporting but cannot declare one true when semantic objectives are equal. Equal optimum solutions remain an ambiguity group.

## No greedy matching

Forbidden:

```text
iterate old endpoints and accept first available new endpoint
accept top search result per old endpoint
sort by one aggregate score and consume endpoints
let source order decide collisions
pick lexical path/name when scores tie
```

These fail one-to-many, many-to-one, swapped-name, duplicated-body, moved-file, and simultaneous rename scenarios.

## Unique solution

A `UniqueDeterministicSolution` exists only when:

- one solution satisfies all hard constraints;
- it is strictly superior under the complete semantic objective before output-only ordering;
- all decisive lanes/capabilities have required coverage;
- no unresolved equal-ceiling explicit conflict exists;
- proof ceiling for each selected relation is independently satisfied.

Uniqueness of optimization output does not automatically raise proof class; selected candidates retain lane ceilings.

## Ambiguity outcomes

```text
ResolvedByExplicitEvidence
    exact owner evidence resolves alternatives

UniqueDeterministicSolution
    one solution under frozen semantics; proposals keep their proof ceilings

MultipleValidSolutions
    two or more semantically tied solutions; retain all

Conflict
    incompatible explicit/high-authority constraints

BudgetTruncated
    component not fully enumerated/evaluated

NotEvaluated
    required capability/profile/evidence missing
```

## One-to-many and many-to-one

Split/merge candidates require explicit group features and scope. Pairwise similarity cannot silently create a group. Group validation checks:

- every endpoint exact and unique;
- cardinality valid;
- capability/contract scope partitioned or combined explicitly;
- source/owner transition evidence or approved deterministic composite rule;
- no hidden unmatched endpoint that changes interpretation;
- complete required coverage.

## Removal/introduction evaluation

After matching:

```text
unpaired from endpoint
-> resolve all candidate components and high-ceiling lanes
-> verify complete to-generation inventory/query coverage
-> verify no profile/conflict/truncation blocker
-> emit removed_in or partial/unresolved record

unpaired to endpoint
-> symmetric introduced_in gate
```

An unpaired endpoint is not automatically removed/introduced.

## Search handoff

E4-A search can provide candidate pairs by querying old identifiers/shapes against the exact target shard or reverse. Required fields:

- exact E4-A SearchUniverseSet and shard generations;
- exact query/result/candidate/signal IDs;
- matched owner entity endpoints;
- lane authority band and explanation;
- coverage/conflicts/omissions/truncation;
- query-relative rank.

The lineage matcher records this as Candidate evidence. It does not rerun search or interpret snippets as source truth.

## Review queue

Ambiguous/Candidate/Possible proposals can be exposed as a deterministic review queue:

```text
ambiguity component
candidate endpoint pairs/groups
supporting and opposing signals
proof ceiling
missing discriminators
exact source/reference/project detail handles
allowed review decisions
```

Review is data input to a new decision/partition; no in-place mutation.

## Determinism

Equivalent exact inputs yield identical:

- candidate sets and filtered reasons;
- component decomposition;
- feature vectors;
- semantic objective values;
- complete optimum solution set within budgets;
- ambiguity status;
- proposal IDs/order;
- omissions and budget reports.

Worker count, input order, storage rows, cache history, or search completion order cannot change them.

## Required mutation cases

- two old entities swap names;
- two equal bodies/signatures move to new files;
- one old entity splits into two targets;
- two old entities merge;
- exact stable ID conflicts with top fuzzy hit;
- explicit replacement exists but same-identity candidate ranks higher;
- same path reused by a different entity;
- source order reversed;
- duplicate candidate signals repeated many times;
- candidate component over budget;
- equal optimum solutions;
- partial target inventory incorrectly reported as removal.
