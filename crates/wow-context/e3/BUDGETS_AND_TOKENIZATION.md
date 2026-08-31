# E3-A context budgets and tokenizer accounting

**Status:** normative multi-axis budget and token-count contract.

## Principle

Context size is not one guessed token number. E3-A always tracks deterministic structural and byte measures. Token counts become exact only under an explicit pinned tokenizer profile over exact canonical bytes.

## Canonical budget axes

```text
roots
entities and skeleton records
members/source skeleton nodes
relations and reason-path edges
query expansions and path depth
source/evidence handles
conflict/coverage/loss/omission records
source excerpts, bytes, and lines
structured model nodes/fields
rendered UTF-8 bytes
Unicode scalar count
exact tokenizer tokens: optional profile-bound
```

No single axis substitutes for all others.

## Budget specification

```text
ContextBudgetSpec
    profile ID/version
    global hard limits
    mandatory reserve limits
    per-root/per-lane/per-artifact limits
    soft target limits: optional
    exact tokenizer profile/budget: optional
    estimated-token profile/range: optional separate
    overflow/truncation/continuation policy
    canonical digest
```

Requested overrides must be within system/profile maxima and cannot disable mandatory evidence/security fields.

## Mandatory reservation

Reserve before optional expansion:

```text
input/generation/profile identity
root identity and requested fields
mandatory evidence/source handles
coverage/conflict/NotEvaluated blockers
loss/omission/stopping state
continuation metadata when incomplete
```

If mandatory records alone exceed the requested budget, return typed budget failure or a profile-defined minimal blocker-only artifact; never silently drop them.

## Budget consumption

Every stage reports deterministic deltas:

```text
Project Map section/item
L0/L1 skeleton/member/relation
query/path expansion
source/evidence record
source excerpt
loss/omission/stopping/continuation
renderer output
```

Consumption uses actual serialized/structured sizes for completed records where possible. Estimates are used only for planning and labeled separately.

## Planning estimates

`ContextDetailRoute` and work items may include cost estimates based on frozen profiles:

```text
expected entity/edge/member counts
known source span length
historical exact corpus distribution
fixed renderer overhead
pinned tokenizer count for already materialized candidate bytes
```

Estimates cannot derive from repository popularity/model intuition and cannot change semantic eligibility.

## Exact byte accounting

Canonical output bytes are measured after deterministic serialization under the selected renderer profile. Semantic bundle metrics can also count canonical semantic JSON bytes independently of human renderer bytes.

Rules:

- UTF-8 byte length exact;
- line endings/profile fixed;
- Unicode scalar count exact under decoded valid UTF-8;
- no locale-dependent counting;
- compression/archive size supplemental, not context size authority.

## Exact tokenizer profile

Required fields:

```text
tokenizer implementation/package/repository
exact version/revision
vocabulary/model/merge file digests
normalization and pretokenization config
special-token/add-prefix-space/chat-template policy
input byte/string encoding contract
counting API/probe fixture
canonical digest
```

An API provider/model display name without these inputs is not a tokenizer profile.

## Exact token count

```text
ExactTokenCount
    tokenizer profile ID
    exact artifact/renderer bytes digest
    special/template policy ID
    token count
    optional token ID digest/count fixture
    canonical digest
```

Count after final renderer bytes. Semantic-model tokens and rendered-context tokens are different subjects.

## Token estimate

When no exact tokenizer is available:

```text
TokenEstimate
    estimate profile ID/version
    artifact byte/scalar/word measures
    estimated range or point
    calibration corpus/profile
    uncertainty/classification
    explicitly_exact = false
```

Do not label it “tokens used” without the estimate qualifier.

## Multiple consumers

One semantic bundle can have distinct renderer/tokenizer metrics per consumer profile. Budget negotiation selects exact target profiles; do not optimize for an unspecified “LLM.”

## Overflow policy

At a limit:

1. finish the current atomic mandatory record or reject before it;
2. preserve budget counters;
3. emit exact truncation/omission/stopping records;
4. retain mandatory blockers/evidence;
5. build deterministic continuation frontier if supported;
6. set bundle/request status partial/truncated;
7. no background continuation.

Do not cut UTF-8, source excerpt, JSON, semantic record, or evidence link mid-unit.

## Per-lane fairness

A profile may reserve budget across explicit lanes/roots to prevent high-fanout optional branches from consuming everything. Fairness is deterministic and profile-defined:

```text
mandatory first
request-explicit roots/lanes
minimum per selected root/lane
stable priority rounds
remaining global budget
```

No timing-based first-completed-wins.

## Budget inheritance

Continuation carries used/reserved/remaining state and cannot reset a total request budget unless the caller explicitly starts a new request/profile. Cursor integrity binds it.

## Source excerpt budget

Track separately:

```text
excerpt count
source bytes
source lines
context expansion before/after target span
license/privacy/security filtered bytes
sanitization/truncation markers
```

A source excerpt cannot consume evidence/blocker reserve.

## Report budgets

Loss/omission/metrics reports themselves are bounded. If a detailed report exceeds budget:

- retain decisive blockers and exact totals/digests;
- provide continuation/detail handle where supported;
- bundle cannot claim complete loss disclosure for affected scope;
- never drop all omission metadata to fit output.

## Determinism

Equivalent exact input/request/profile yields identical planning estimates, actual budget consumption, cutoff item, omissions, continuation, output byte counts, and exact tokenizer counts.

## Required tests

- every structural/byte axis;
- mandatory records exceed budget;
- optional high-fanout branch and lane fairness;
- atomic record/UTF-8/source span boundary;
- exact semantic versus renderer bytes;
- exact pinned tokenizer vectors and special-token policy;
- tokenizer version/vocabulary/config change;
- unlabeled estimate mutation fails;
- multiple renderer/tokenizer profiles;
- continuation preserves budget state;
- loss/omission report budget retains blockers;
- 1/2/N cutoff determinism;
- no background continuation.

## Hard stops

- no guessed universal token count;
- no exact token claim without pin/config/input digest;
- no budget reset through continuation;
- no mandatory evidence/security omission;
- no timing-dependent branch allocation;
- no partial semantic record/source codepoint;
- no smaller-output optimization that changes truth/proof limits.
