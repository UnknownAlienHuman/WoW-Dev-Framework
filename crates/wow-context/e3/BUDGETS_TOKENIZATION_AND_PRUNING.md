# E3-B budgets, token accounting, and deterministic pruning

**Status:** normative.

## Budget dimensions

Every request has system maxima and a request profile for:

```text
input views/roots
map nodes/edges/groups/members
L0/L1 skeletons and facets
graph queries/depth/fanout/paths/expansions
reference records
source handles/spans/excerpts/bytes/lines
semantic items/evidence/conflicts/omissions
canonical semantic bytes
rendered bytes per artifact
token counts or deterministic bounds
wall/CPU/memory/cancellation checks
continuation pages/cursor bytes
```

Unlimited/negative/overflowing values are invalid.

## Exact byte accounting

Canonical semantic bytes are measured from the frozen canonical semantic serialization. Rendered bytes are measured from exact output bytes after encoding/line-ending policy.

Before selection, candidates use deterministic predicted costs derived from their canonical typed payload or a profile-defined safe upper bound. After selection/rendering, exact bytes are recomputed. Mismatch outside the frozen estimator tolerance triggers re-plan or failure, never silent overflow.

## Token accounting classes

### `Exact`

Requires:

```text
specific tokenizer implementation and version
vocabulary/merge/model/config digest
normalization and special-token policy
message/framing policy if applicable
frozen deterministic test vectors
```

The exact rendered bytes are passed to that tokenizer. A model/provider display name alone is insufficient.

### `DeterministicEstimate`

A frozen local estimator produces a reproducible estimate but does not claim exact consumer tokens. Report algorithm/version/config and calibration error class.

### `UpperBound`

A frozen conservative algorithm produces a bound that must not be exceeded for its declared input class. If its assumptions do not apply, accounting becomes `Unavailable`.

### `Unavailable`

Only exact byte limits are enforced. The pack cannot claim token fit.

## Semantic versus rendered budgets

```text
semantic budget
    canonical structured pack before renderer-specific labels/layout

render budget
    exact bytes/tokens for one renderer profile
```

A semantic pack can produce multiple renderings. Each rendering validates independently. A renderer cannot silently omit semantic items; any renderer-specific selection requires an explicit new semantic request/profile and distinct semantic pack ID.

## Budget pools

Profiles may reserve pools:

```text
mandatory identity/boundary/coverage/conflict/omission metadata
Project Map
L0
L1
relations/reason paths
ReferenceView facts
source handles and excerpts
existing finding evidence
renderer overhead
continuation metadata
```

Reservations are deterministic. Unused optional capacity may flow only according to an explicit profile rule.

## Mandatory minimum

Compute the minimum mandatory closure before optional selection. It includes all records needed to interpret and trust the requested roots.

If any hard budget is less than this minimum:

```text
fail context_minimum_required_content_exceeds_budget
```

Do not:

- remove universe/generation identity;
- omit evidence/provenance/coverage/conflict state;
- hide omissions or truncation;
- drop source boundary notices;
- emit invalid/partial JSON;
- claim a smaller token count by excluding accounting metadata.

## Optional pruning algorithm

```text
calculate mandatory closure and exact/upper-bound cost
-> reject if mandatory exceeds hard limit
-> order optional candidates by profile tier and stable tie key
-> consider each candidate with complete dependency closure
-> select only when all active limits remain satisfied
-> otherwise emit BudgetPruned omission for the candidate/dependents
-> fetch approved excerpts only for selected excerpt candidates
-> recompute exact costs
-> if exact renderer cost exceeds prediction:
       apply the profile's deterministic whole-item rollback order
       or fail when rollback would remove mandatory content
-> finalize budget and omission reports
```

No binary search/random sampling/model compression.

## Whole-item pruning

Prune semantic items/sections only at declared boundaries. Never cut:

- an ID/digest;
- a JSON token/string/object;
- a UTF-8 code point;
- an evidence/coverage record away from its claim;
- a direct relation away from source/target identity;
- a source excerpt without exact truncation/range metadata;
- a reason path into a direct-edge-looking fragment.

## Source excerpt budgeting

Source excerpts have independent maxima:

- per excerpt bytes/lines;
- per file/source object;
- per universe;
- total excerpt pool;
- adjacent-context expansion;
- transformation/redaction overhead.

When a candidate range exceeds the limit, the source-excerpt profile chooses one of:

```text
Deny
ExactSubrangeWithExplicitOriginalAndReturnedRanges
ContinuationPages
DeclarationOnlyInsteadOfBody
```

There is no arbitrary head/tail clipping without a record.

## Tokenizer failure

If an exact tokenizer fails, exceeds its own limits, or does not match its frozen digest:

- exact token accounting is unavailable;
- do not reuse prior counts;
- either fail a hard-token-gated request or re-plan under an explicitly permitted deterministic upper-bound profile;
- record the downgrade and distinct profile/result identity;
- never claim exact fit.

## Rendering overhead

Renderer profiles freeze labels, separators, indentation, source-boundary prefixes, line endings, and section headers so overhead is reproducible. Exact output is measured after all transformations.

Markdown and canonical JSON can have different token/byte costs. Both link the same semantic items unless a distinct semantic request is made.

## Omission and completeness

Budget report distinguishes:

```text
known candidates selected
known candidates omitted by budget
candidates not enumerated because an upstream stage/query was bounded
input facts unavailable/partial
renderer-only overflow
```

A map/skeleton/pack cannot report complete projection coverage when unenumerated or budget-pruned items affect requested facets.

## Benchmarks and profile freeze

Before implementation, benchmark with:

- tiny synthetic project;
- pinned `roth-ui` project fixture;
- frozen Blizzard UI source fixture;
- high-fanout/cyclic graph corpus;
- large file/function/template/state/event cases;
- 1/2/N worker modes;
- canonical JSON and Markdown;
- exact tokenizer and fallback estimator profiles where active.

Freeze quantitative limits only after measurement. Documentation placeholders cannot be reported as passing thresholds.

## Determinism tests

Equivalent inputs/profile produce identical:

- candidate cost records;
- selected/omitted sets;
- selection rollback order;
- exact semantic bytes;
- exact rendered bytes;
- exact/estimated/bound token records;
- budget/overflow/truncation reports.

Storage order, cache hit/miss, worker count, host, clock, and temp roots cannot change them.
