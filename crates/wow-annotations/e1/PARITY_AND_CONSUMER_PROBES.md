# Semantic parity and consumer compatibility probes

**Status:** normative E1-C Ketho differential-oracle, EmmyLua/LuaLS consumer, baseline, comparison, classification, and promotion contract.

Matching one oracle file tree does not prove correctness. E1-C validates three distinct questions:

```text
Reference fidelity
    Did our semantic model project the exact selected ReferenceView facts?

Oracle parity
    How does our semantic output compare with a pinned Ketho-style artifact from equivalent inputs?

Consumer compatibility
    Do exact pinned EmmyLua/LuaLS consumers load and interpret the artifact as required without configuration mutation or diagnostic suppression?
```

## 1. Pinned oracle input

```text
OracleBaselineRequest
    oracle kind/name
    exact repository/revision/version/toolchain
    exact source snapshot/profile/content digest
    oracle configuration/profile
    generated artifact/file manifest and checksums
    semantic extraction/comparison profile
    licenses/provenance
    budgets
```

Do not compare floating main/latest or a different Blizzard source/profile and call differences renderer defects.

## 2. Ketho role

Ketho is used for:

```text
annotation layout familiarity
known LuaCATS declaration/type conventions
selected enum/event/CVar/widget/script-object projections
curated correction behavior as historical comparison where inputs align
consumer/editor expectation fixtures
```

Ketho is not used for:

```text
platform authority over ReferenceView
editor setting mutation policy
diagnostic suppression
automatic global/library configuration
runtime dependency
blind correction import
full byte-equality requirement
```

## 3. Oracle semantic extraction

A separate pinned test/tool adapter converts oracle output into `OracleSemanticModel` using a reviewed extractor/profile.

```text
OracleSemanticModel
    baseline ID
    exact oracle/source/profile/config identity
    logical modules/declarations/members/types/docs/modifiers
    file/layout metadata
    extraction coverage/loss/conflicts
    canonical semantic/file digests
```

Extractor limitations remain explicit. Do not use our production semantic model parser to assume oracle truth without independent fixtures.

## 4. Comparison domains

### Artifact-level

```text
selected profile/source equivalence
consumer/format profile
module/declaration kind coverage
file/path/layout manifest
```

### Declaration-level

```text
kind/name/namespace/owner/receiver
availability/deprecation/modifiers
declaration cardinality
```

### Signature/member-level

```text
parameter/return/field/payload/value order
name/type/optional/nilability/variadic/default
```

### Type-level

```text
primitive/named/array/map/tuple/union/function/alias/enum
literal values
generic/recursive references
Secret/restriction analysis projection
```

### Documentation/layout-level

Secondary unless declared mandatory. Sanitization/source-map/projection-loss differences remain distinct.

## 5. Parity classifications

### `Equal`

Canonical semantic fields equal under comparison profile.

### `SemanticallyEquivalent`

Different syntax/file/layout representation with equivalent declared consumer semantics.

### `ExpectedProjectionDifference`

Intentional reviewed profile/consumer/security/loss difference.

### `OurDefect`

ReferenceView and consumer profile support exact semantics but our semantic/lowering/rendering output is wrong/missing.

### `OracleDefectOrStale`

Oracle output contradicts exact selected source/reference, uses stale input, loses known metadata, or has known extractor defect.

### `InputMismatch`

Source/profile/config/correction/consumer inputs differ; no direct parity conclusion.

### `ConsumerDisagreement`

Output comparison may appear equivalent, but actual consumer behavior differs.

### `Unresolved`

Evidence insufficient/conflicting; blocks applicable parity gate.

No classification chosen from popularity/prose/model guess alone.

## 6. Parity record

```text
SemanticParityRecord
    record ID
    exact subject module/declaration/member/type/file
    our semantic/file/reference IDs
    oracle baseline/semantic/file IDs
    comparison rule/profile
    field-level differences
    classification
    ReferenceView/source/consumer evidence
    projection loss/coverage links
    review owner/status/rationale
    canonical digest
```

## 7. Parity report and gates

```text
SemanticParityReport
    report ID
    artifact/reference/profile/oracle/comparison IDs
    counts by domain/classification
    unresolved/blocking records
    semantic parity capability state
    layout/byte parity state separately
    canonical digest
```

Release-ready annotation profile can tolerate only classifications allowed by its gate policy. Unresolved/OurDefect on mandatory semantics blocks.

OracleDefectOrStale requires exact evidence and does not silently delete the baseline; record persists.

## 8. Byte/layout parity

Byte equality may be useful for a narrowly frozen compatibility profile but is not the primary truth contract.

Classify separately:

```text
byte_equal
layout_equal_semantic_bytes_differ
semantic_equal_layout_different
not_comparable
```

A source documentation sanitization change may intentionally break bytes while preserving semantics and improving security.

## 9. Consumer profiles

For each consumer:

```text
ConsumerCapabilityProfile
    consumer kind/name/version/revision/features
    exact executable/library identity
    clean test configuration fixture
    supported annotation tags/type/declaration forms
    known defaults/implicit libraries/globals
    diagnostic IDs/severities/policies relevant to probes
    source span and incremental behavior assumptions
    limits/performance expectations
    forbidden setting/diagnostic mutation
    canonical digest
```

A consumer profile is empirical and versioned; documentation alone is insufficient for mandatory semantics.

## 10. Probe workspace

The external test adapter creates an isolated temporary workspace containing only:

```text
annotation artifact
small positive/negative Lua fixtures
explicit generated test config
probe manifest
```

No user/workspace editor config or installed addon libraries. No network or source repo execution. Paths/results normalized; temp path noncanonical.

## 11. Positive consumer assertions

Probe exact examples:

```text
known API resolves
parameters/returns/member types infer correctly
named structures/fields resolve
event payload/enum/CVar/widget forms behave as declared
optional/nil/multiple-return/union/callback forms behave as expected
nominal Secret analysis types are recognized without runtime methods
source location points to expected generated fragment/source map
```

Assertions inspect structured analyzer output where possible, not only absence of error prose.

## 12. Negative consumer assertions

Must remain failing/diagnostic as expected:

```text
unknown API/global
wrong parameter/member/type use
removed/unavailable profile symbol
consumer-unsupported/lossy fixture where capability says unavailable
attempt to use Secret nominal type as ordinary runtime object/number when consumer can model it
```

A probe that only checks valid symbols can pass because diagnostics were disabled.

## 13. Configuration mutation audit

Before/after hash and enumerate:

```text
user/workspace editor settings
probe explicit config
annotation artifact files
consumer global/library/diagnostic configuration
outside-root files
```

Expected:

- only isolated probe output/logs change;
- no user settings;
- no auto-add globals/libraries;
- no diagnostic suppression/weak-union change;
- no extension install/update;
- no artifact rewrite by consumer.

## 14. Diagnostic baseline

Record exact diagnostic IDs/severities/defaults from pinned consumer. A version update:

- reruns compatibility corpus;
- classifies new/lost/changed diagnostics;
- does not silently change blocking policy;
- retains last-known-good profile/artifact when mandatory behavior regresses.

## 15. Dual-consumer strategy

### Shared artifact

Allowed only when mandatory semantic probes pass for all declared consumers and no consumer-specific syntax changes meaning.

### Consumer-specific artifacts

Use distinct type/layout/consumer profile and artifact IDs when required. Semantic ReferenceView links can remain common.

Never emit ambiguous syntax hoping each consumer interprets it differently.

## 16. Probe result

```text
ConsumerProbeResult
    result ID
    consumer/profile/artifact/reference IDs
    parse/load/index status
    positive/negative assertion records
    inferred type/signature/source span records
    diagnostic baseline/diff
    config/filesystem mutation report
    performance/resource report
    errors/coverage
    bounded raw output digest/handle
    canonical digest
```

## 17. Performance/size probe

Measure with exact corpus/hardware/runtime/profile:

```text
artifact file/count/bytes/declarations
time to load/index
resident memory if available
incremental/open fixture behavior
consumer diagnostic latency
```

No performance gate from one anecdotal run. E1 establishes baseline/budgets; optimization later requires measured need.

## 18. Parity and consumer disagreement

If Ketho parity says equal but consumer probe differs:

- classify ConsumerDisagreement;
- inspect consumer versions/config/extractor semantics;
- do not patch ReferenceView facts;
- choose consumer-specific rendering/loss/profile only after exact evidence.

If consumers agree but oracle differs:

- classify based on source/reference evidence;
- OurDefect or OracleDefectOrStale/ExpectedProjectionDifference as proven.

## 19. Baseline updates

Updating oracle/consumer revision requires:

1. pin exact new identity/source/config;
2. rerun source equivalence and full semantic extraction/probe corpus;
3. classify all differences;
4. update profiles/contracts/fixtures/checksums only after review;
5. retain last-known-good baseline and rollback notes;
6. no floating automatic acceptance.

## 20. Security

- do not execute oracle/source repo build scripts automatically inside crate;
- external tool adapter uses reviewed explicit command/executable/config;
- source comments/docs cannot modify probes;
- no user config/credentials/private paths in artifacts/results;
- bound oracle/probe files/output/logs/time/memory;
- treat analyzer/oracle output as untrusted structured input and validate schemas/sizes;
- no raw source or private workspace upload/network.

## 21. Determinism

Equivalent artifact/baseline/comparison/consumer profiles yield equivalent:

```text
oracle semantic baseline IDs/digests
parity records/classifications/order/report digest
consumer assertion/diagnostic/config-mutation records
probe result semantic digest
```

Timing/performance samples are supplemental/noncanonical unless normalized report contract explicitly includes them.

## 22. Required operations

Library-domain operations:

```text
build_oracle_baseline_request
validate_oracle_semantic_baseline
compare_annotation_semantics
classify_parity_difference
build_semantic_parity_report
build_consumer_capability_profile
validate_consumer_probe_request
validate_consumer_probe_result
compare_consumer_probe_results
classify_consumer_compatibility
classify_parity_and_consumer_gate
```

External tool adapter owns process execution/workspace creation.

## 23. Required tests

- equal/semantically equivalent/expected/our defect/oracle stale/input mismatch/consumer disagreement/unresolved cases;
- same vs different source/profile/correction inputs;
- field/type/member/source-map parity;
- byte/layout parity separate;
- positive and negative consumer assertions;
- diagnostic suppression mutation detection;
- user config/library/global mutation detection;
- shared vs consumer-specific artifact gates;
- consumer version upgrade/last-known-good;
- bounded malicious oracle/probe output;
- deterministic classification/report order;
- no oracle overwrite of ReferenceView/artifact.

## 24. Hard stops

- no floating oracle/consumer;
- no oracle as platform authority;
- no match-at-any-cost artifact mutation;
- no byte parity as sole correctness;
- no consumer compatibility inferred from parse-only or green diagnostics;
- no positive-only probe;
- no user/editor config mutation or diagnostic suppression;
- no hidden process execution in library crate;
- no unresolved mandatory discrepancy released.
