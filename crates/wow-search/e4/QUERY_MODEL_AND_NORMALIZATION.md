# E4-A query model and normalization

**Status:** normative.

## Query classes

```text
ExactEntity
ExactCanonicalName
ExactAlias
NamespaceMember
IdentifierPrefix
DocumentationText
IdentifierSimilarity
StructuredShape
RelatedToExactSeed
MixedStructured
```

The caller declares the class. E4-A does not infer intent through an LLM or hidden classifier.

## SearchRequest

Required fields:

```text
exact SearchUniverseSetId
query class
explicit enabled and required lane IDs
allowed universes/entity kinds
confidence/provenance/coverage policy
ranking and explanation profiles
hard result/candidate/lane/graph/text/time/memory budgets
privacy/snippet/detail policy
```

Query payload fields are closed:

```text
exact entity key/ID
exact canonical name
exact alias string
namespace/member or receiver/method
identifier prefix
bounded literal text
structured signature/type/restriction/load/role features
exact graph seed entity IDs
```

Unknown fields are rejected.

## Identifier normalization

### Exact representation

- validate UTF-8 and bounded length;
- apply only the frozen exact Unicode normalization profile;
- preserve case and punctuation required by owner identity;
- preserve original input and normalized digest.

### Approximate representation

A distinct profile may:

- apply a frozen case fold;
- segment camelCase, PascalCase, snake_case, namespace separators, and digits;
- normalize declared separators;
- produce bounded trigrams;
- calculate bounded edit distance.

Approximate normalization never changes the exact query value or match class.

## Literal text normalization

- UTF-8 validation;
- frozen Unicode normalization;
- bounded whitespace normalization;
- field-appropriate punctuation/token handling;
- no locale/host-dependent behavior;
- no hidden stopword/stemming/synonym expansion;
- no model keyword extraction.

Original and normalized digests are retained.

## Safe FTS query AST

```text
FtsQueryAst
    AllTerms([LiteralTerm])
    AnyTerms([LiteralTerm])
    ExactPhrase([LiteralTerm])
    PrefixTerm(LiteralPrefix)
    ColumnScoped(field_id, child)
    And(children)
    Or(children)
```

The profile may support only a subset.

Not supported in E4-A:

```text
raw MATCH text
NEAR supplied by caller
unbounded wildcard
column names supplied as raw text
unary NOT over an open corpus
SQL operators/functions
tokenizer directives
auxiliary-function invocation
```

Every term is data, not syntax. The compiler escapes/quotes it according to the frozen FTS profile and binds parameters through store-owned operations.

## Structured filters

Filters use typed IDs/enums/integers only:

```text
universe
entity kind
namespace/receiver
parameter/return count ranges
exact type/restriction facets
package/load/owner/universal role
confidence/provenance classes
```

No arbitrary JSON path, field name, expression, SQL, regex, callback, or source predicate.

## Lane eligibility

For each lane, the planner records:

```text
Enabled
ExcludedByRequest
UnsupportedQueryClass
MissingShardCapability
PartialOwnerCoverage
ConflictBlocked
PrivacyDenied
BudgetUnavailable
```

A required lane that cannot execute makes the result `NotEvaluated`, partial, or failed according to the frozen policy. It is never silently skipped.

## Fallback rules

Fallback is explicit and finite. Example:

```text
ExactCanonicalName
-> if exact miss is nonauthoritative or request allows candidates:
   IdentifierPrefix
-> IdentifierSimilarity
```

No fallback may:

- change universe/profile/generation;
- convert case-folded/fuzzy text into exact;
- enable a lane excluded by privacy;
- exceed cumulative budgets;
- rerun against current;
- introduce lineage/replacement semantics.

## Query plan

The plan freezes:

- exact shard order;
- lane execution order and parallelism class;
- per-lane budgets;
- candidate limits;
- required coverage;
- fusion/explanation profile;
- result-set materialization policy;
- cancellation checkpoints.

Equivalent normalized requests produce the same plan.

## Query injection tests

Include terms containing:

```text
quotes
apostrophes
parentheses
minus/plus
asterisk/caret/colon
FTS keywords
column-like prefixes
SQL fragments
NUL/control characters
invalid UTF-8
very long repeated tokens
Unicode confusables and combining sequences
```

They must remain literal data or be rejected by the declared input policy.

## Normalized request identity

`NormalizedSearchQueryId` binds exact original-data digest, normalized fields, query AST, filters, lane eligibility inputs, and profile versions. It excludes host, clock, worker count, and physical shard paths.
