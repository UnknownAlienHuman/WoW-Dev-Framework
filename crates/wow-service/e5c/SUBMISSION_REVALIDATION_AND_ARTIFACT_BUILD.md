# E5-C submission revalidation and core artifact build

**Status:** normative.

## Input

E5-C accepts one exact E5-B `PromotionSubmission` with expected digest, schema, target publication profile, and retention guards. It never selects the newest, best, highest-metric, first, last, sole, or same-name submission.

## Independent revalidation

Before artifact construction, reacquire and validate:

```text
submission bytes/state/profile
exact E5-A candidate artifact and candidate pack bytes
corpus/provenance/labels/split/fact snapshots
visible and holdout run/case/mutation/metric reports
graph/security/determinism/deactivation reports
review envelopes, use-time authorization and quorum/independence
holdout grants, execution receipts, audit chain, disclosure and consumption
license/privacy/notice and generalization scope
all blockers/conflicts/partial/NotEvaluated/nonclaims
```

E5-C cannot trust E5-B's `ReadyForE5CReview` label alone. It validates the exact references and current revocation/retention state under the E5-C profile.

## Blockers

Artifact build is blocked by any required:

```text
missing/stale/superseded/revoked artifact
candidate bytes or implementation/profile mismatch
hard-gate failure
provenance leakage or unsupported generalization
invalid graph output or producer namespace
unauthorized/incomplete/conflicting review
missing/unauthorized/incompatibly consumed holdout
license/privacy/notice restriction
invalid deactivation/stale-closure plan
partial/conflict/truncated/NotEvaluated required evidence
retention or audit failure
```

No aggregate score or authorized reviewer can override a semantic, security, license, graph, or holdout blocker.

## Artifact build

The core artifact is newly constructed and receives a new identity:

```text
validate exact semantic pack projection through wow-recognizers
-> validate registered graph outputs/producer namespace through wow-graph
-> bind production execution/compatibility/reindex profiles
-> include exact candidate lineage and submission references
-> include deactivation/stale-closure plan
-> include blockers/nonclaims manifest
-> canonicalize immutable bytes
-> validate byte/digest/schema closure
```

`trust_class = core` is allowed only after this procedure. It is not copied from E5-A metadata.

## Production semantics

The artifact may contain only reviewed universal E2-B operator/rule semantics and registered graph outputs. Repository, addon, owner, path, popularity, labels, split, reviewer, holdout membership, metric score, search/model output, or canary cohort identity cannot enter matcher clauses, captures, confidence, budgets, or producer keys.

## Candidate lineage

The artifact records exact originating candidate/submission identities but does not overwrite them. Failed, rejected, or superseded candidates remain immutable historical evidence.

## Validation operation

`core_pack_artifact_validate` is read-only. It verifies identity closure, canonical bytes, production profile, operator/rule/literal schemas, universal invariance constraints, graph registry/output compatibility, producer namespace uniqueness, deactivation/closure plan, blockers/nonclaims, and retention. It never edits/rebuilds the artifact in place.

## Nonclaims

Building/validating an artifact does not mean:

```text
signed
published
active
default
canary-assigned
runtime-verified
safe for every addon/build/profile
free of all false positives outside admitted evidence
publicly distributable
```
