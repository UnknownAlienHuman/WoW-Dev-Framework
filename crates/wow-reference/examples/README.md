# `wow-reference` E0-B normative examples

These files define the closed fixture contract consumed by the future E0-B implementation.

## Files

- [`fixture-bundle.json`](fixture-bundle.json) — fixture profile, evaluator policy, catalog records, variants, expected coverage shape, and source/evidence identities.
- [`lookup-cases.json`](lookup-cases.json) — exact symbol and restriction-facet expected outcomes.
- [`CHECKSUMS.json`](CHECKSUMS.json) — member list and SHA-256 byte-freeze contract.

## Current state

The repository is still documentation-only: no Rust implementation or final canonical serializer exists. Therefore `CHECKSUMS.json` lists all members but leaves byte digests `null` with `freeze_required=true`.

Before the first Rust implementation commit, the implementation agent must:

1. select the canonical JSON byte rules from `wow-core` E0-A;
2. canonicalize these files without changing their semantics;
3. write actual SHA-256 digests into `CHECKSUMS.json`;
4. update fixture/model generation IDs derived from those canonical bytes;
5. run the complete validation matrix;
6. commit the freeze separately before or with the first implementation, never generate it opportunistically during tests.

A `null` digest is invalid once `crates/MANIFEST.json` changes the `wow-reference` implementation state from `not-started`.

## Review rules

- Fixture records are synthetic and project-owned.
- The profile is explicitly `fixture`, never release-grade.
- `C_E0Fixture.SecretText` is a synthetic Secret producer used only to test dataflow between crates.
- `RemovedApi` is only an absent exact query key; it has no alias or replacement.
- Complete, partial, and conflict variants are distinct generations/model identities.
- Raw unknown fields and conflicting observations remain visible.
- Reference source handles never refer to addon/project paths.
- Expected outcomes compare structured fields, not messages.

## Change protocol

A semantic fixture change must update:

- `FIXTURE_PROFILE.md`;
- `DATA_MODEL.md` or `OPERATIONS.md` when applicable;
- `CONTRACT.json`;
- affected lookup cases;
- `TEST_MATRIX.md`;
- `CHECKSUMS.json` member digests after byte freeze.

Do not update expected lookup outcomes merely because an implementation produced different results. First determine whether the implementation or the normative contract is wrong.
