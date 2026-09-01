# E5-A label schema and independent review

**Status:** normative label/evidence contract. Durable reviewer authorization and promotion workflow are E5-B.

## Label objects

Labels apply to exact universal outputs, not vague prose:

```text
entity kind/role and semantic key ingredients
relation kind, source, target and qualifiers
required typed attributes
maximum valid recognizer confidence
ambiguity/cardinality expectations
closed no-match scope when Negative
required capability/coverage state
```

## Label classes

### `Positive`

The exact universal proposal is expected and independently supported.

### `Negative`

The proposal must not exist in the exact closed scope. Negative requires decisive evidence and complete relevant label/source/fact coverage.

### `Possible`

The input is intentionally ambiguous/dynamic and only a `Possible` output is acceptable.

### `NotEvaluated`

A required capability or exact evidence is intentionally unavailable; the recognizer must not fabricate a conclusion.

### `Unknown`

Review cannot establish truth. Unknown is excluded from TP/FP/FN/TN and never coerced to Negative.

### `Conflict`

Independent evidence/reviewers support incompatible labels. The conflict remains explicit until a versioned resolution.

## Review inputs

Reviewers may inspect:

- exact materialized source under privacy/license policy;
- TOC/XML/load facts;
- Emmy/analyzer facts and source maps;
- exact graph/reference facts relevant to the universal role;
- source handles and evidence;
- coverage/conflict/NotEvaluated state;
- label schema and decisive-clause checklist.

By default blind label review does not show candidate-pack output, current recognizer result, aggregate metrics, search/model suggestions or donor popularity.

## Reviewer independence

The label manifest records:

- reviewer role and qualification profile;
- whether the reviewer authored/tuned the candidate pack;
- which outputs were hidden;
- exact evidence reviewed;
- structured decision/reason codes;
- disagreements and resolution;
- authorization/attestation refs when E5-B implements them.

A pack author can propose a label but cannot alone establish independent test/holdout truth under profiles requiring separation.

## Evidence hierarchy

For structural recognizers:

1. exact normalized fact and source evidence under the target generation;
2. exact project/graph relationships independently produced by owners;
3. source implementation and explicit public structural convention;
4. reviewed external documentation only as supporting evidence;
5. comments/README prose as untrusted nonauthority context;
6. search/model/community suggestions as Candidate context only.

Labels do not become WoW API/runtime authority. Patch-sensitive platform behavior still uses exact Reference/runtime/KB routes.

## Decisive-clause checklist

Every Positive/Negative label states which facts are decisive:

```text
exact resolved callee/receiver/member/literal
same scope/owner/package/XML document
required producer or registration relation
required control-flow relation supplied by analyzer
required TOC declaration or XML edge
closed searched partition for negative clause
confidence/coverage constraints
```

This checklist drives near-miss and sensitivity mutations.

## Multi-label and ambiguity

An example can legitimately expect multiple universal entities/relations. Labels state cardinality and whether alternatives are compatible, Possible, conflicting or mutually exclusive.

Do not force one label because evaluation code expects one target. One-to-many and competing matches remain explicit.

## Label versioning

Changing any expected output, confidence ceiling, decisive evidence, ambiguity, negative scope or coverage requirement creates a new label-set version and corpus manifest. Prior labels/results remain reproducible.

Corrections record:

```text
prior label ID
new label ID
reason/evidence
review decisions
affected split/run/report IDs
whether prior evaluation is invalid, superseded or still valid for its label generation
```

## Label conflict resolution

Resolution requires explicit reviewed evidence and produces a new accepted label version. It never deletes conflicting historical decisions. Until resolved, the affected case is Conflict/Quarantine or evaluated under an explicit conflict profile.

## Label leakage prohibitions

- expected IDs/literals cannot be inserted into pack conditions solely because they appear in labels;
- corpus expected outputs cannot be read by runtime matcher;
- label/split IDs cannot affect clause evaluation or ranking;
- evaluation reports cannot alter pack bytes within one run;
- labels cannot be weakened after seeing failures without a new version and contamination record.

## Review security/privacy

Review notes are bounded untrusted text. They cannot define clauses, execute source, authorize tools or be emitted as graph facts. Sensitive source may use handle-only/local reviewer workflows and cannot enter public fixtures.

No private keys, access tokens or credentials are stored in label fixtures.

## Label validation

- exact example/source/fact/evidence refs resolve;
- output kind/relation registered in graph profile;
- semantic keys/attributes/capabilities well typed;
- confidence ceiling valid for recognizer output;
- Negative has complete closed-scope evidence;
- Possible/Unknown/NotEvaluated/Conflict distinctions preserved;
- reviewer independence/visibility policy satisfied;
- no current recognizer/model/search output as authority;
- no unresolved privacy/license blocker for label evidence;
- canonical bytes/digest reproduce.
