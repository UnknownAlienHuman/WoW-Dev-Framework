# Original native Reference callable comparison

`wow source reference-compare --before-config before.json --after-config after.json --request compare.json [--format json|text]`

This one-shot command calls one Service operation. The operation loads each
explicit local Source Reference configuration once through the existing native
loader, checks its exact configuration/generation/view guards, retains both
immutable original projections, and invokes the Reference-owned comparison.
It performs no source acquisition, Store publication, Current selection or edits.
Inputs are the existing [Source Reference configurations](SOURCE_REFERENCE.md).
Currentness remains `unverified-current`.

Obtain each side's `configuration_content_digest`, `reference_generation` and
`reference.view.self_digest` from its exact `wow source reference` result. Supply
them in the request; omission, null, unknown fields or a stale guard fail. A
request has this shape (replace the angle-bracket guard values):

```json
{
  "schema": "wow-service/source-reference-comparison-request/1",
  "comparison_profile": "wow-reference/native-callable-comparison/1",
  "privacy": "local_metadata_only",
  "before": {
    "configuration_content_digest": "<exact before digest>",
    "reference_generation": "<exact before generation>",
    "reference_view_digest": "<exact before view digest>"
  },
  "after": {
    "configuration_content_digest": "<exact after digest>",
    "reference_generation": "<exact after generation>",
    "reference_view_digest": "<exact after view digest>"
  },
  "keys": ["function:C_Example.Read"],
  "limits": {"max_keys": 16, "max_view_bytes": 1048576, "max_steps": 1048576},
  "max_output_bytes": 1048576
}
```

The selected compatibility policy requires the same native callable projection
profile, flavor, edition, environment, profile class and schema versions. Exact
build/Interface/revision/profile differences are retained as the explicit ordered
comparison pair; chronology is never inferred. Native document extensions,
compatibility aliases and correction/type-resolution overlays are outside this
profile and are refused. Before and after may be the same exact input.

The owner validates both original signature-catalog bindings even for absent or
conflicted selections. Keys are exact, unique and canonically sorted in output.
For a found pair it compares ordered argument and return positions, original
names, unresolved source type names, inner types, tri-state nilability and raw
default/stride observations. Raw value comparison ignores source spans, preserves
table order and numeric lexemes, and performs no evaluation or type resolution.
Defaults/stride bodies and documentation are not emitted. Original record digests,
source IDs, file hashes and spans retain the evidence for those observations.

Results distinguish `selected_fields_equal`, `selected_fields_differ` and
`not_evaluated`. Equality covers only the selected supported fields. Per-side
lookup states retain `found`, `conflict`, `authoritative_exact_key_absence` and
`unknown`. Complete exact-key absence is a Reference lookup observation, never
an introduced/removed or lineage conclusion. Association remains Candidate;
accepted lineage count is zero and lineage negative authority is false.
The comparison digest uses the existing bounded Reference JSON digest writer
over the deterministic result with `comparison_digest` omitted.

Caller bounds are required. Limits are at most 4096 selected keys, 64 MiB per
validated view/comparison metadata and 32 Mi traversal/write steps; the Service
output is at most 16 MiB. Step use is cumulative across both view validations,
lookup/source scans, field/raw comparisons and result digesting. Exhaustion fails
without truncated success. JSON output is exact Service bytes plus one LF; text
adds a fixed label. Partial input or an unevaluated selected lookup exits 2;
completed selected comparison exits 0. Neither exit is full Reference acceptance.
Invalid configuration/arguments exit 64, cancellation 130 and other failures 4.

Explicit stable identity, rename/move/deprecation/replacement and correction
transitions, graph producer admission, review authorization, accepted lineage,
migration, static impact and runtime qualification remain separate W21 work in
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125).
