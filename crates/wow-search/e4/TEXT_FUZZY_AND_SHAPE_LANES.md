# E4-A text, identifier-similarity, and structured-shape lanes

**Status:** normative candidate-only retrieval.

## Text lane

The text lane searches only fields approved by `SearchFieldDefinition`.

### Input

```text
safe FtsQueryAst
allowed document/universe/entity kinds
field whitelist
text-lane profile
per-shard/cumulative budgets
```

### Execution

- compile the AST to the frozen FTS5 query form;
- execute separately inside each exact shard;
- retain matched field IDs and origins;
- retain shard-local FTS rank ordinal;
- treat raw rank values as noncanonical diagnostics unless the frozen profile explicitly canonicalizes a finite decimal form;
- never compare raw BM25/FTS values across shards.

### Output signal

```text
TextFieldMatch
PhraseMatch
PrefixTextMatch
MultiFieldTextMatch
```

Text signal includes authority class of each matched field. A Reference contract documentation match and a source comment match are not equivalent.

### Snippets

Snippets/highlights are optional bounded presentation fields:

- produced only by a frozen safe profile;
- exact matched document/field/ranges are retained;
- snippets cannot become source evidence;
- source/private/license policy applies;
- snippets cannot execute HTML/Markdown/control sequences;
- truncation is explicit;
- exact source detail is acquired through owner handles, not snippet text.

## Identifier similarity lane

Uses only identifier fields and a frozen deterministic algorithm.

Candidate features can include:

```text
normalized segment equality
common prefix length
trigram intersection/union counts
bounded Damerau-Levenshtein or Levenshtein distance
separator/case-only difference
qualified-component overlap
```

Rules:

- exact original and approximate normalized values remain separate;
- distance limit is bounded by input length/profile;
- no locale-dependent behavior;
- no arbitrary regex or phonetic algorithm;
- no corpus popularity boost unless a reviewed nonauthority feature is explicitly added later;
- a similarity result is never an alias, typo correction, lineage edge, or replacement fact.

## Structured-shape lane

Input features are explicit typed fields, optionally copied from one exact seed entity:

```text
entity kind
namespace/receiver kind and exact key
parameter count/order/type/optional/variadic facets
return count/order/type/nilability facets
named type/enum/container/literal facets
restriction/Secret/protected facets from ReferenceView
package/load/owner/universal-role facets
registration/hook/state/object/template relation kinds
```

### Match classes

```text
ExactShape
CompatibleShape
PartialShape
ConflictedShape
UnknownShape
```

The profile defines field weights/caps and which differences are hard filters.

Rules:

- missing, nilable, optional, unknown, unsupported, and conflicted are distinct;
- no `any` synthesis;
- no type widening guessed by text;
- Reference restriction fields remain exact profile-bound facts;
- implementation source cannot supply platform restriction authority;
- no cross-generation replacement inference;
- shape compatibility is query-relative candidate evidence only.

## Cross-lane constraints

- Text/fuzzy/shape signals remain below exact identity/name/alias bands.
- Repetition across approximate lanes does not upgrade authority.
- A required structured filter that fails excludes the candidate; a missing capability yields partial/NotEvaluated according to profile.
- Approximate signals preserve their own field origins and coverage.
- One candidate may retain all approximate signals without converting them to one synthetic exact signal.

## Fallback

The lane planner may activate these lanes after exact lanes or directly for declared query classes. Fallback is finite, explicit, and cumulative-budget-bound.

A text query never silently becomes a fuzzy identifier query unless the request/profile declares that transition.

## Budgets

Text:

- term/phrase/token count;
- prefix expansion;
- FTS result rows;
- fields/snippets/bytes;
- query time and memory.

Similarity:

- identifier length;
- trigram set size;
- edit-distance cells/operations;
- candidate prefilter size.

Shape:

- feature count;
- type recursion/depth;
- candidate set;
- relation lookups;
- explanation bytes.

## Failure isolation

One shard/lane failure is recorded. Required-lane failure blocks completion. Optional-lane failure may produce an explicit partial result; it cannot be omitted from explanation.

## Evaluation

Measure each lane separately:

- recall at k;
- candidate-set size;
- false exact/alias/lineage/replacement assertions, which must remain zero;
- latency/memory/bytes;
- robustness to collisions, long text, repeated terms, case changes, separators, confusable Unicode, and adversarial query syntax.
