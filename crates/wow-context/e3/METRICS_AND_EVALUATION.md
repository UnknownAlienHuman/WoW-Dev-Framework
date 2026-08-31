# E3-A context metrics and evaluation

**Status:** normative correctness, usefulness, compression, redundancy, and consumer-evaluation contract.

## Principle

A context artifact is not good merely because it is short, readable, or liked by a model. E3-A measures independent properties and never trades correctness/evidence closure for a lower token count without an explicit loss record and policy.

## Evaluation dimensions

```text
structural correctness and mandatory-record recall
evidence/provenance closure
coverage/conflict/blocker honesty
request relevance
redundancy and duplicate avoidance
compression and detail efficiency
budget adherence and cutoff stability
continuation completeness/stability
source-excerpt faithfulness and security
renderer/tokenizer consistency
consumer task utility under a pinned protocol
performance/resource measurements: supplemental
```

No single aggregate score is correctness authority.

## Frozen evaluation corpus

```text
ContextEvaluationCorpus
    corpus ID/version
    exact synthetic and pinned real project/publication/reference inputs
    request/task classes
    mandatory expected entity/relation/member/evidence/blocker records
    forbidden/invented record patterns
    acceptable compact/loss/partial policies
    expected stopping/continuation records
    source excerpt expectations
    renderer/tokenizer profiles
    consumer utility protocol
    license/provenance
    canonical digest
```

The corpus is reviewed data. It cannot be rewritten automatically by the implementation/tests.

## Fixture classes

### Synthetic exact fixture

Small closed source/project/graph/reference state with exhaustive expected records. Used for exact recall, negative, conflict, partial, truncation, continuation, and determinism.

### Pinned real user-owned project fixture

A bounded exact `UnknownAlienHuman` addon repository snapshot, initially the E2-C `roth-ui` pin or a later reviewed fixture. Used for scale, navigation, load/ownership/source-unit coverage, compression, and repository-name mutation. It is not API or architecture authority.

### Mutation corpus

```text
repository/package/path/local-name changes
fact/relation/source-handle deletion or conflict
high fanout and cycles
partial/failed coverage
malicious source comments/prompts
source and budget size extremes
renderer/tokenizer/profile changes
publication-generation changes
```

## Structural correctness

Compare included semantic records to exact expected corpus records:

```text
mandatory expected included
mandatory forbidden absent
entity/relation/member/source-node identity exact
lane/direction/path/direct-edge distinctions preserved
confidence/provenance/coverage/conflict exact
no cross-generation/universe merge
```

Report exact missing, extra, changed, and unclassified records.

## Mandatory-record recall

```text
MandatoryRecall
    expected mandatory record count/digest
    included exact count/digest
    missing IDs/reasons
    recall ratio when denominator exact
    blocker classification
```

A record intentionally excluded under an accepted partial/loss policy is still reported, not silently counted as present.

## Evidence closure

For each material record/field:

- at least one exact evidence link or deterministic derivation closure;
- all required generations and source handles valid;
- producer/confidence/coverage/conflict preserved;
- no orphan evidence/source/excerpt refs;
- dedup group retains every contributing evidence ID.

Metrics:

```text
material fields total
fields with exact closure
orphan/missing/stale links
unique source/evidence records
reused evidence link count
```

## Honesty metrics

Count and validate:

```text
partial/unknown/failed/NotEvaluated input partitions
conflicts and ambiguity groups
Possible/Candidate records included/excluded
loss/omission/truncation/stopping records
unsupported/deferred detail
exact domain-authoritative absence versus nonauthoritative empty
```

A “clean-looking” artifact that hides blockers fails even if task utility appears high.

## Request relevance

Relevance is evaluated against a frozen request/task definition, not model intuition alone.

Record classes:

```text
mandatory relevant
optional relevant
contextually supporting
out-of-scope/redundant
forbidden/private/unsafe
```

Relevance labels come from reviewed corpus rules or exact task profile. Optional external human/model review can supplement but cannot change mandatory correctness expectations automatically.

Metrics:

```text
included mandatory/optional/supporting counts
out-of-scope included count/bytes
relevant records omitted under budget
per-lane and per-root allocation
```

## Redundancy

Redundancy is semantic duplication under a frozen equivalence policy:

```text
exact duplicate presentation records
repeated evidence already reachable through sidecar
repeated identical reason paths/prefixes
repeated source excerpts/overlap
renderer-only duplicate headings
```

Do not mark distinct source occurrences, competing assertions, different confidence/coverage, or task-relevant paths redundant.

Metrics:

```text
raw candidate record/byte count
included record/byte count
deduplicated count/bytes
retained evidence multiplicity
false-dedup mutation failures
```

## Compression

Separate baselines:

```text
complete selected source bytes/lines
project logical record count/bytes
graph assertion/view/query result count/bytes
full requested uncompressed context candidate
Project Map/L0/L1/bundle output bytes
exact tokenizer tokens per pinned profile when available
```

Report ratios with exact numerator/denominator subjects. A high compression ratio is not success if mandatory recall/evidence/security fails.

## Detail efficiency

```text
new mandatory/relevant records per expansion step
new evidence IDs per query/edge/source byte
duplicate/no-new-evidence rate
source excerpt bytes per resolved task question
budget consumed by mandatory versus optional detail
```

These metrics can tune future profiles only through reviewed changes, not adaptive production guessing in E3-A.

## Budget adherence

Validate exact:

- no axis exceeds hard limit;
- mandatory reserve retained;
- cutoff occurs at an atomic stable work item;
- used/reserved/remaining arithmetic reconciles;
- omitted frontier/continuation matches cutoff;
- exact token counts match pinned tokenizer when active;
- estimates are labeled nonexact.

## Continuation evaluation

For cumulative context after all permitted continuations:

```text
same final semantic bundle as one sufficiently large request under the same profile, where policy declares equivalence
no duplicate/lost records between pages
no budget reset
stable order and frontier
same stop reasons and exact generation
no continuation after RequestedComplete/NoChange
```

When a per-page renderer differs from cumulative rendering, semantic closure still matches.

## Source excerpt evaluation

- exact content digest and span;
- faithful bytes/text under declared normalization;
- no stale-generation path fallback;
- no prompt/directive/container escape;
- license/privacy/security gates;
- no paraphrased/reconstructed code;
- source budget metrics correct.

## Consumer utility protocol

```text
ContextConsumerTask
    task ID/class
    exact input publication/request/context profiles
    allowed context artifact(s)
    expected answers/actions and evidence citations
    forbidden unsupported claims
    evaluation method and scoring rubric
    consumer implementation/model/human pin when applicable
    repetitions/randomization policy
```

Possible tasks:

```text
locate project bootstrap and direct load path
identify exact native versus custom signal registration
trace a state root to direct readers/writers
locate API-use source and exact reference entity
choose the next evidence/source detail route
answer whether a requested capability is partial/conflicted
```

Do not use open-ended “write the project” as an E3-A context test.

## External model evaluation

Allowed only as supplemental evidence through an external harness:

- exact model/provider/version/config/reasoning/prompt pin where possible;
- no network/model call inside `wow-context` crate;
- artifact supplied without hidden extra repository context;
- multiple runs and variance recorded;
- expected evidence citations and forbidden claims checked structurally;
- model score cannot override deterministic correctness, evidence, coverage, or security failures;
- source prompt injection behavior explicitly tested;
- cost/token timing metrics supplemental and date/profile scoped.

## Acceptance policies

A profile can define thresholds, but mandatory hard gates include:

```text
100% mandatory structural/evidence records unless exact accepted partial input prevents them and blockers are present
0 forbidden invented semantic records
0 cross-generation/universe/provenance upgrades
0 hidden mandatory conflicts/partial/truncation
0 source/security/private-data violation
byte-identical deterministic semantic outputs for equivalent input
exact budget reconciliation
```

Relevance/redundancy/compression/consumer utility thresholds cannot waive hard gates.

## Profile tuning

A change to priority, grouping, field set, lane, cost, budget, source, or renderer profile requires:

1. exact before/after corpus reports;
2. no hard-gate regression;
3. classify every added/removed/changed record;
4. rerun repository/path/name mutations;
5. rerun tight-budget and continuation cases;
6. update profile/version/contracts/fixtures/checksums;
7. retain last-known-good evaluation baseline.

No online adaptive profile mutation in E3-A.

## Performance measurements

Measure with exact corpus/hardware/runtime/profile:

```text
build and expansion wall/CPU time
peak/resident memory where available
queries/rows/edges/source bytes read
serialization/tokenizer time
cache use: only if later explicit cache exists
```

Performance is supplemental until a reviewed gate is set. Average-only conclusions are insufficient; report distributions/outliers where meaningful.

## Evaluation report

```text
ContextEvaluationReport
    exact corpus/input/request/profile/artifact IDs
    structural missing/extra/changed records
    mandatory recall and evidence closure
    honesty/blocker results
    relevance/redundancy/compression/detail-efficiency metrics
    budget/token/source/continuation results
    consumer utility outcomes and supplemental model/human evidence
    performance observations
    hard-gate and profile-threshold decisions
    error/loss refs
    canonical digest for deterministic components
```

## Required tests

- exact closed synthetic corpus;
- pinned real-project scale/navigation corpus;
- every hard-gate mutation;
- false dedup versus true duplicate;
- compression smaller but mandatory record missing;
- relevance includes out-of-scope high-centrality node;
- partial/conflict artifact remains honest;
- continuation cumulative equivalence;
- exact token versus estimate;
- malicious source prompt consumer task;
- external model variability and no-authority rule;
- profile tuning before/after classification;
- performance result does not alter semantic output;
- deterministic report order/digest.

## Hard stops

- no single opaque quality score as correctness;
- no token/compression optimization over evidence recall;
- no model preference as production ranking/authority;
- no automatically rewritten golden corpus;
- no hidden extra context in consumer tests;
- no missing blocker counted as relevance success;
- no performance claim without exact corpus/profile/run data.
