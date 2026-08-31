# E1-D normative examples

These fixtures freeze the future build, validation, deterministic rebuild, pack-member, gate, recovery, and application handoff shapes.

- `build-request.json` — exact source/profile/component/layout/budget request.
- `build-result.json` — candidate/validated-local outcome and component/member/gate closure.
- `validation-cases.json` — independent nonrepairing validation and mutations.
- `rebuild-comparison.json` — logical/canonical/store/archive equivalence classes.
- `CHECKSUMS.json` — prerequisite/member/bundle freeze gate.

All implementation-dependent IDs and SHA-256 values remain null while implementation state is `not-started`. They must be frozen before the first E1-D Rust commit.
