# E3-B metrics and frozen evaluation

**Status:** normative evaluation contract. Metrics and evaluation results are later records and never semantic evidence or inputs to `ContextSemanticPackId`.

## Goals

Measure whether Project Maps, skeletons, and context packs are compact, faithful, complete for declared fields, reproducible, secure, and useful for frozen tasks without promoting subjective scores into correctness.

## Metric classes

### Structural

```text
map nodes/edges/groups
L0/L1 roots, facets, members
selected direct relations and reason paths
unknown/collapsed/omitted regions
source/reference/evidence/conflict records
continuation pages and expansion rounds
```

### Budget

```text
mandatory and optional canonical bytes
renderer bytes
token class/count/estimate/bound
per-section and per-universe allocation
candidates selected/omitted/unenumerated
source excerpt bytes/lines
```

### Coverage and fidelity

```text
requested fields satisfied/partial/blocked
origin/evidence closure rate
confidence/coverage/conflict preservation
projection-loss and omission counts
source excerpt digest/range round-trip
negative-authority misuse count
```

### Determinism

```text
semantic ID/byte equality across 1/2/N workers
shuffled query/storage order equality
cold/warm cache equality
continuation page-chain equality
renderer byte equality
```

### Operational

```text
wall/CPU time
peak memory
owner read/query counts
cache hit/miss
bytes fetched from source views
```

Operational metrics are noncanonical and may vary. They cannot affect selection or artifact identity.

## Frozen evaluation corpus

At minimum:

```text
small synthetic exact project
partial/conflicted/high-fanout/cyclic synthetic project
pinned roth-ui publication fixture
pinned Blizzard UI E3-A source publication fixture
combined user-project + Blizzard UI + ReferenceView fixture
source-boundary/adversarial corpus
```

Every corpus item pins exact generation/profile/source/evidence/checksum identities.

## Frozen task families

```text
navigate to package/file/module
inspect exact declaration/signature
trace load reason path
trace native/custom signal relation
trace hook/script structure
trace state reads/writes
explain exact API use with ReferenceView facts
collect existing finding evidence
prepare exact bounded change context without proposing an edit
continue a truncated pack on the same snapshot
```

Expected mandatory records, allowed optional records, forbidden claims, and acceptable omissions are machine-readable fixtures.

## Hard gates

- 100% mandatory root/universe/evidence/coverage/boundary record recall;
- zero cross-universe/generation substitutions;
- zero authority/confidence upgrades;
- zero direct-edge fabrication from paths;
- zero hidden conflicts/partial/truncation/omissions;
- zero source-boundary escapes and private-data violations in the frozen adversarial corpus;
- exact canonical semantic bytes across deterministic rebuild modes;
- exact renderer bytes for frozen render profiles;
- exact-token claims only under the frozen tokenizer profile;
- no model/tool/parser/storage side effect.

Thresholds for optional utility, compression, latency, and memory remain null until executable implementations and corpus measurements exist.

## Utility measures

Possible nonauthority measures:

```text
mandatory recall
optional useful-record recall
irrelevant optional record rate
semantic compression ratio versus allowed full structured baseline
source-byte reduction
query/expansion count
consumer task success under a frozen consumer harness
consumer citation/origin accuracy
```

A consumer model score is evaluation evidence only. It cannot select canonical context, change confidence, or authorize implementation.

## Baselines

Compare against frozen alternatives such as:

- exact roots with no context;
- L0 only;
- L0 + root L1;
- profile-selected semantic pack;
- bounded raw source excerpt baseline;
- bounded graph neighborhood baseline.

Do not compare against an unbounded repository dump or a moving external model/provider without exact harness/model/profile pins.

## Rename and universality mutations

Mutate repository, package, directory, file, and irrelevant local identifier names while preserving normalized structure. Universal selection, grouping, and context facets must remain unchanged except exact source/entity identities that legitimately changed.

Named addon/framework/product heuristics fail the gate.

## Evaluation reports

```text
ContextEvaluationReport
    exact corpus/task/harness/consumer/profile IDs
    semantic pack and renderer IDs
    expected/observed mandatory records
    optional/forbidden records
    coverage/conflict/omission findings
    security/source-boundary findings
    byte/token/operational metrics
    pass/fail/NotEvaluated per gate
    canonical digest excluding incidental timings where declared
```

Missing consumer/model/tokenizer/probe implementation yields `NotEvaluated`, never pass.

## Reproducibility

Record exact framework commit, contract/profile/checksum bundles, input publications, tokenizer/renderer/harness versions, worker modes, and platform adapter. Retain raw result manifests sufficient for independent comparison.

## Completion

E3-B implementation cannot be declared complete until hard gates execute against all mandatory corpora. Documentation-only fixtures and null thresholds are blocking, not successful evaluation.
