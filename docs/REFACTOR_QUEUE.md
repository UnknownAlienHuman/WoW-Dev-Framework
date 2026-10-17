# Refactoring and completion queue

**Start here.** The default next code task is [#193](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193).
Issues hold the current scope, claim and result; this file holds order and dependencies.
The [reviewed audit](AUDIT_2026-10-17.md) explains the defects; the
[completion matrix](PROJECT_COMPLETION_MATRIX.md) retains package/launch gates.
Preparation is [PR #192](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/pull/192);
its changes are usable from that branch until integration.

## How to take a task

Read [AGENTS.md](../AGENTS.md), the selected Issue and latest claim, then only its
named owner/caller contracts and audit section. Follow
[EXECUTION_MODEL.md](EXECUTION_MODEL.md) for one manager/worktree, finite ownership,
checks and publication. Historical read-all lists do not expand the selected task;
unique invariants of touched owners still apply.

Trace the existing path, identify what will be removed, implement one complete
replacement and exercise its actual consumer. No new crate, parser, cache,
framework, format combination or fixture factory without a concrete requirement.
Keep source/generation/coverage, boundedness, cancellation and durable readers.
A necessary byte-contract change needs an explicit reader/migration decision.

Use the smallest applicable Clippy/policy check. After the function works, use
one existing bounded reproduction of the changed behavior. Full tests/corpus/
runtime qualification belongs to the selected acceptance milestone, not the start
of every patch. Report authored Rust growth separately from tests/generated data;
do not optimize for an arbitrary deletion percentage.

## Current code tasks

Order is a default, not a global lock. Independent tasks may proceed under
non-overlapping claims; one manager keeps one worktree. Do not open a second
tracker for a responsibility already listed here.

| Order | Task and first replacement | Done / real dependency |
| --- | --- | --- |
| 1 | [#193 — one Emmy analysis](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/193): remove repeated registration and syntax/member/flow constructions. | One analysis for the ordinary cold input set. Existing AnalyzerSession still owns two; wrapping it is insufficient. |
| 2 | [#194 — Core delivery](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194): remove success-finalize/reorder size fixed points, then one Search paginator. | Bounded final output with an explicit digest/reader contract. Error finalization has no such loop. |
| 3 | [#195 — W1 recognizers](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195): replace pack encode/parse/match roundtrip with typed facts. | Same facts and witness/partition support. Full mutation acceptance stays #103. |
| 4 | [#196 — Reference ownership](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/196): move ReferencePack store-member construction from Service into Reference. | One existing-owner operation; old Service algorithm removed. |
| 5 | [#131 — retained reads](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131): stored typed facts for ordinary Project/Graph read; explicit legacy conversion. | Source-absent read without analyzer/graph rebuild. Native authority extension has its own contract; no global cache. |
| 6 | [#169 — Search storage](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169): direct Store-owned FTS/content tables replace SQLite-image → hex → JSON. | One layout, same logical query and holds/retirement; explicit retained-data conversion. |
| 7 | [#122 — Context](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122): readable REQUEST, then one shared request payload family. | Legible bounded render and unchanged semantic pack; external admission remains. Independent of storage. |
| 8 | [#197 — CLI defaults](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197): finite defaults and explicit overrides for existing Search commands. | Query works without repeated budget flags. Saved budget config is a later addition, not an existing API. |
| 9 | [#198 — LSP](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198): valid empty definition/reference/symbol results. | Actual framed request followed by another request succeeds; real errors/cancellation remain. Independent. |
| 10 | [#199 — MCP](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199): nonfatal unknown notifications and bounded initialization metadata. | Selected 2025-11-25 session remains usable; no extra capability profiles. Independent. |
| 11 | [#200 — addon configuration](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200): add the missing Service-backed TOC configuration preparation command. | Existing admitted target/Library artifacts produce reusable status/check input; no fabricated identities. |
| 12 | [#201 — local workflow](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201): real publication → Context host registration, then check/context/search quick-start. | Supported addon sequence and later-process query work. Needs selected #200/#197/#122 handoffs, not all performance refactors. |
| Alongside | [#120 — evidence checks](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/120): verify a real frozen receipt's bytes, then one mapped lifecycle pair. | Wrong bytes/state fail; hashes/Quick never auto-accept packages. No global implementation gate. |
| With touched owner | [#112 — concise docs/versions](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112): retained reader inventory and remaining historical-doc cleanup. | Each old variant has a reader or explicit disposition; no repository-wide deletion sweep. |

## Finish the planned product

The local workflow is a useful milestone, not full project completion.
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125) contains the
next concrete slice, reading route and prerequisite for each later package.
Activate one real owner/application path at a time; optional or externally blocked
work does not block unrelated local implementation.

| Scope | Canonical tracker / completion requirement |
| --- | --- |
| W11 mutation and W12–W16 persistence/graph acceptance | [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103), [#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107), [#108](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/108), [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109), [#110](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/110), [#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111). Reuse their existing vectors at the relevant milestone. |
| W17 and real corpus | [#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106), [#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113), [#116](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/116), [#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117), [#118](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118). A subset or unavailable runtime cannot become complete coverage. |
| Full Context and Search | #122 / #169 retain wider facets, mappings, continuation and fault/corpus qualification after the first refactors. |
| W21 lineage/migration/impact | #125: real evidence producers, then actual review authority and accepted relations. Similarity remains Candidate. |
| W22–W23 frontends | #125: additional owner-backed operations, broader sessions/workspaces/documents, language features and client/platform conformance; daemon separately selected. |
| W24 / E5 / E5-C | #125: real calibration/holdout/reviewer inputs, then guarded artifact publication and activation. Never auto-promote an evaluated pack. |
| E6 | #125: optional, disabled-by-default external Candidate providers; no exact/negative authority and no local-product prerequisite. |
| W25 / E7-B | #125: exact source/build/bundle identity and selected Windows release/update/rollback qualification. A task description does not authorize release effects. |
| R0 → A0 → A1 → B0 → V1 | Use the [launch gates](PROJECT_COMPLETION_MATRIX.md#launch-gates) and #125. Required owner acceptance plus a real advertised app workflow; no launch claim from compilation or #201 alone. |
| W01–W10 residuals | [#124](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/124): relevant remaining hardening joins its milestone; completed branch/W01/W02 work stays completed. |

Completed selected scopes #157/#162/#170/#173–#190 are evidence in their Issues.
Check current code before recreating a feature from an old specification.

## Existing owners and donors

Read the matching row when changing that responsibility; do not import a donor's
architecture wholesale. Use the repository's selected dependency revision.
Moving source selectors are resolved only when the task needs current semantics.

| Responsibility | Read / reuse | Do not add |
| --- | --- | --- |
| Lua analysis | [EMMYLUA_UPSTREAM.md](EMMYLUA_UPSTREAM.md), existing wow-emmy adapter and [pinned upstream analysis API](https://github.com/EmmyLuaLs/emmylua-analyzer-rust/blob/aaaca68425d9362876228649b0b8d92f07654daa/crates/emmylua_code_analysis/src/lib.rs). Registration/update and compilation/semantic-model access already exist. | A second parser, analyzer session layer or upstream upgrade merely to unify calls. |
| Annotation/type normalization | [KETHO_RUST_PORT.md](KETHO_RUST_PORT.md); [reviewed Ketho emitter](https://github.com/Ketho/vscode-wow-api/blob/d0b5b51fac4c52c493371b9b18e66ce604ea4326/luasrc/annotate/init.lua) → wow-reference/wow-annotations native owners. That SHA is review evidence, not permanent latest. | Lua/Node/VS Code runtime, second importer or automatic fixture blessing. |
| Search persistence | [Store](../crates/wow-store/README.md), current Cargo.lock/rusqlite and [SQLite FTS5](https://www.sqlite.org/fts5.html); transactions, leases and GC already exist. | Another database image inside the Store, search engine or retention framework. |
| LSP / MCP | Existing app transports plus [LSP 3.18](../crates/wow-service/e7/LSP_3_18_PROFILE.md) and [MCP 2025-11-25](../crates/wow-service/e7/MCP_2025_11_25_PROFILE.md); compare corrections with the linked official specifications. | SDK rewrites, implicit daemon coupling or a profile per feature combination. |
| Addon loading | [TOC_INPUT.md](../apps/wow/TOC_INPUT.md), existing Project load/XML/disk capture through Service. | New parser, unbounded discovery or trusted identity inferred from a path. |
