# Coverage-join recovery audit — 2026-09-22

## Retained state

Recovery began at remote `852239b9d74f48c23b6a34fe81a5a50b9c21bc5a`.
The last activated owner checkpoint was `06b87cf382660cd6934acc7c437ad39eb599cdc6`.
The coverage-join patch was retained under a temporary guarded CI input; neither
failed attempt activated its owner changes. Existing source, fixtures and the
Windows/Linux publication guards survived the chat interruption.

## Two failed attempts

The first attempt stopped at test compilation, as recorded by the retained
`852239b9` correction commit. Its byte-slice correction was retained, not discarded.
The second attempt, run `35694647154`, compiled successfully and executed all 22
tests: 20 passed; ENVELOPE-JOIN-001 and ENVELOPE-JOIN-010 failed at byte equality.

The retained Linux artifact `10680096175` includes the exact compiled test source,
source checksums and assertion operands. Independently decoding those operands
shows equal JSON values in both failures. Their byte lengths were 6549/8414 and
10351/13119: actual output was compact canonical JSON, while the expected operand
was the pretty-printed fixture file including insignificant whitespace. This is
an oracle representation error, not evidence that these two cases changed data.
It does not certify the not-yet-executed workspace or baseline controls.

## Corrective approach

Follow the existing `tests/e0_examples.rs` golden-check strategy. Parse committed
fixture text directly into `serde_json::Value` and canonicalize that raw value.
Compare full actual bytes with those expected bytes through decoded-envelope
validation, finalization and canonical reordering. Derive the permutation oracle
once from the untouched fixture, before the 64 seeded shuffles.

Do not regenerate fixtures, derive expectations from actual output, deserialize
the expected side through the typed envelope, remove failing assertions, ignore
digests/counters, sort expected arrays, weaken any owner check, or skip a gate.
All 22 conformance tests and all eight baseline controls remain required.
The reviewed owner patch and its existing API/error/contract changes are retained.

## Acceptance and remaining scope

This recovery correction is admitted by its own patch hash and exact resulting
tree before formatting. Publication requires both operating systems to pass all
locked workspace gates and produce identical tested trees, while remote `main`
still equals the staging commit. Temporary inputs are removed from the accepted
tree; no force push is permitted.

A successful recovery closes only this coverage/evaluation-join slice. Full E0-A,
source provenance, producer coverage completeness, the remaining E0 fixture
chain, and the public R0 CLI are separate gates. Use the final acceptance run and
published commit for pass/fail evidence; this audit does not predeclare success.
