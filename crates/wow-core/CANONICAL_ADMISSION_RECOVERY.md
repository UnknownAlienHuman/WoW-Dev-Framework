# Canonical-admission recovery audit — 2026-09-22

## Two guarded attempts

Run `35700832415` stopped at compilation: serde_json's concrete map implements
both SerializeMap and SerializeStruct. The retained correction qualifies
`SerializeMap::end`; no production behavior was activated by that attempt.

Run `35701099720` passed all 19 core-only tests. With the workspace's existing
`arbitrary_precision` feature, 18 passed and CANONICAL-017 failed. The retained
Linux artifact `10681917345` contains both logs and the exact tested tree.
No negative controls or full workspace gates had passed for that attempt.

## Root cause and changed verification approach

The negative list parsed lexical JSON `-0` into Value before calling the
serializer. In serde_json 1.0.151, `src/de.rs::parse_any_number` under
`arbitrary_precision` emits I64(0) for that token; Number subsequently stores
"0". Without the feature, the parser preserves floating negative zero.
This is information loss before the boundary under test, not evidence that
canonical serialization accepted a retained negative token.

Keep the case and assert both precise upstream representations. Add a direct
Number-protocol emitter so retained `-0`, signs, leading zeros, fractions,
exponents and overflow are actually presented to canonical admission. Validate
the retained numeric spelling before invoking serde_json's scalar adapter.
Do not infer a raw JSON lexical guarantee from a Serialize-to-bytes operation.
Primary source inspected: serde_json 1.0.151, `src/de.rs` lines 932–954 and
`src/number.rs` Number serialization and integer conversions.

## Invariants kept

All 19 grouped tests, both feature profiles, the five unchanged-baseline
controls and every locked Linux/Windows workspace check remain mandatory.
Golden files, stored IDs/digests, dependencies and array order are unchanged.
The correction retains the first trait-disambiguation fix and strengthens
numeric-token admission rather than allowing a malformed token through.

Publication still requires matching tested trees and an unchanged remote main.
This audit records the failed attempts, not a predeclared pass. Full E0-A/R0,
raw-byte JSON decoding limits/duplicate admission and source provenance remain
separate gates.
