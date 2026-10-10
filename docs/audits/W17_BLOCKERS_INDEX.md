# W17 blockers index

Canonical tracker: Issue #106.

Hard blockers found by the 2026-10-10 deep audit:

- Issue #113 — exact current-source corpus exceeds reviewed XML/source-graph limits;
- Issue #114 — graph axes are not definition-aware and reject rich registries;
- Issue #115 — cumulative native `/1`–`/4` profiles are not selected by one composed application/replay path;
- Issue #116 — no complete store-independent `BlizzardUiIndexCandidate` exists before publication and `SkeletonInputView`.

Required order:

```text
#113 measured corpus/profile closure
#114 definition-aware axes and relation semantics
#115 selected capability/profile composition
XML object/region classification
stable fingerprints
#116 validated candidate and publication consumption
bounded SkeletonInputView
source service/CLI transport
```

Do not advance W18 or route around these contracts. The detailed audit is
`W17_DEEP_PROCESS_AND_PRODUCT_AUDIT_2026-10-10.md`.
