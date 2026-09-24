# Implementation status and update policy

**Current census:** [PROJECT_COMPLETION_MATRIX.md](PROJECT_COMPLETION_MATRIX.md).
**Execution findings:** [AUDIT_2026-09-19.md](AUDIT_2026-09-19.md).
**Normative order:** [IMPLEMENTATION_HANDOFF.md](IMPLEMENTATION_HANDOFF.md), I0–I7.

This ledger describes executable scope, not completion of the E0–E7 architecture.
Do not use old bootstrap or owner README status lines as evidence that existing
Rust implementations must be written again.

## Active workspace and unaccepted scope

The root Cargo workspace has **15 members**: `wow-core`, `wow-store`,
`wow-reference`, `wow-annotations`, `wow-emmy`, `wow-project`, `wow-rules`,
`wow-service`, `wow-graph`, `wow-recognizers`, `xtask`, `wow-render-contract`,
`wow-ketho-literals`, `wow-cli` and the guest in `modules/ketho-literals`.
The separate `bridges/literal-host` workspace has dedicated CI and is deliberately
excluded from the root workspace.

Real analyzer, project-generation, diagnostic, service, persistence, graph and
recognizer slices exist. E0-B–E0-F normative checksum manifests nevertheless
retain required pending/null fields. Partial executable state is not complete
package acceptance. This correction does not manufacture missing evidence or
waive the earlier fixture-freeze policy.

`apps/wow` now implements one-shot `status/check` over explicit materialized input
through real project/analyzer/rule owners; see [local input](../apps/wow/LOCAL_INPUT.md). `apps/wow-reference-builder` has inactive
source/tests with missing service symbols, command-contract differences and
input-boundary issues; root workspace CI does not test it. Search, context,
optional external bridge and supported release owners remain planned.

Next: complete functional source/Library input materialization and supported-profile
owner routing, then the dormant builder. Do not block missing code on new test
matrices. Existing full acceptance and launch gates remain open.

## Native source and annotation boundary

Ketho/vscode-wow-api remains the annotation implementation donor; the
[port map](KETHO_RUST_PORT.md) is the route for changes. Current Gethe source,
resolved once per operation, supplies Blizzard facts. Source and generated Lua
are data and must not be executed.

Native source/model/scalar/correction/alias handling, library projection,
receiver/type/catalog/inheritance/navigation slices and bounded literal rendering
already exist. Exact native scope is in their code, focused tests and port/usage
contracts. Scoped dual-consumer tests, source-bundle validation and a Wasm swap
probe do not establish universal corpus parity, platform truth or E1 acceptance.

The development driver is `crates/wow-annotations/examples/native_library.rs`;
use its documented explicit inputs rather than inventing a public `wow` command.
Raw source observations remain unchanged by reviewed projections. Missing,
conflicting, failed, omitted or unsupported inputs remain partial/NotEvaluated,
never `any`, a clean negative, or authoritative runtime safety.

Legacy v1 API/topology readers remain compatibility boundaries. The retired
interpreter producers are not a fallback, and the annotation TOC loader is not a
full E2 TOC/XML/load/project index. Do not restore Python, embedded interpreters,
wrappers or interpreter-based project tests.

## Implemented maintenance commands

See [xtask commands and limits](../tools/xtask/README.md):

```sh
cargo xtask check
cargo xtask sync-skill --check
cargo xtask sync-skill --write
cargo xtask check-source /path/to/checkout live
cargo xtask update-source /path/to/checkout live --expected-head <observed-SHA>
cargo xtask manifest /path/to/checkout HEAD live /path/to/new-manifest.json
cargo xtask verify-manifest /path/to/manifest.json /path/to/checkout HEAD
cargo xtask verify-library /path/to/native-output --require-input-complete
```

These are internal maintenance/development operations, not a replacement product
service or public CLI. Repository checks enforce native-only assets/invocations,
JSON syntax, **unique decoded object keys**, and synchronized skill copies.
They do not yet prove full contract-schema/ID/dependency/fixture/Markdown closure.
Eight duplicate-key regression groups were added at implementation checkpoint
`c10579d359b5f0044fc6fcdfef3b353c5caa65dc` without changing dependencies.

## Source update and provenance policy

Choose an explicit flavor/moving source selector at operation start and record
one exact source revision, version and manifest. Re-resolve on a new operation.
A recorded commit/build/toolchain is evidence for a run, not permanently embedded
current truth. Offline or unavailable remote observation is `unverified-current`.

`check-source` is read-only. `update-source` is an explicitly authorized guarded
fast-forward for an existing, exclusively owned standalone checkout; see
[SOURCE_CHECKOUT_UPDATES.md](SOURCE_CHECKOUT_UPDATES.md). Expected HEAD/branch,
origin, dirty/ignored state, divergence and races are checked. Never reset/stash
operator changes, switch unexpected branches, force-push or retry an uncertain
apply. Managed cloning, GitHub-only materialization and update scheduling are
not implemented by this command.

The planned `auto`/`prompt`/`never` policy does not imply those missing acquisition
paths already exist. Optional operator-only context is advisory, disabled by
default and configured outside the repository. No private endpoint, token,
provider or corpus is a public build/runtime prerequisite.

## CI and exact evidence

Root CI exercises Linux/Windows repository policy, formatting, locked workspace
check, strict Clippy, debug/release tests and rustdoc. Other lanes exercise updated
dependencies, rolling parser consumers, isolated literal-host/guest swap/rollback
and semantic annotation consumers. The rolling parser lane deliberately excludes
semantic analyzer consumers; it is not proof that the entire analyzer adapter was
tested against a new upstream revision.

The current-source workflow resolves explicit Gethe/Ketho inputs and retains
source identities, counts, output bytes/hashes and reports. Source admission
failures are errors; explicit projection omissions remain partial, not a semantic
pass or an R0 product evaluation.

Baseline source `e2c74314bb7ccde4a9b48c049dfeacc157259960` passed
[CI 34735568141](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/34735568141).
The duplicate-key implementation is associated with
[CI 35487232004](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/actions/runs/35487232004).
Use each run's exact head and actual conclusions; later documentation commits have
their own CI. A source archive and successful CI are not a release signature,
full fixture-freeze acceptance, real-addon runtime test or supported installation.

## Updating this ledger

Work sequentially in `main`, one owned slice at a time, without task branches or
worktrees. Read owner contracts and prerequisites; execute applicable checks;
publish without force and read back exact remote HEAD/blob identities. Record
failed, skipped and unavailable checks separately. Do not claim a local cargo
run when validation came from Actions.

Update the machine inventory, current matrix and affected owner status together
when executable scope changes. An Implemented/Complete gate requires its full
reviewed fixtures and evidence, not only code presence. Historical annotation
checkpoint details remain available in Git history and their focused port/usage
documents; they must not be recycled as a current backlog without checking code.

## Verified code checkpoint

All eight jobs in CI run `35487232004` concluded success for
`c10579d359b5f0044fc6fcdfef3b353c5caa65dc`: Linux/Windows locked checks,
strict Clippy, debug/release tests and rustdoc; updated dependencies; rolling
parser; Linux/Windows semantic consumers; Linux/Windows Wasm swap/rollback.
This is code-checkpoint evidence, not a full package or public launch certificate.

## Explicit disk inputs

The one-shot CLI also accepts `wow-service/local-project-files/1`: pinned metadata
artifacts plus explicitly listed Main/Library files below the config directory.
`wow-project::disk` owns bounded read-only acquisition; both input modes use the
same owner composition. See [FILES_INPUT.md](../apps/wow/FILES_INPUT.md).
The separate `wow-service/local-project-toc/1` mode adds selected TOC/XML external-
file expansion, a generation-bound load receipt and explicit partial blockers;
see [TOC_INPUT.md](../apps/wow/TOC_INPUT.md). There is no source scan, live-rule expansion, full E2-C or R0 acceptance.
XML now has a source-backed syntax index and extracted inline units; check parses
those units with the pinned EmmyLua grammar and returns mapped XML diagnostics.
See [XML_ANALYSIS.md](../apps/wow/XML_ANALYSIS.md). Virtual-unit semantics remain partial. Local XML parent/inheritance references
are linked as described below, without inherited receiver materialization.

Selected-TOC acquisition now preflights package-wide target filters before any
Lua/XML descendant read. Included declarations enter the v3 load receipt; excluded
and unresolved packages return distinct typed local-operation outcomes without an
analyzer snapshot. See [TOC_CONTEXT.md](../apps/wow/TOC_CONTEXT.md#package-filters-before-source-acquisition).

## XML local references

Selected-TOC input links XML parent/inheritance names across the captured closure,
retaining duplicate candidates, load order, source anchors and separate parent/
inheritance cycle components. Link issues enter ordinary XML-scoped findings.
See [XML_REFERENCES.md](../apps/wow/XML_REFERENCES.md). This is local source linking,
not XSD object validation, Lua binding, inheritance flattening or runtime acceptance.

## XML to Lua declarations

Selected-TOC checks query `mixin`, `function` and direct/inherited-mixin `method`
candidates through the existing Emmy member-call session. Inherited handler sources
also have consuming-declaration contexts; method queries use that consumer's mixins.
Source provenance and incomplete ancestry remain explicit. Exact Main/Library declaration
locations and unresolved/ambiguous results enter owner receipts and ordinary
XML-scoped findings. See [XML_BINDINGS.md](../apps/wow/XML_BINDINGS.md).
No new compiler session, synthetic source or callable/runtime proof is introduced;
receiver construction and inherited method precedence remain partial.

## Bounded graph projection

The graph owner now exposes `GraphSubgraphQuery` for exact-snapshot, multi-root
breadth-first neighborhoods with original evidence and explicit coverage and
truncation. See [SUBGRAPH_USAGE.md](../crates/wow-graph/SUBGRAPH_USAGE.md).
This completes the bounded subgraph code slice of E2-01, not its full acceptance.
Exact retained-support explanations now use `GraphExplainQuery`; see
[EXPLANATION_USAGE.md](../crates/wow-graph/EXPLANATION_USAGE.md). Registry-bound axes are described in
[AXIS_USAGE.md](../crates/wow-graph/AXIS_USAGE.md). Full conflict/derivation/evidence
resolution and coherent ProjectStore acquisition remain open.

## Retained graph CLI

`wow graph subgraph|axis|explain` now routes explicit retained partition-snapshot
and query artifacts through `wow-service` to the existing graph owner. It has
bounded JSON/file admission, exact identity guards and faithful typed status.
See [GRAPH_INPUT.md](../apps/wow/GRAPH_INPUT.md). It does not acquire ProjectStore,
produce a graph from `wow check`, or advance full E2/E3/E7 acceptance.
