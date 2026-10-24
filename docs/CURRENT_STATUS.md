# Current implementation status

**Checked:** 2026-10-24  
**Implementation code reviewed through:** PR #250, `e40b64ffa1c16851fa657c0bd7fa0073e3cc33d9`  
**Documentation reconciliation:** PR #251, `3ecf03b15bb4597daa5bcaa90d84224b23d0fffb`  
**Documentation baseline:** PR #244, merge `0fdc34c4a8afb1af51ec47d2038734d02500af42`

This is the authoritative current-state checkpoint. It does not replace the
product specification in [ARCHITECTURE.md](ARCHITECTURE.md), the implementation
design in [IMPLEMENTATION.md](IMPLEMENTATION.md), or the baseline source audit in
[AUDIT.md](AUDIT.md). It prevents those baseline documents from being mistaken
for a live progress report after implementation changes. PR #251 changed only
documentation and active Issue instructions; runtime status remains the code
reviewed through PR #250.

## Verification vocabulary

- **baseline reviewed** — the source/document claim was checked at the commit
  named by the baseline audit;
- **implemented** — code is present in `main` and passed the stated exact-head
  repository/format/compile/Clippy gate;
- **observed** — an ordinary user command or protocol sequence was actually run
  with recorded input and result;
- **open** — the producer-to-reader path is absent, incomplete, or still uses the
  transitional platform.

Build and Clippy evidence never promote a workflow to **observed**.

## Baseline documentation audit

PR #244 completed the old-document reconciliation:

- all 329 baseline Markdown files were read;
- 287 applicable document requirements were traced to their source owners and
  readers;
- 42 policy, notice, descriptive-data, or retired-governance rows were marked
  not applicable to implementation;
- no baseline row remains `partial` or `not_checked`;
- superseded E1/E2/E3/platform specifications were removed from the active tree;
- useful behavior, donors, defects, and removal boundaries were retained in the
  current guides and registries.

These figures describe document requirements, not Rust code coverage and not
runtime qualification.

The supplied Claude review independently ran the full baseline test suite and
reported 1,085 passed, 13 failed, and 23 ignored. Its diagnosis was used as
input, but the raw run log was not attached to the repository. The source review
therefore records those counts as reported evidence and separately traces every
named failure. It rejected the blanket recommendation to delete eight tests
because several retain useful admission, identity, bounds, privacy, or lock-
lifetime behavior.

## Implemented after the documentation baseline

| Slice | Delivered behavior | Evidence | Runtime observation |
| --- | --- | --- | --- |
| T12 / PR #246 | All configured Emmy diagnostics are retained; code, exact bounded message, upstream severity, and byte range reach the adapter. The standalone path initializes Lua 5.1 STD declarations. | exact-head run #418 passed | unexecuted as an addon/editor workflow |
| T06/T13 / PR #247 | `ConfiguredAnalyzerSession` owns one explicit Main + Library + optional virtual analysis, applies caller diagnostic configuration, initializes embedded STD once, and separates diagnostics-only from requested semantic collection. | exact-head run #422 passed | unexecuted |
| T14 / PR #248 | `wow-service::addon_check::check_addon` returns the direct diagnostic/limitation result without Reference, Rules, Project publication, graph, store, or presentation-platform construction. | exact-head run #425 passed | unexecuted |
| T07/T15 / PR #249 | Compiled Emmy provenance is distinct from compatibility-report evidence; direct callers need not fabricate report hashes. Legacy report-backed admission remains explicit and fallible. | exact-head run #440 passed | unexecuted |
| T16 / PR #250 | Reuse admission for the immutable configured owner requires the exact Main snapshot identity, plus the Library set, virtual identity/generation, and equivalent diagnostic configuration. | exact-head run #445 passed | unexecuted |

The current lower-level language-check core therefore exists. The public product
path does not.

## Open path to the first usable command

The next coherent slice is:

```text
ordinary CLI/config
  -> addon root, flavor, TOC and dependency selection
  -> bounded exact Main/Library acquisition
  -> mapped XML preparation
  -> existing ConfiguredAnalyzerSession
  -> existing wow-service::addon_check
  -> text/JSON diagnostics, limitations and documented exit status
```

The following remain open and must not be described as delivered:

- normal root/TOC/flavor/Library configuration and ambiguity handling;
- a small load target independent of `ProfileIdentity` and publication receipts;
- selected dependency-closure acquisition, including reviewed `Dep` spelling;
- preservation of independent units when one included XML file is malformed;
- public `wow check` wiring, rendering, exit policy, and cancellation handoff;
- XML-only Main admission and exact virtual-diagnostic projection to XML;
- addon chunk `(addonName, privateTable)` context and verified handler signatures;
- generated-Library manifest consumption by the ordinary reader;
- corrected `StrideIndex`, `NilableContents`, `MayReturnNothing`, mixin, and
  member-environment projection;
- semantic API binding and Secret uncertainty fixes before WoW rules are enabled;
- coherent updates after TOC/XML/dependency/Library changes;
- project-aware LSP reuse, valid version gaps, stale-result rejection, and
  standard empty results;
- MCP metadata/unknown-notification behavior and reuse of the same Service owners;
- direct source-body/symbol SQLite index, later-process query, readable context,
  and real SQL interruption.

`check_addon` currently consumes exact already-acquired snapshots and reports
`Partial`: addon chunk context is unavailable, and virtual input additionally
reports missing exact XML projection and handler context. This is deliberate
truthfulness, not completion.

## Documentation reconciliation

The post-merge reread found and corrected real drift in the active documentation:

1. root and application READMEs still called the four-code/message loss a current
   defect after PR #246;
2. the Emmy guide said T06 remained open after PR #247;
3. contributor instructions still told agents to create the configured owner and
   direct Service result that already exist;
4. the implementation route and queue mixed delivered prerequisites with
   remaining CLI/acquisition work;
5. frontend guidance did not separate the delivered snapshot-to-Service result
   from transitional CLI/LSP/MCP callers;
6. the trace register cited exact-head runs #437 and #442 for PRs #249/#250;
   the actual successful runs are #440 and #445;
7. the baseline audit's environment note about Cargo is not a product finding and
   conflicts with the supplied review's separate full test run. Current status
   does not rely on that sentence;
8. active umbrella/completion/configuration/CLI/LSP/MCP/result Issues still
   assigned already delivered #246–#250 work;
9. the baseline register retained 31 local links to headings renamed during the
   current-guide rewrite. Those anchors are repaired, and repository policy now
   rejects missing local Markdown paths/anchors so Quick cannot silently pass the
   same drift again.

PR #251 corrected items 1–8 while preserving baseline audit evidence and all
runtime rows as unobserved. The follow-up Markdown guard corrects item 9 and
makes the same class of drift executable repository-policy failure.

## Readiness verdict

- **Documentation baseline reconciliation:** complete.
- **Post-merge documentation/current-Issue reconciliation:** complete, including
  the local Markdown path/anchor guard.
- **Known source-level architecture/error register:** complete for the retained
  baseline requirements; new implementation findings belong in
  [IMPLEMENTATION_TRACE_REGISTER.md](IMPLEMENTATION_TRACE_REGISTER.md).
- **First implementation owner boundaries:** defined and partially implemented.
- **First ordinary user workflow:** not yet implemented or observed.
- **Project ready for blind execution without further tracing:** no. Each next
  slice must follow its actual producer, handoff, reader, and visible result and
  update the trace register when the boundary changes.

Do not reopen the old platform merely because a baseline document mentions it.
Do not repeat T12–T16. Continue from the open path above.