# Captured Inventory spans in the native direct graph

Select `inventory_spans_v1` through the exact
[capability configuration](PLATFORM_GRAPH_CAPABILITIES.md). The native selected
plan adds captured spans and file containment using the existing collector,
source handles and evidence over one genuine retained platform ProjectView.
The selected registry binds the exact set before input generation, foundation
and raw prelude construction. Follow the existing ordered
[admission/finish protocol](PLATFORM_DIRECT_PRODUCERS.md).
| Role | Registry and owner | Exact support |
| --- | --- | --- |
| Captured span | `source_span`, Inventory, Proven | Semantic key `source_handle` is the original canonical handle ID; original handle and EvidenceRecord are retained. |
| File containment | `source_file_contains_span`, Contains, Inventory, Proven | Inventory-local file and span proposal addresses, with the same precise span handle and evidence. |

Each retained handle is reconstructed through the held ProjectView and compared
in full, including origin/revision, path/digest, generations, optional entity key
and span. ByteRange retains native half-open byte coordinates and file bounds;
WholeFile remains WholeFile. Unknown handles emit no extent or containment row.
`inventory_span_omissions()` on the plan and finished capability returns
`Some(count)` when InventorySpansV1 is selected, including Some(0); otherwise None. The source
report retains the omitted handles and original source coverage.

Contains coverage stays Partial with negative authority false. This indexes known
captured support spans, not all physical raw members or XML lexical/source-map
roles. A span relation does not establish runtime behavior or inventory completeness.

The existing 4 MiB cumulative conservative serialized metadata allowance covers
original inputs, added borrowed indexes/drafts/omissions, stages, addresses and
catalog. Existing node/edge and combined 200,000 direct-assertion limits still
apply. This allowance is not a process resident-memory bound. Cancellation or
invalid/missing/conflicting support prevents finished capability construction.
Finish resolves every admitted proposal and returns the genuine held evidence
catalog; serializable proposal reports cannot construct that capability.

The admitted native /1 service/replay path retains its identities. The former
native /2-only recipe is consolidated into the selected set; see
[compatibility and remaining scope](PLATFORM_GRAPH_CAPABILITIES.md#compatibility-inventory).

## Historical checkpoint verification

The existing disk-absent
[direct producer lifecycle](../crates/wow-project/tests/platform_direct_producers.rs)
now exercises native /2 over the same held source. It verifies complete known-span
and containment membership, WholeFile/ByteRange separation, exact source/evidence
and catalog closure, accepted Producer addresses, Partial Contains coverage,
wrong-version/foreign-registry refusal and unchanged native /1 owner plus replay8.
It adds no fixture matrix. Unknown-span injection, empty-range and boundary fault
fixtures, real Gethe/Ketho corpus, performance, WoW runtime and full W17/E3
acceptance remain NotEvaluated. The separate
[native /3 entry](PLATFORM_STRUCTURAL_ROLES.md) now adds project/raw membership,
selected TOC and lexical XML roles with these same captured spans. XML object/region
classification, virtual-Lua source mapping, fingerprints, SkeletonInputView and
source transport remain open.

Final local workspace policy, fmt, all-target/all-feature check, strict Clippy,
tests, strict rustdoc and all-target/all-feature build pass, completed
2026-10-10 10:31:01 UTC: 944 passed, 0 failed, 1 ignored, 112 targets. These checks
qualify this native library slice; remote CI and full package acceptance are
separate.
