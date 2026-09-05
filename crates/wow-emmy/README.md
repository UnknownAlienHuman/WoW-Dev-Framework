# `wow-emmy` implementation contract

**Status:** E0-C implementation-ready contract; no Rust code yet and no upstream dependency is pinned by this documentation change.

## Mission

`wow-emmy` is the single adapter around upstream EmmyLua Rust analysis. It owns analyzer-session lifecycle, main/library workspace assembly, file updates, generic diagnostic normalization, and the smallest normalized syntax/semantic fact surface required by E0 rules.

It does not own WoW platform truth. API existence and restriction facets come from `wow-reference`; project generation and repository structure come from `wow-project`; rule algorithms come from `wow-rules`; cross-component orchestration comes from `wow-service`.

## E0-C outcome

A future implementation agent must be able to prove this seam:

```text
explicit candidate upstream commit + compatibility probe
    -> one isolated analyzer session
    -> one main fixture workspace
    -> one annotation-library fixture workspace
    -> deterministic file update/index
    -> one normalized generic Emmy diagnostic
    -> exact project source handles/spans
    -> exact resolved/unresolved API-reference facts
    -> one direct local producer/use/guard fact slice
    -> immutable analyzer snapshot bound to one supplied project generation
```

No LSP, MCP, editor extension, full Blizzard source library, or WoW diagnostic provider is implemented in E0-C.

## Owned responsibilities

- one upstream dependency pin behind one adapter;
- compatibility-probe execution and report;
- analyzer configuration derived from explicit inputs;
- analyzer-session actor/lifecycle;
- main versus library workspace separation;
- deterministic normalized file identity/content updates;
- annotation-library loading and health state;
- built-in generic diagnostic execution;
- normalization of upstream diagnostics into `wow-core` findings;
- canonical source-span conversion;
- normalized API/member/call/reference facts;
- normalized local binding, use-operation, call, and guard/control-flow facts required by E0;
- immutable analyzer snapshot/read view;
- per-file/per-capability analyzer coverage;
- incremental update invalidation and last-known-good behavior for unaffected files;
- upstream-version/source-span/diagnostic compatibility reporting.

## Explicit non-responsibilities

`wow-emmy` does not:

- decide whether a WoW API exists in a selected build;
- store Secret/restriction facets;
- run `wow.api.exists` or `wow.secret.local_operation`;
- parse TOC or XML;
- publish the repository's canonical `ProjectGenerationId`;
- build the project/load graph;
- persist project/reference databases;
- search external repositories or Codebase Memory;
- generate Reference Packs or annotations;
- mutate user/workspace editor settings;
- expose raw upstream analyzer internals as the framework contract;
- add an external diagnostic plugin system during E0;
- start an LSP/MCP server;
- execute analyzed Lua or repository tools;
- claim in-client behavior.

## Required reading

Before implementation, read:

1. [`../AGENTS.md`](../AGENTS.md)
2. [`../DEPENDENCY_GRAPH.md`](../DEPENDENCY_GRAPH.md)
3. [`../WORKSTREAMS.md`](../WORKSTREAMS.md)
4. [`../wow-core/CONSUMER_GUIDE.md`](../wow-core/CONSUMER_GUIDE.md)
5. [`AGENTS.md`](AGENTS.md)
6. [`DECISIONS.md`](DECISIONS.md)
7. [`PIN_AND_PROBE.md`](PIN_AND_PROBE.md)
8. [`SESSION_MODEL.md`](SESSION_MODEL.md)
9. [`FACT_MODEL.md`](FACT_MODEL.md)
10. [`DIAGNOSTIC_NORMALIZATION.md`](DIAGNOSTIC_NORMALIZATION.md)
11. [`SOURCE_COORDINATES.md`](SOURCE_COORDINATES.md)
12. [`ERROR_MODEL.md`](ERROR_MODEL.md)
13. [`TEST_MATRIX.md`](TEST_MATRIX.md)
14. [`IMPLEMENTATION_PLAN.md`](IMPLEMENTATION_PLAN.md)
15. [`CONTRACT.json`](CONTRACT.json)
16. Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

Normative repository sources:

- [`../../docs/EMMYLUA_AND_DIAGNOSTICS.md`](../../docs/EMMYLUA_AND_DIAGNOSTICS.md)
- [`../../docs/PROVENANCE_AND_COVERAGE.md`](../../docs/PROVENANCE_AND_COVERAGE.md)
- [`../../docs/SECURITY_MODEL.md`](../../docs/SECURITY_MODEL.md)
- [`../../docs/TEST_STRATEGY.md`](../../docs/TEST_STRATEGY.md)
- [`../../docs/RESEARCH_BASELINE.md`](../../docs/RESEARCH_BASELINE.md)

Upstream source to inspect at implementation time:

- [EmmyLuaLs/emmylua-analyzer-rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust)

The revision recorded in `RESEARCH_BASELINE.md` is historical research input, not the automatic E0-C pin.

## Dependency and pin policy

E0-C depends directly only on `wow-core` plus the smallest approved upstream/analyzer-support dependencies.

The implementation must:

1. select an exact upstream commit, not a floating branch;
2. record repository, commit, crate versions, features, Rust/MSRV requirements, and license;
3. run the compatibility probe in [`PIN_AND_PROBE.md`](PIN_AND_PROBE.md);
4. retain the previous last-known-good candidate until the new pin passes;
5. isolate upstream names/types inside the adapter;
6. update the pin only with probe fixtures and observed behavior changes.

No fork is introduced by default. An optional upstream provider API proposal cannot block E0.

## E0-C fixture workspace

The closed fixture uses four first-party Lua files and one library annotation file:

```text
main/
    clean.lua
    generic-error.lua
    missing-api.lua
    secret-local.lua

library/
    C_E0Fixture.lua
```

Conceptual cases:

- `clean.lua` resolves `C_E0Fixture.KnownApi` cleanly;
- `generic-error.lua` produces one selected built-in generic analyzer diagnostic;
- `missing-api.lua` exposes an exact unresolved member/call fact for `C_E0Fixture.RemovedApi` without deciding platform absence;
- `secret-local.lua` exposes a producer call to `C_E0Fixture.SecretText`, a local binding, a direct unsafe operation, and guarded variants;
- `C_E0Fixture.lua` declares only the E0 fixture namespace/signatures required for analyzer resolution.

The annotation fixture does not carry canonical Secret metadata. The reference facet remains owned by `wow-reference`.

## Workspace assembly

```text
main workspace
    first-party E0 Lua fixture files only

library workspace
    closed C_E0Fixture annotation file only

excluded
    full Blizzard UI implementation
    unrelated addons/libraries
    user editor configuration
    generated source bodies not required by E0
```

Workspace roots are explicit logical roots supplied by the harness. Source handles remain repository-relative and never expose local host paths.

## Session and snapshot model

One analyzer actor owns one mutable upstream analysis instance. Writes are serialized. Readers consume an immutable `AnalyzerSnapshot` that is bound to:

```text
one ProfileIdentity
one ReferenceGenerationId
one caller-supplied ProjectGenerationId
one analyzer pin/probe identity
one workspace/configuration digest
one ordered file-content digest set
one analyzer snapshot ID
one capability/coverage set
```

`wow-emmy` validates the supplied project generation but does not invent the canonical project generation. `wow-project` owns that identity when E0-D activates.

A snapshot never mixes facts or diagnostics from different file/configuration generations.

## Required analyzer operations

Concrete Rust names may change only with a matching contract update. Required E0 semantics are defined in [`SESSION_MODEL.md`](SESSION_MODEL.md):

```text
select_candidate_pin
run_compatibility_probe
build_analyzer_configuration
create_analyzer_session
add_main_workspace
add_library_workspace
update_file
remove_file
build_or_refresh_index
publish_analyzer_snapshot
validate_analyzer_snapshot
run_builtin_diagnostics
extract_reference_facts
extract_local_flow_facts
resolve_project_source_handle
report_analyzer_capabilities
close_analyzer_session
```

## Normalized fact boundary

E0-C emits facts, not rule conclusions:

```text
FileFact
ResolvedReferenceFact
CallFact
LocalBindingFact
LocalUseFact
OperationFact
GuardFact
ControlFlowRelation
GenericDiagnosticObservation
AnalyzerCoverageRecord
```

Examples:

- `C_E0Fixture.KnownApi` can be resolved as a member/call reference;
- `C_E0Fixture.RemovedApi` can be reported as unresolved/unknown to the analyzer;
- `SecretText()` can be linked to a local value and concatenation/branch/logging use;
- `canaccessvalue(value)` can be represented as a guard/control-flow fact;
- no fact states that `SecretText` is actually Secret—that join occurs in `wow-rules` using `wow-reference`.

See [`FACT_MODEL.md`](FACT_MODEL.md).

## Generic diagnostic normalization

E0-C selects one stable generic diagnostic category through the compatibility probe. Upstream diagnostic IDs/messages may change; the adapter records both:

```text
stable framework category
upstream diagnostic ID/code/version
normalized severity
exact project source span
structured message arguments
producer/version
project generation and analyzer snapshot
coverage
```

The framework category cannot hide materially different upstream behavior. A pin update must classify any changed diagnostic family before activation.

See [`DIAGNOSTIC_NORMALIZATION.md`](DIAGNOSTIC_NORMALIZATION.md).

## Source coordinates

Canonical framework coordinates use validated UTF-8 byte half-open ranges. Derived line/column values are supplementary.

The adapter must prove conversions for:

- LF and CRLF;
- ASCII and multibyte UTF-8;
- empty files and EOF spans;
- upstream zero/one-based indexing;
- UTF-16 LSP positions when a future transport requires them;
- update/reindex span stability.

No consumer receives a raw upstream range type. See [`SOURCE_COORDINATES.md`](SOURCE_COORDINATES.md).

## Coverage and failure isolation

Coverage is explicit per file/capability/producer.

Examples:

```text
emmy.session.ready
emmy.library.loaded
emmy.file.parsed:<file>
emmy.file.diagnostics:<file>
emmy.fact.references:<file>
emmy.fact.local_flow:<file>
```

Rules:

- an annotation-library failure blocks dependent resolution but not unrelated parser diagnostics;
- a parse failure in one file does not fabricate facts for that file;
- unaffected files may remain usable in the same coherent snapshot only when the actor can prove their state;
- an upstream panic/session corruption invalidates the session/snapshot rather than becoming partial success;
- missing capability produces `NotEvaluated` in higher layers.

## E0-C hard stops

- No `Cargo.toml` or Rust source in this documentation phase.
- No floating upstream dependency.
- No direct dependency on `wow-reference`, `wow-project`, or `wow-rules`.
- No external diagnostic plugin framework.
- No LSP/MCP/CLI server.
- No editor-setting mutation.
- No full Blizzard source workspace.
- No runtime Lua execution.
- No WoW API/restriction authority claims.
- No project-generation invention.
- No raw upstream type leakage as public contract.
- No fake clean result when library/parse/fact capability failed.

## Normative fixtures

The closed examples under [`examples/`](examples/README.md) define:

- workspace and file declarations;
- annotation-library declaration;
- expected generic diagnostic normalization;
- expected resolved/unresolved reference facts;
- expected local binding/use/guard facts;
- compatibility-probe cases;
- pending canonical byte-freeze manifest.

As with E0-B, actual byte digests are frozen after E0-A canonicalization is implemented and before the first `wow-emmy` Rust commit.

## Definition of done

E0-C implementation is complete only when:

```text
one exact upstream commit is pinned and licensed
compatibility probe passes and is committed
one isolated session loads main/library fixtures without editor mutation
KnownApi resolves to exact project reference/call facts
generic-error yields one normalized generic finding
RemovedApi yields unresolved analyzer facts but no platform-absence conclusion
SecretText local binding/use/guard facts match fixtures
all spans/source handles are exact and stable
all public outputs carry one supplied project generation and analyzer snapshot
partial/failed capabilities remain explicit
randomized update/input order yields deterministic canonical snapshot output
incremental update invalidates only proven affected facts
no analyzed Lua or repository code executes
all TEST_MATRIX cases pass
```

Until then, this directory remains an implementation-ready adapter contract, not a functioning analyzer integration.
