# `wow-emmy` implementation contract

**Status:** E0-active contract scaffold; no Rust code yet.

## Mission

`wow-emmy` isolates the pinned upstream EmmyLua analyzer behind one adapter, owns the single-writer analyzer actor, exposes normalized syntax/semantic facts and generic diagnostics, and prevents analyzer version/configuration behavior from leaking through the rest of the framework.

## Owned responsibilities

- upstream dependency pin and compatibility identity;
- analyzer configuration and WoW library/workspace assembly inputs;
- single-writer analyzer session/actor lifecycle;
- VFS add/update/remove and index lifecycle;
- immutable analyzer snapshot identity for a higher-level project generation;
- source URI/path/span normalization;
- built-in diagnostic normalization;
- normalized syntax/semantic facts required by recognizers and rules;
- diagnostic provider interface/registry contract, without implementing WoW rules;
- analyzer compatibility probes and last-known-good selection;
- bounded library/workspace behavior and cancellation.

## Explicit non-responsibilities

`wow-emmy` does not:

- fork the upstream analyzer by default;
- own APIDocumentation/reference truth;
- parse TOC or XML;
- implement WoW-specific diagnostic algorithms;
- mutate user/editor settings;
- load the full Blizzard UI implementation into the normal library workspace;
- own graph persistence, search, context rendering, or transport;
- infer runtime Secret state;
- expose upstream internal types as public framework contracts without an adapter.

## Analyzer actor model

One actor/session owns one mutable upstream analysis instance. Writes are serialized. Readers receive an immutable analyzer snapshot token/fact view tied to:

```text
upstream version/commit
configuration digest
library roots and digests
workspace roots and file digests
index epoch
profile/reference identity supplied by the caller
```

The broader `ProjectGeneration` is owned/coordinated with `wow-project`/`wow-service`; `wow-emmy` must provide enough identity to prove that its facts belong to that generation.

## Workspace contract

```text
main workspace
    first-party addon Lua files only

library workspace
    generated WoW annotations
    project-declared libraries
    narrow explicit Blizzard stubs when required

excluded by default
    full Blizzard UI source
    unrelated installed addons
    arbitrary generated bodies
    user editor configuration
```

All roots are explicit and normalized. Repository content cannot add analyzer libraries or commands implicitly.

## Required operations

| Operation | Required behavior |
|---|---|
| `create_analyzer_session` | Construct one isolated analyzer actor from explicit config/library/workspace inputs. |
| `load_library_snapshot` | Load generated annotations/libraries with digests and report parse/coverage failures. |
| `set_workspace_files` | Establish the first-party file set without filesystem-wide discovery. |
| `update_file` | Apply one content/digest update and return affected analyzer partitions. |
| `remove_file` | Remove one known file and invalidate dependent analyzer facts explicitly. |
| `build_or_update_index` | Produce a new analyzer snapshot token; cancellation cannot publish partial state. |
| `diagnose_file` | Run upstream built-in diagnostics and normalize IDs/severity/spans/messages. |
| `extract_syntax_facts` | Return normalized correctness-path syntax facts needed by higher crates. |
| `extract_semantic_facts` | Return symbols/types/references/calls/expressions through stable framework records. |
| `resolve_analyzer_span` | Convert upstream coordinates to a stable project source handle input. |
| `register_provider_descriptor` | Register capability/rule metadata and an invocation seam without depending on `wow-rules`. |
| `run_compatibility_probe` | Measure config keys, diagnostics, inference, updates, spans, determinism, and performance. |
| `select_last_known_good` | Reject activation when a mandatory compatibility capability is lost. |
| `shutdown_session` | Cancel work and release resources deterministically. |

## Normalized fact boundary

Higher crates must not pattern-match upstream AST/CST internals directly. `wow-emmy` publishes only facts that have stable semantics and source coordinates, for example:

```text
file/chunk identity
function/method declarations
parameters/returns
local/global/table/member declarations
call expressions and receiver shape
literal table/field accesses
branches/loops/returns
assignment and data-flow-local expression facts
comments/annotations as untrusted source records
built-in type/symbol resolution status
```

When upstream cannot provide a required fact, expose a capability gap. Do not synthesize it with a second parser.

## Generic diagnostic normalization

Normalization preserves:

- upstream diagnostic identity/version;
- original severity and framework policy severity separately;
- primary and related spans;
- structured message arguments where available;
- analyzer snapshot/generation identity;
- suppression/configuration provenance;
- whether the diagnostic family is classified or shadow-only.

Do not deduplicate by message text.

## E0 deliverable

Implement only:

- one pinned upstream analyzer revision;
- one explicit fixture workspace;
- one generated annotation fixture as library input;
- file update/index lifecycle;
- one generic diagnostic family normalized into `wow-core` findings;
- syntax/semantic facts required by `wow.api.exists` and one local Secret rule;
- deterministic repeated runs;
- compatibility report for the exact used surface.

E0 excludes LSP, multi-root discovery, broad incrementality optimization, full compatibility matrix, and dynamic provider loading.

## Invariants

1. One writer owns analyzer mutation.
2. No result after cancellation is published as a new snapshot.
3. No editor configuration mutation.
4. No second correctness-path Lua parser.
5. No full Blizzard source library by default.
6. Upstream version/configuration is part of result identity.
7. Parse failure is partitioned; unaffected files remain identifiable.
8. New upstream diagnostics default to shadow until classified.
9. Source spans are validated against the exact file digest.
10. Upstream update activation is reversible.

## Required tests

- session create/update/remove/shutdown;
- valid annotation fixture resolves a known API;
- generic diagnostic normalization and stable ID/span;
- malformed library/input reports partial capability;
- randomized update ordering produces the same canonical snapshot result;
- cancellation does not publish partial index;
- source span/digest mismatch rejection;
- user editor config remains untouched;
- full Blizzard tree is not included by default;
- compatibility probe detects changed diagnostic/config/span behavior;
- last-known-good fallback.

## Documentation sources

- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/ARCHITECTURE.md`](../../docs/ARCHITECTURE.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/RESEARCH_BASELINE.md`](../../docs/RESEARCH_BASELINE.md)

## Definition of done

The E0 adapter is complete when a caller can reproduce one analyzer snapshot, obtain normalized generic diagnostics and required semantic facts, prove exact source/generation identity, and upgrade/rollback the upstream dependency through an executable compatibility probe rather than undocumented assumptions.
