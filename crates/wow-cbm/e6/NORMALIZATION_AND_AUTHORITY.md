# E6-A normalization, score handling, and authority ceiling

**Status:** normative.

## Normalization pipeline

```text
validate exact response/state/schema
-> preserve bounded raw fields and unknowns
-> map provider fields through reviewed adapters
-> normalize identifiers/strings/numbers under explicit profiles
-> create provider-local candidate identity
-> create UnverifiedProviderLocator records
-> record every dropped/transformed/defaulted/unsupported field
-> attach provider/bridge coverage and conflicts
-> force semantic_candidate + Candidate authority
-> canonicalize deterministic candidate/result bytes
```

Normalization cannot fill missing fields from model knowledge, local project state, another provider, or same-name candidates.

## Authority ceiling

Every candidate, explanation, result set, and artifact contains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
source_verification = unverified unless a later E6-B owner receipt exists outside E6-A artifact
```

The bridge rejects any normalized record that omits or exceeds this ceiling.

## Provider labels

Retain labels such as `exact`, `verified`, `authoritative`, `stable`, `fresh`, `high confidence`, or `production` only as quoted provider fields. They do not alter bridge confidence, provenance, source verification, or coverage.

## Score/rank model

A score record binds:

```text
provider descriptor/capability/profile
provider-local field name/type/unit/range/direction
raw value and normalized display value
rank/tie/order metadata
missing/invalid/conflict/loss state
```

Rules:

- no raw numeric comparison across providers or incompatible score profiles;
- no universal 0–1 confidence conversion unless a reviewed exact transformation states it is descriptive only;
- top-1, sole result, high score, large gap, repeated rank, or multiple similar provider signals never promote authority;
- deterministic ordering within one result set follows the exact provider/adapter profile plus stable candidate tie keys;
- an invalid score can be omitted from ordering only with an explicit loss record; candidate remains available if otherwise valid.

## Unknown and lossy fields

States remain distinct:

```text
Missing
ExplicitNull
UnknownField
UnsupportedValue
InvalidValue
TransformedWithLoss
KnownValue
Conflict
NotEvaluated
```

No coercion to empty string, zero, false, complete, safe, or absent.

## Candidate identity

Candidate identity includes exact provider/state/query/result-local identity and canonical normalized payload. It excludes process ID, clock, network timing, cache hit, worker order, local path, and E6-B mapping/selection results.

Two providers returning equal text create distinct candidates. Two state generations returning equal text remain distinct unless an explicit comparison records equality.

## Explanations

`explain_external_candidate` returns exact raw field origins, normalization transformations, provider-local scoring/rank semantics, locator fields, coverage/conflicts/loss, and authority nonclaims. It does not explain local source truth or why the candidate is correct.

## Artifact subset

`build_external_candidate_artifact` may include an explicitly supplied list of exact candidate IDs. The builder cannot choose top/best/sole candidates. Inclusion is an audit fact, not a verified selection or local mapping.

## Comparisons

Descriptive comparison can report:

```text
same/different normalized candidate IDs
same/different provider-local locators or snippets
rank/order changes within compatible profile
field additions/removals/loss changes
coverage/state/query/profile incompatibility
```

It cannot name a winner, authoritative provider, replacement, intended entity, or globally better result set.