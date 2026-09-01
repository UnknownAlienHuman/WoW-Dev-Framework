# E4-B lineage fixtures

**State:** closed documentation shapes. Implementation-dependent IDs, paired corpora, expected records, benchmark reports and SHA-256 values remain null only while `implementation_state = not-started`.

## Files

- [`lineage-universe-set.json`](lineage-universe-set.json) — exact before/after user-project, Blizzard UI or Reference comparison binding.
- [`producer-proposals.json`](producer-proposals.json) — owner stable identity, fingerprints, structural changes, Reference transitions and Candidate-only search proposals.
- [`ambiguity-review-cases.json`](ambiguity-review-cases.json) — one-to-one/one-to-many/many-to-one/copy/split/merge ambiguity and bounded review decisions.
- [`change-records.json`](change-records.json) — typed rename/move/signature/type/restriction/ownership/load/relation/removal/introduction state transitions.
- [`migration-cases.json`](migration-cases.json) — explicit replacement, migration Candidate and validated-recipe boundaries.
- [`impact-cases.json`](impact-cases.json) — direct/transitive/Possible/conflicted/truncated static impact reason paths.
- [`CHECKSUMS.json`](CHECKSUMS.json) — prerequisite/profile/corpus/vector/member/bundle freeze gate.

## Fixture rules

- Every `case_id` exists in [`../TEST_MATRIX.md`](../TEST_MATRIX.md).
- Every `expected_error` exists in [`../ERROR_MODEL.md`](../ERROR_MODEL.md).
- Tests verify committed bytes and never rewrite fixtures automatically.
- Search/name/path/fingerprint/score inputs remain Candidate-only unless an independent owner/Reference/review contract raises the ceiling.
- Synthetic or named real-corpus identifiers never authorize production special cases.
- Null values become invalid before the first E4-B Rust commit.
