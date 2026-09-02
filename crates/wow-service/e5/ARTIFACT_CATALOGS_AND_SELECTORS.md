# E5-B artifact catalogs, selectors, and acquisition

**Status:** normative.

E5-B operates only on exact immutable retained artifacts. Accepted selectors are:

```text
ExactArtifact
    artifact kind + ID + expected digest/schema/profile

ForExactBinding
    exact owner generation + exact profile tuple

ExactRun
    run ID + expected request/result digest

ExactSubmission
    submission ID + expected candidate/profile digest
```

There is no `latest`, `best`, `highest score`, `most recent`, `default`, `first`, `last`, or `sole candidate` selector.

Catalog lookups return exactly one of:

```text
UniqueEligible
None
MultipleConflicting
NotEvaluated
Failed
```

A raw row is not eligible until kind, schema, digest, profile, owner, universe, generation, retention, validation, privacy/license, coverage, conflict, and supersession state close.

Canonical acquisition order is:

```text
1 durable operation/idempotency record
2 exact project/source/analyzer/fact publication views
3 exact corpus/candidate-source artifacts
4 exact provenance/label/split artifacts
5 exact pack/fact-snapshot artifacts
6 exact prior run/candidate/deactivation artifacts when requested
7 graph validation resources
8 review authorization resources
9 holdout authorization/vault resources when requested
10 retention and audit resources
```

Acquire only required resources while preserving relative order; close in exact reverse order. A missing required retained artifact, conflicting catalog candidates, or incomplete compatibility yields typed unavailable, conflict, or `NotEvaluated`. The service never substitutes another generation.

Before returning any durable run, review, holdout, continuation, or promotion-submission handle, E5-B obtains retention receipts for every referenced artifact. Selectors are structured IDs, never paths, URLs, SQL, expressions, source text, or model prompts.