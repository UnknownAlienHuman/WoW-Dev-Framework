# Captured Inventory spans in the native direct graph

The explicit library entry
`wow_project::graph::build_platform_graph_proposal_plan_with_inventory_spans`
adds exact captured source-span entities and file-to-span containment. It requires
one genuine retained platform ProjectView and uses the existing collector,
source handles and evidence. No parser, analyzer or source acquisition is added.

The entry selects native `wow-project/platform-direct-producers/2`, direct stage
producer version `2`, and registry `wow-project.source-load` version `17` before
input generation, foundation and raw prelude are constructed. Follow the same
ordered native admission/finish protocol as
[the direct producer plan](PLATFORM_DIRECT_PRODUCERS.md). Old accepted batches are
not valid in the new registry scope.

| Role | Registry and owner | Exact support |
| --- | --- | --- |
| Captured span | `source_span`, Inventory, Proven | Semantic key `source_handle` is the original canonical handle ID; original handle and EvidenceRecord are retained. |
| File containment | `source_file_contains_span`, Contains, Inventory, Proven | Inventory-local file and span proposal addresses, with the same precise span handle and evidence. |

Each retained handle is reconstructed through the held ProjectView and compared
in full, including origin/revision, path/digest, generations, optional entity key
and span. ByteRange retains native half-open byte coordinates and file bounds;
WholeFile remains WholeFile. Unknown handles emit no extent or containment row.
`inventory_span_omissions()` on the plan and finished capability returns
`Some(count)` for native /2, including Some(0), and None for native /1. The source
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

The original native /1 entry and stage serialization remain unchanged. Source
/22, selected service request12/result19 and native replay/storage v9 still use
native /1. Native /2 adds no configuration, service, CLI or replay selector. The
existing registry-16 raw excerpt binder does not admit registry17; new raw read
binding is outside this slice.

The existing disk-absent
[direct producer lifecycle](../crates/wow-project/tests/platform_direct_producers.rs)
now exercises native /2 over the same held source. It verifies complete known-span
and containment membership, WholeFile/ByteRange separation, exact source/evidence
and catalog closure, accepted Producer addresses, Partial Contains coverage,
wrong-version/foreign-registry refusal and unchanged native /1 owner plus replay8.
It adds no fixture matrix. Unknown-span injection, empty-range and boundary fault
fixtures, real Gethe/Ketho corpus, performance, WoW runtime and full W17/E3
acceptance remain NotEvaluated. Missing Inventory/project/raw-role joins, TOC/XML
roles, fingerprints, SkeletonInputView and source transport remain open.

Final local workspace policy, fmt, all-target/all-feature check, strict Clippy,
tests, strict rustdoc and all-target/all-feature build pass, completed
2026-10-10 10:31:01 UTC: 944 passed, 0 failed, 1 ignored, 112 targets. These checks
qualify this native library slice; remote CI and full package acceptance are
separate.
