# Explicit native Reference identifier similarity

The [Reference query](SEARCH_REFERENCE.md) supports a separately selected
Candidate-only similarity mode over its genuine captured qualified/short names:

```json
{
  "schema": "wow-search/native-reference-query/1",
  "selectors": {
    "identifier_similarity": {
      "identifier": "Reed",
      "field": "short_name",
      "case_policy": "preserve",
      "max_distance": 1,
      "max_prefiltered_documents": 64,
      "max_distance_cells": 8192
    }
  },
  "enabled_lanes": ["identifier_similarity"],
  "required_lanes": ["identifier_similarity"],
  "max_scanned_documents": 128,
  "max_candidates": 16,
  "max_signals": 64,
  "max_result_bytes": 1048576
}
```

Use the exact retained handle and ordinary `search reference query` CLI flags.
No implicit fallback changes an exact or text query into similarity. An explicit
selector selects `wow-search/native-callable-name-member-shape-similarity/1`
and `wow-search/reference-authority-ordinal-fusion-similarity/1`. Without it,
existing query profiles/bytes/behavior remain unchanged, including a formerly
required-but-unavailable similarity lane. No shard rebuild or new index/backend
is required: this query uses its captured typed identifier fields.

`field` is `short_name`, `qualified_name` or `both`. Fields preserve separate
origins, including equal text from a global name and its qualified-name field.
The original owner values and exact lanes never change. `case_policy` is
`preserve` or `ascii_fold`; the latter lowercases ASCII only. Non-ASCII Unicode
scalar values remain unchanged. There is no Unicode case folding, NFC repair,
separator rewriting, stemming, synonym/phonetic dictionary or locale behavior.

The frozen algorithm is two-row banded scalar Levenshtein
`wow-search/scalar-banded-levenshtein-k4/1`: insert/delete/substitute cost one.
It compares scalar values, not UTF-8 bytes, and does not treat transposition as
one edit. Query length is 2–128 scalars, distance is 0–min(4, query length), and
the safe length lower bound admits candidate fields of at most 132 scalars.
Only recurrence cells are charged, before computation; initialization is bounded
by those hard lengths. Cancellation is checked through the scan and matrix.

Length differences beyond the selected distance are complete safe rejections.
Other eligible documents consume `max_prefiltered_documents`; all compared
fields share `max_distance_cells`. General scan/candidate/signal/result caps
remain in force. Exhaustion is explicit `truncated`, never a clean no-hit or
authoritative absence. A required unavailable lane cannot be hidden by a fuzzy
hit. Unknown fields, algorithms, case/field choices, excessive lengths/distances,
zero bounds and controls reject.

Signals retain original/normalized query and identifier digests, the actual
owner field/record/source origin, normalization/algorithm profile, threshold,
distance, charged cells and an explicit ASCII case-only observation. The result's
`identifier_similarity_work` records prefiltered documents, compared fields,
safe length rejections, matrix cells, matched signals and bounded omissions.
Source and acquisition omissions/conflicts remain in the original owner summary.

Similarity is always band `8`, including distance-zero or case-only matches.
It does not establish an exact name, alias, typo correction, caller intent,
lineage, replacement or runtime fact. Exact bands dominate any approximate sum.
Lane ordinals sort by distance, normalized identifier bytes, exact entity key
and fixed field order. The bounded best-hit set retains the closest ordinals
rather than the first encountered witness. Signal/candidate caps remain explicit
truncation; all retained signals feed the existing integer fusion and explanation.

Existing [Reference pages](SEARCH_REFERENCE_PAGINATION.md) retain the exact query,
new profile IDs and full manifest. Every replay uses the same scalar algorithm
and bounded request, must reproduce all distances/work/result bytes, and charges
the existing cumulative page/replay budgets. Ordinary retirement/GC and explicit
session release keep the original three-object retention contract.

The algorithm/profile is generic query logic over pinned owner data. It does not
change Ketho source normalization, annotations or type lowering, and does not
qualify current WoW source/runtime. Other document kinds, explicit aliases,
graph lanes, federation and full E4 corpus/fault/platform acceptance remain open.
