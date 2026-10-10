# Native recognizer assertion adapters

Additive `wow-recognizers` APIs compose retained TOC/XML/script/state facts over
exact Producer addresses from the held native direct graph. This checkpoint
adds native caller composition; configuration, service publication, replay and
CLI routes retain their existing layouts. The focused native scenario and all
workspace gates/build pass on 2026-10-10. Full W17/E3 acceptance remains open.

## APIs and native inputs

All entry points take cancellation. TOC, XML and state-core also take their
existing family selector. The old entry points and matcher families remain.

| Module / entry point | Input and recognition boundary |
| --- | --- |
| `source_state::recognize_source_state_assertions` | `SourceStateAssertionInput`: actual function-call report and facts with exact root/caller/target references. Separate `SourceStateAssertionRecognition`, profile `wow-recognizers/source-saved-variable-assertions/1`. |
| `source_scripts::recognize_source_script_assertions` | `SourceScriptAssertionInput`: script facts with exact receiver/handler references. Separate `SourceScriptAssertionRecognition`, profile `wow-recognizers/source-xml-script-assertions/1`. |
| `source_toc::recognize_source_toc_assertions` | `SourceTocAssertionInput`: retained TOC facts and qualified file-path to exact Entity reference map. Existing `SourceTocProposals` / `wow-recognizers/toc-structural/1`. |
| `source_xml::recognize_source_xml_assertions` | `SourceXmlAssertionInput`: retained XML facts/script bindings, qualified file references and original XML/Analyzer proposal ID to exact Entity reference map. Existing `SourceXmlProposals` / `wow-recognizers/xml-structural/1`. |
| `source_state_core::recognize_source_state_core_assertions` | `SourceStateCoreAssertionInput`: admitted access predecessor and actual `SourceStateAssertionRecognition`, for Read or Write. Existing `SourceStateCoreProposals` / `wow-recognizers/state-structural/2`. |

Finish the [native direct plan](PLATFORM_DIRECT_PRODUCERS.md) against its exact
four-stage graph plus selected raw prelude. Retain that graph and its borrowed
`PlatformGraphProvenance`; extend a separate native graph for recognizers.
Direct `finish` deliberately rejects the recognizer-extended partition set.
Obtain addresses with `direct.assertion(&GraphLocalAssertion)` using original
proposal IDs. Supplied references must be Producer addresses. Local references,
wrong scope/batch/proposal or missing accepted receipts refuse. Native lookup
returns input-generation nodes; final materialized IDs are not inputs. Failed
lookup never selects a legacy source partition as fallback.

The native caller runs state access, then `SourceTocFamily::ALL` plus explicit
`SavedVariableRoot`, scripts, `SourceXmlFamily::ALL`, and core Read/Write.
Admit each dependent batch before continuing. SavedVariableRoot is excluded
from TOC ALL. TOC's supplied file map rejects paths absent from retained File
facts; XML separately takes its required file/entity maps. Qualified paths and
original XML occurrence IDs remain intact.

Scripts retain addresses in `recognition.endpoints()` and scope in
`recognition.scope()`; their batches do not gain GraphAssertionRecords. The
named script resolves an XmlStructure receiver and AnalyzerStructure Lua
handler. State access/core retain actual graph derivation records; core checks
the admitted access receipt and exact root/caller/target prerequisites.
Typed support, confidence, context, ambiguity and omissions remain intact.
Serialized metadata cannot reconstruct the private state/script recognition
envelopes or admit a graph owner. Bounded metadata counting and inherited
limits apply per call; whole-corpus and exhaustion acceptance are separate.

## Scoped verification and remaining work

The single [native caller test](../crates/wow-service/src/live_project/tests/namespaces/native_assertions.rs)
`native_assertion_chain_uses_exact_direct_predecessors` admits the namespace
fixture with SavedVariables declarations and the existing state-core Lua
donor through the real platform owner. Disk is absent before projection.
Raw plus four stages finish before native recognizer composition, without a
monolithic overlay or service publication.

It passes with 12 access receipts, one script assignment, 17 XML results,
six core reads and six writes. Checks cover exact state/script support, native
endpoint producers/input membership, core prerequisites, nonempty root/core
outputs, no rejections and nonauthoritative coverage. Substituting a different
genuine Entity for the state root returns `AdapterFactMismatch` without changing
the predecessor; pre-set cancellation returns `Cancelled`. This scenario does
not establish every optional family shape or cancellation during execution.

Final local workspace gates/build completed **2026-10-10 09:19:03 UTC**:
`cargo xtask check`, `cargo fmt --all --check`, workspace all-target/all-feature
check, strict Clippy, tests (**941 passed, 0 failed, 1 ignored, 112 targets**),
strict rustdoc under `RUSTDOCFLAGS=-D warnings`, and workspace
all-target/all-feature build all passed. Publication and CI are not asserted.

Configuration/service/replay selection of the split and other recognizer
routes remain open. Existing source `/19`-`/21`, ordinary replay v1-v4,
platform replay v5-v8 and storage/catalog/epoch channels remain unchanged.
Missing direct roles, fingerprints, bounded `SkeletonInputView`, source
transport, native fault/exhaustion and real Gethe/Ketho corpus/performance/WoW
runtime acceptance remain open. Partial coverage supplies no clean absence,
runtime, origin or license authority. Full W17/E3 is not accepted; Gethe
materialization stays deferred and W18-W26 are unchanged.
