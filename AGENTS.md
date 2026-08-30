# AGENTS.md — WoW Dev Framework

These instructions apply to all automated and human contributors in this repository.

## Repository state

- Product: Rust-first WoW code intelligence, diagnostics, graph, search, and agent tooling.
- Phase: architecture/bootstrap; implementation has not started.
- Active engineering target: E0, the executable vertical slice in `docs/ROADMAP.md`.
- Release model: exact, immutable Reference Packs and deterministic project generations.
- License: MIT.

Do not respond to implementation uncertainty by redesigning the whole platform. Preserve the accepted architecture and reduce the next change to a testable vertical responsibility.

## Required reading

Before editing, read in this order:

1. `README.md`
2. `docs/README.md`
3. `docs/ARCHITECTURE.md`
4. `docs/PROVENANCE_AND_COVERAGE.md`
5. `docs/DECISIONS.md`
6. `docs/ROADMAP.md`
7. the task-specific document

For live WoW API, security, patch, and field-note research, consult the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb). Do not copy its living notes into this repository. Promote a conclusion only as a contract, ADR, schema, fixture, test, or pinned release input.

## Authority order

1. Accepted contracts and ADRs in this repository.
2. The exact Reference Pack manifest and pinned Blizzard source snapshot selected by the task.
3. Generated Blizzard API documentation and Blizzard UI implementation/XML/TOC from that snapshot.
4. Project-owned fixtures and tests.
5. Runtime probe evidence tied to an exact client build and scenario.
6. The external WoW engineering knowledge base.
7. Selected third-party implementations at pinned commits.
8. General community reports or model inference.

A lower source may reveal a gap; it may not silently override a higher source. Record conflicts instead of reconciling them by guesswork.

## Architectural invariants

Every contribution must preserve these rules unless an explicit accepted ADR replaces one:

- No default EmmyLua fork. Pin upstream behind one adapter.
- No second correctness-path Lua parser.
- No hidden editor-setting mutation.
- No execution of arbitrary Lua during reference ingestion.
- No direct writes into Codebase Memory storage.
- No production branches on repository or addon names.
- No mixed-profile diagnostics.
- No inferred relation without provenance, generation, coverage, and confidence.
- No clean negative answer from a stale, partial, or failed partition.
- No fuzzy or semantic candidate presented as a proven replacement.
- No full Blizzard UI tree placed into the normal Emmy library workspace or default agent context.
- No database server, vector database, or graph service in v1 without measured necessity.
- No component promoted to the default path without unique correctness responsibility or measured task benefit.

## Work sequence

1. State the exact task, affected contract, and selected profile/generation.
2. Inspect existing files and search for the nearest contract or decision.
3. Identify whether the change is normative, operational, research, or experimental.
4. Make the smallest coherent change that can be tested.
5. Add or update fixtures before claiming correctness.
6. Run the relevant deterministic checks fresh.
7. Report exact commands and `pass`, `fail`, or `skipped`.
8. Update the documentation index, roadmap, decision register, or schemas when routing changes.

Missing tooling is `skipped`, never `pass`. Never claim an in-client runtime result that was not actually observed.

## Evidence discipline

Every externally visible result must be capable of carrying:

```text
profile_id
reference_generation
project_generation
source_handle or source artifact
extractor/recognizer/rule ID and version
evidence provenance
confidence
coverage partition and status
competing evidence or known gap
```

Use the vocabulary in `docs/PROVENANCE_AND_COVERAGE.md`. A result can be source-confirmed yet still have incomplete surrounding coverage; these are separate axes.

## Implementation rules

### Rust boundaries

- Prefer narrow libraries with explicit data contracts.
- Create a crate only when the responsibility is independently testable and reusable.
- Keep service/use-case logic independent of CLI, MCP, LSP, and editor frontends.
- Keep mutable project state inside one actor/owner; publish immutable generations to readers.
- Keep storage migrations explicit, reversible where practical, and covered by round-trip tests.
- Avoid `unsafe` unless there is a documented invariant, focused test coverage, and a clear performance or FFI requirement.

### Parsers and ingestion

- Use Emmy syntax/semantic facts for Lua correctness-path recognizers.
- Parse XML structurally and with bounded resource use.
- Parse TOC order and variants explicitly.
- The APIDocumentation evaluator is declarative and allow-listed; arbitrary Lua is rejected or quarantined.
- Preserve unknown upstream fields verbatim and mark affected capabilities as incomplete.
- Treat external repositories as untrusted input. Do not execute build scripts, Lua, hooks, installers, or repository-local tools merely to index source.

### Diagnostics

- Separate root causes from downstream symptoms.
- A rule declares required capabilities and returns `NotEvaluated` when they are unavailable.
- Autofixes require exact, mechanically checkable preconditions. Otherwise return a plan or candidate.
- New rule families begin in shadow/evaluation mode until their false-blocking rate is measured.

### Graph and search

- Preserve distinct ownership, load, object, inheritance, registration, lifecycle, state, and call axes.
- Emit universal roles from declarative recognizers.
- Exact, alias, deprecation, replacement, and lineage lanes precede text and semantic lanes.
- Search explanations must expose why a result ranked and which lanes were used.

### Secret Values and restrictions

- Preserve raw restriction metadata independently of annotation projections.
- Unknown restriction facets make dependent checks `NotEvaluated`.
- Static Secret analysis starts with direct local operations and guard dominance.
- Runtime spell secrecy or other data-driven state is never frozen into a permanent source-code whitelist.

## Documentation rules

- English is the canonical repository language.
- The v8.0 archive path is a retired-source tombstone; use Git history for architecture archaeology.
- `docs/ARCHITECTURE.md`, `docs/PROVENANCE_AND_COVERAGE.md`, public schemas, and accepted ADRs are normative.
- `docs/IDEAS.md` is explicitly non-normative.
- Avoid duplicated truth. Link to the owning document.
- Use concrete terms from `docs/GLOSSARY.md`; do not invent synonyms for established contracts.

## External repositories

- Pin repository, commit, path, and license before using code as evidence.
- Community code is implementation evidence, never Blizzard API authority.
- Do not vendor third-party source into this repository unless an explicit decision and license review require it.
- Store only manifests, checksums, fixtures permitted by license, and stable source handles.

## GitHub and automation

### Connector capability verification

Before claiming that GitHub is read-only or that a write cannot be performed:

1. reload the complete GitHub tool catalog without a query filter;
2. call `get_repo` and verify `permissions.push == true` for the target repository;
3. if write capability is still uncertain, call `create_blob` as a harmless probe and do not attach that blob to a tree or commit;
4. use GitHub API write actions even when the local VM has no network route, Git credentials, or authenticated `gh` session.

Never infer connector capabilities from a filtered or partially loaded tool list. Local Git failure is not evidence that connector writes are unavailable.

- Do not add decorative CI. A workflow must execute a real local command, have a defined owner, and correspond to a roadmap gate.
- Do not enable scheduled jobs, release automation, CodeQL, Dependabot, or publishing merely by convention during bootstrap.
- Pull requests must list affected contracts, validation commands, profile assumptions, and unresolved coverage gaps.
- Keep commits coherent. Prefer one contract or independently reviewable implementation slice per commit.

## Completion report

A completion report must state:

```text
changed files
contract or decision affected
selected profile/build, when relevant
commands run and result
fixtures or evidence added
known gaps and NotEvaluated capabilities
follow-up issue only when genuinely outside the accepted scope
```
