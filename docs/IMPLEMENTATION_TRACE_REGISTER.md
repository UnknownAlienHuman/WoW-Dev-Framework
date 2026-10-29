# Implementation trace register

**Source review:** `8ab892f040c42a9877836be8bd9818b98b1455d8`, 2026-10-28.
[Current status](CURRENT_STATUS.md) summarizes delivery;
[the queue](REFACTOR_QUEUE.md) orders work. This register keeps bounded source
traces and actual observations; it does not repeat implementation requirements.

**traced** means source inspected; **implemented** means code exists;
**observed** means ordinary input and visible result were executed. A lower
implemented owner is not a complete public workflow. Prior code gates are in the
linked PR history; this review ran no product build or tests.

## Active traces

| ID | Chain | Code status | Observation |
| --- | --- | --- | --- |
| T01 | public `wow check` -> direct Service result | open; legacy route remains | unexecuted |
| T02 | ordinary config -> root/TOC/flavor/Library selection | traced; strict adapter open | unexecuted |
| T03 | target/context -> TOC/package admission | predicates implemented; context open | unexecuted |
| T04 | bounded disk/Library bytes -> exact snapshots | lower readers exist; direct handoff open | unexecuted |
| T05 | selected XML -> mapped virtual Lua | extraction/maps exist; direct projection open | unexecuted |
| T06 | Main/Library/virtual -> one configured Emmy owner | implemented #247 | unexecuted |
| T07 | compiled Emmy pin -> runtime identity | implemented #249 | unexecuted |
| T08 | unsaved text -> configured project diagnostics | overlay exists; shared owner open | unexecuted |
| T09 | source/config changes -> coherent diagnostics/index | traced; rebuild route open | unexecuted |
| T10 | builder output -> ordinary Library metadata/maps | traced; manifest reader open | unexecuted |
| T11 | addon chunk arguments -> mapped context | traced; injection open | unexecuted |
| T12 | upstream diagnostic -> direct result | implemented #246 | unexecuted |
| T13 | diagnostic config -> selected collection | owner implemented #247; strict admission/canonical semantics open | unexecuted |
| T14 | exact snapshots -> direct findings/limitations | implemented #248 | unexecuted |
| T15 | compiled identity vs report evidence | implemented #249 | unexecuted |
| T16 | exact snapshots/config -> owner reuse | snapshot admission #250; semantic config equivalence open | unexecuted |
| T17 | TOC metadata/file clauses -> package/unit context | traced; handoff open | unexecuted |
| T18 | selected order -> snapshots + validated context | traced; context open | unexecuted |
| T19 | channel + `LoadWith` -> reachability/trigger | traced; open | unexecuted |
| T20 | implicit bindings -> mapped callbacks | traced; open | unexecuted |
| T21 | Interface zero + unsupported `Secure` -> admission | traced; open | unexecuted |
| T22 | chunk table + retrieval/suppression metadata | traced; open | unexecuted |
| T23 | TOC metadata -> issue class/completeness | designed; draft #266, not in `main` | unexecuted |
| T24 | virtual diagnostics without Project publication identity | implemented #259 | unexecuted |
| T25 | direct result without snapshot/analysis receipts | implemented #260 | unexecuted |
| T26 | input limitation -> optional exact location | implemented #261 | unexecuted |
| T27 | virtual diagnostic -> exact XML location | #262 hides unprojected findings; projection open | unexecuted |
| T28 | Project issue -> finite direct limitation code | implemented #263 | unexecuted |
| T29 | retained reachable package issues -> limitations | implemented #264 | unexecuted |
| T30 | line/package predicates -> closure/completeness | implemented #265 | unexecuted |
| T31 | selected Main requirements -> scope completeness | traced; unconditional chunk fallback remains | unexecuted |
| T32 | ordinary target -> profile-free target/scope plan | traced; current plan profile-bound | unexecuted |
| T33 | physical plan -> standalone blockers + Main | traced; current loader opens XML/package mapper only | unexecuted |
| T34 | admitted Main/Libraries -> exact snapshots/paths/errors | traced; construction open | unexecuted |
| T35 | empty Main + deferred units -> partial without dummy | traced; route open | unexecuted |
| T36 | opened roots + admitted paths -> collision-free authority | traced; authority validation open | unexecuted |
| T37 | ordinary config -> one Service assembly -> lower check | traced; operation open | unexecuted |
| T38 | file failure -> structured operation error | traced; origin flattened | unexecuted |
| T39 | strict config/result/error streams -> deterministic exit | traced; frontend open | unexecuted |

## Source owners behind the open boundaries

| Traces | Source / reason | Canonical contract |
| --- | --- | --- |
| T01/T02/T37/T39 | [CLI dispatch](../apps/wow/src/main.rs), [arguments](../apps/wow/src/args.rs), [legacy operation](../crates/wow-service/src/local/response.rs); ordinary route absent | [Service assembly](IMPLEMENTATION.md#first-implementation-slice), [frontends](FRONTENDS.md) |
| T23/T32/T33/T35 | [TOC](../crates/wow-project/src/load/toc.rs), [metadata](../crates/wow-project/src/load/metadata.rs), [profile-bound/eager-XML loader](../crates/wow-project/src/load/mod.rs) | [Physical plan and scope](IMPLEMENTATION.md#first-implementation-slice) |
| T34/T36 | [component-wise root opener](../crates/wow-project/src/disk.rs), [Library reader](../crates/wow-project/src/disk/library.rs); direct preparation/authority handoff absent | [Safe roots and snapshots](IMPLEMENTATION.md#safe-roots-and-selected-path-authority) |
| T31/T37 | [lower checker](../crates/wow-service/src/addon_check.rs) always adds chunk limitation/Partial; [lib.rs](../crates/wow-service/src/lib.rs) publicly exports lower request/check | [same-owner requirement](EMMYLUA_UPSTREAM.md#addon-chunk-context), [Service ownership](IMPLEMENTATION.md#first-implementation-slice) |
| T13/T16 | [configuration equality](../crates/wow-emmy/src/configuration.rs) compares serialized settings; [configured owner](../crates/wow-emmy/src/configured_session.rs) already exists | [Diagnostic semantics](EMMYLUA_UPSTREAM.md#diagnostic-configuration) |
| T38 | [configured errors](../crates/wow-emmy/src/configured_session.rs) and [lower check mapping](../crates/wow-service/src/addon_check.rs) flatten Library input failure | [Structured failures](IMPLEMENTATION.md#structured-failures) |
| T08–T11/T17–T22/T27 | [load-context trace](TOC_LOAD_CONTEXT_AUDIT.md), [trigger/binding trace](TOC_TRIGGER_BINDINGS_AUDIT.md), [baseline source audit](AUDIT.md) | [Later slices](REFACTOR_QUEUE.md#1b-dependencies-xml-and-chunk-injection), [WoW semantics](WOW_LINTER.md) |

T23 remains in draft [PR #266](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/266),
not reviewed as merged code. Reuse its work rather than creating a parallel
metadata implementation.

## Observations

No ordinary addon, editor or source-search workflow is recorded as executed.
Pending [completion cases](REFACTOR_QUEUE.md#completion-cases) stay in the queue.
When an operation is exercised, update its row with the command/protocol input,
revision and result; a concise PR/run link is enough. Do not add an observation
merely because a source trace or compilation passed.

## Historical evidence

The [previous register at 8ab892f](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/8ab892f040c42a9877836be8bd9818b98b1455d8/docs/IMPLEMENTATION_TRACE_REGISTER.md)
preserves D01–D82 correction history and baseline errata. The
[documentation register](AUDIT_REGISTRY.md) links the full 329-document baseline.
Current contracts retain the relevant decisions; old correction IDs are not an
allocation protocol or an implementation prerequisite.
