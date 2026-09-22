# Bounded E0 JSON admission

The concrete byte-to-envelope boundary is:

```rust
use wow_core::{E0CheckResultEnvelope, E0DecodeLimits};

let limits = E0DecodeLimits::new(1024 * 1024, 64, 100_000, 64 * 1024)?;
let envelope = E0CheckResultEnvelope::from_json_slice(received_bytes, limits)?;
```

`E0OperationErrorEnvelope::from_json_slice` uses the same limits and raw-input
policy and verifies its own schema, structured error and canonical digest.
These operations decode only core's two existing E0 contracts. They do not add
source-document parsing, transport framing, filesystem or network access, or a
public generic JSON decoder.

## Limits

`E0DecodeLimits::new(max_input_bytes, max_nesting_depth, max_tokens,
max_string_bytes)` validates four nonzero caller-owned `usize` limits. Fields
are private; there is no default, setter or `Deserialize` implementation.

| Limit | Counts | Implementation ceiling |
|---|---|---|
| `max_input_bytes` | Entire supplied slice, including whitespace and escapes | 64 MiB |
| `max_nesting_depth` | Simultaneously open objects and arrays; root container is depth 1 | 64 |
| `max_tokens` | Opening containers, object keys and scalar values; not closing delimiters or punctuation | 1,000,000 |
| `max_string_bytes` | Raw UTF-8 bytes between quotes, including escape spelling | 1 MiB |

Limits above a ceiling or equal to zero fail with `budget_invalid`; they are not
clamped. Input that exceeds an admitted limit fails with `budget_exceeded` and a
fixed `input.bytes`, `input.nesting_depth`, `input.tokens` or `input.string_bytes`
field path. No input field, including the envelope's own Budget, can raise these
limits. The ceilings are decoding resource policy, not a schema version, an
output-budget default or a claim that every currently permitted 1 GiB output can
be imported. Large-output producers and consumers must agree on a smaller limit
or explicitly revise the bounded input policy before using this boundary.

## Admission order

1. Check raw slice length, then UTF-8 without allocating a decoded document.
2. Iteratively scan tokens and container scopes under all four limits. Decode
   each bounded string with serde_json; preserve every decoded object key until
   its object closes. Reject duplicate keys even when values match or key escape
   spellings differ. Independent objects may reuse the same key.
3. Reject raw `null` and noncanonical unsigned numeric tokens before a decoder
   can discard an optional field or normalize `-0`. Only `0` or decimal integers
   without leading zeros, within `u64`, are accepted as numeric tokens. JSON
   strings are not interpreted as numbers. Valid escapes and surrogate pairs
   are checked for every string, including values.
4. Decode the original bytes directly into the selected E0 type with serde_json,
   which remains the JSON grammar and schema authority. Trailing documents,
   missing/unknown fields and incorrect types fail. No intermediate `Value`
   projection can replace a member before duplicate admission.
5. Run the existing complete envelope validator. Schema support, context,
   reference closure, coverage, evidence, status, ordering, budgets and canonical
   digest are not bypassed or repaired.

Whitespace, CRLF and object-key order do not affect semantic identity. The input
need not already be compact canonical output. The result's `output_bytes` still
measures canonical output, not the original transport spelling. Collection order
is never silently repaired and a new digest is never generated to accept input.

## Errors and scope

Duplicate decoded keys produce `duplicate_field` at `input.object_key`. Forbidden
scalar/number forms produce `canonicalization_failure` at `input.scalar` or
`input.number`. Invalid UTF-8/JSON and typed schema decoding failures produce
`contract_violation` at `input`. Serde's error prose is discarded rather than
parsed or echoed. Semantic validator errors retain their existing narrow codes.
All decode-policy errors use `decode_result_envelope` and `after_input_change`.
They never copy raw names, values, snippets or host paths into diagnostics.

The existing `Deserialize` implementations remain structural building blocks for
internal composition and mutation tests; `serde_json::from_slice` alone does not
become this new admission boundary. The five existing E0 golden example tests use
the bounded entrypoints, including canonical round trips. Host/network readers
must also bound acquisition **before** allocating their input buffer. This pure
slice API cannot retroactively bound that allocation or recover information a
caller discarded before passing bytes. It does not certify source provenance,
producer coverage completeness, the rest of E0-A, or the R0 product.
