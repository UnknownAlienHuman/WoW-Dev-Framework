# Context budgets and token accounting

**Status:** normative resource and accounting contract.

## Budget layers

Every operation has hard system maxima and a request budget that may only reduce them.

```text
ContextBudget
    root count
    candidate fragment count
    included fragment count
    entity count
    relation/edge count
    graph path count and max depth
    graph expansion count
    evidence/source/coverage/conflict refs
    source excerpt count and bytes
    canonical machine bytes
    rendered member bytes
    total rendered bytes
    exact tokenizer tokens: optional
    wall/CPU/memory/cancellation limits
```

No `unlimited`, negative, overflowed, or host-memory-derived value.

## Canonical hard controls

These are always available and canonical:

- record/entity/relation/path/depth limits;
- source excerpt count/byte limits;
- canonical machine serialized-byte limit;
- rendered member and total UTF-8 byte limits;
- recursion/expansion limits;
- profile-defined maximum string/list sizes.

Time and memory observations are operational. Exceeding them yields explicit cancellation/budget status and never changes a completed semantic ID.

## Token accounting states

```text
ExactPinned
    exact tokenizer profile present and probe-compatible;
    counts can enforce a canonical token budget

Unavailable
    no compatible pinned tokenizer;
    no exact token count or hard token claim

OperationalEstimate
    explicitly noncanonical estimate for telemetry only;
    cannot select/omit canonical fragments or report exact budget compliance

Failed
    tokenizer/profile/input error; exact-token request is NotEvaluated/failed
```

## Tokenizer freeze requirements

An exact tokenizer profile pins:

```text
implementation/library
version and dependency lock digest
vocabulary/model/config bytes and SHA-256
normalization rules
UTF-8/error handling
special-token and prefix/suffix policy
message/template framing policy, if any
separator/newline policy
probe vectors and expected IDs/counts
```

A provider/model marketing name is insufficient. Updating any relevant component creates a new tokenizer profile and invalidates token-dependent artifacts/bundles.

## Selection and token budgets

When exact token budget is required:

1. build candidate machine fragments;
2. render each fragment with the exact rendering profile;
3. apply exact inter-fragment separators/wrappers;
4. tokenize the exact bytes under the pinned profile;
5. include the full dependency group only if the exact group cost fits;
6. record exact per-fragment/group/total counts and tokenizer profile;
7. verify final rendered bytes recount to the same total.

Do not sum isolated fragment counts if boundary-sensitive tokenization makes the sum differ from the concatenated document. The profile defines whether costs are computed by exact incremental concatenation or a proven composable framing scheme.

## Byte budget remains mandatory

Even with a token budget, enforce byte and structure budgets. A tokenizer can map very large or hostile byte sequences unexpectedly; token count is not a memory/security bound.

## Mandatory uncertainty closure

Coverage, conflict, redaction, truncation, generation, and root identity records are mandatory. If they cannot fit:

- use the profile's bounded minimal closure representation; or
- fail with `context_required_closure_exceeds_budget`.

Never omit uncertainty to make the artifact fit.

## Truncation granularity

Allowed deterministic boundaries:

```text
whole optional field
whole relation group
whole path after a complete prefix
whole source excerpt or exact line/span prefix with marker
whole optional Project Map record/section after required header
whole optional skeleton after mandatory root closure
```

Forbidden:

- invalid JSON/UTF-8;
- cutting an ID/digest/token in half;
- dropping a relation endpoint/evidence state;
- cutting source bytes without exact span/truncation metadata;
- cutting text at an iteration/timing-dependent boundary.

## Stable priority under different budgets

For one exact input/profile, larger budgets extend the same canonical candidate ordering. They do not reorder already selected equal-priority fragments unless the rendering/tokenizer profile changes.

## Budget reports

```text
ContextBudgetReport
    requested/effective/system-max profile
    exact counts by category
    selected/omitted groups
    byte usage
    token state/profile/counts
    graph/source expansion usage
    truncation/continuation
    cancellation/time/memory operational metrics
    status
```

Operational elapsed/CPU/RSS fields are excluded from canonical semantic IDs.

## Security cases

- tokenizer vocabulary/config substitution;
- invalid UTF-8 handling mismatch;
- huge combining/Unicode/control text;
- source excerpt with very low token/high byte ratio;
- fragment-boundary token merge/split;
- integer overflow and enormous requested limits;
- high-fanout graph under small token but large structure cost;
- cancellation during rendering/tokenization;
- token estimate incorrectly used as exact selection gate.
