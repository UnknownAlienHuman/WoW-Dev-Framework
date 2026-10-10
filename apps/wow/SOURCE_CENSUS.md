# Explicit local source census

```text
wow source census --config <source-census.json> [--format json|text]
```

This read-only command calls `wow-service::source_census::census_local_source`
once. The configuration parent is the explicit source snapshot root. The project
disk owner reads only manifest-declared members through the retained directory
handle, checks lengths/digests and refuses symlinks or changed content. No source
download, directory scan, include expansion, Lua execution or store write occurs.

The strict configuration has four fields:

| Field | Contract |
|---|---|
| `schema` | `wow-service/source-census-input/1` |
| `profile` | `BlizzardUiSourceProfileRequest` from [source admission](../../docs/PLATFORM_SOURCE_ADMISSION.md) |
| `inventory` | `wow-project/platform-source-inventory/1`, bound to that exact profile digest |
| `selection` | Explicit `package_root` and optional `load_context` using the existing TOC context contract |

Paths remain snapshot-relative. The selected reference profile supplies the
exact flavor/Interface identity; the operation does not infer them from TOC names
or a moving branch. Caller Git/version, materializer and root-completeness claims
remain assertions. A fixture cannot become vendor-source evidence.

The result retains source/profile/manifest/admission identities, target, origin,
root assertions, omissions and per-document outcomes. Counts include every
declared file by kind/disposition, immediate package child directories observed
in manifest entries, their selected/included/declared TOCs, and selected-TOC
lexical file records. Direct files beneath `package_root` and entries outside it
are counted separately. Empty directories absent from the manifest are unknown.

Every included XML and XSD document is parsed once using the existing XML owner.
Selected TOCs use the existing TOC parser; other TOCs receive `unselected_toc`.
XML and schema counts are separate. Elements, attributes, script source kinds,
inline units/bytes and mapping segments are exact observations of successfully
parsed unique documents, independent of how often a load path might reach them.
The XSD count is lexical XML measurement, not schema-component admission.

Processing retains one syntax index at a time. The inherited parser bounds remain
in force, including 1 MiB per document and 32,768 lexical records. Invalid UTF-8,
oversize documents, parser syntax/budget refusal and missing selected TOCs remain
explicit; refused documents do not contribute invented syntax counts. Integer
counts use checked arithmetic. Configuration is bounded by the existing 32 MiB
disk port; result/index encoding is bounded by the 64 MiB native-artifact ceiling.

`index_json_bytes` measures serialized XML indexes through a counting sink,
without allocating their JSON. Skipped decoded attribute values and inline text
have separate byte counts. `max_measured_document_*` records the largest successful
document observation. These numbers are not allocator or peak-RSS estimates.

JSON output is the exact service result plus LF. Text prefixes the same receipt
with a scope label. Exit codes: `0` measures all declared included members with
no declared omissions/refusals; `2` preserves partial inventory, omissions,
selection issues or refusals; `4` is acquisition/identity/budget/output failure;
`64` is invalid transport/configuration input; `130` is cancellation. Root
completeness is unverified even at exit `0`.

Expanded loads, dependency closure/SCCs, source handles/evidence, graph nodes/edges
and graph metadata, peak memory and full-corpus capacity are `NotEvaluated`, not
zero. This operation is the measurement prerequisite in Issue #113; it does not
close the measured-profile or W17 application/candidate/Skeleton acceptance gates.
