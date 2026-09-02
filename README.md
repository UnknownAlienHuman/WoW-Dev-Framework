# WoW Dev Framework

**Rust-first, editor-independent code intelligence and agent tooling for World of Warcraft addon development.**

> **Documentation frontier:** E7-A complete — closed service operation registry, explicit frontend sessions/workspaces, project-owned unsaved overlays, local daemon, LSP 3.18, MCP 2025-11-25, exact cancellation/reconnect/backpressure, and multi-client isolation.
>
> **Next documentation package:** E7-B — reproducible builds, artifacts, packaging, signing/attestations, installation, updates, rollback, retirement, compatibility, support and public release.
>
> **Implementation frontier:** not started. Executable work still begins with E0-A. No `Cargo.toml`, Rust source, or CI workflow exists.
>
> Patch-sensitive WoW guidance remains in the separate [WoW Addon Engineering Knowledge Base](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb).

WoW Dev Framework provides exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API contracts and UI source, diagnostics, typed graphs, search, lineage, migration evidence, static impact, Project Maps, L0/L1 skeletons, structural-recognizer calibration/publication, optional external semantic candidates, and thin service-only developer frontends.

It is not a generic RAG product, editor-settings mutator, runtime injection platform, source-edit executor, repository-specific heuristic engine, model-authority layer, provider database owner, generic MCP tool proxy, or automatic unreviewed promotion system.

## Contract stack

```text
E0  identity/evidence/results + fixture analyzer/project/rules/service/CLI slice
E1  ReferenceStore/View, annotations, Reference Pack build/validation
E2  typed graph, recognizers, TOC/XML/load indexing, ProjectStore
E3  Blizzard UI source, Project Map/L0/L1/context, context service/CLI
E4  exact-generation search, lineage/migration/static impact, service/CLI
E5-A calibration corpora, shadow packs, mutations, metrics, candidates
E5-B durable runs, independent review, sealed holdout, PromotionSubmission
E5-C immutable core artifact, signing, publication, canary, rollout,
     activation, LKG, rollback, revocation and partition closure
E6-A optional external semantic-candidate owner bridge
E6-B provider/session/result/mapping/selection/context service and CLI
E7-A closed frontend registry, sessions, daemon, LSP and MCP
E7-B next: reproducible packaging, distribution, updates and support lifecycle
```

Machine state and implementation order are in [`crates/MANIFEST.json`](crates/MANIFEST.json), [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md), and [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md).

## Core invariants

- One result/effect binds one exact coherent profile/generation set.
- User project, Blizzard UI source, Reference Pack, search candidate, calibration corpus, external candidate, runtime and history remain separate universes.
- EmmyLua is the sole correctness-path Lua parser/analyzer.
- TOC/XML/reference/repository ingestion is bounded and nonexecuting.
- Graph assertions retain producer, evidence, confidence, coverage, conflicts and generation.
- Production recognizers never branch on repository/addon/owner/path/popularity identity.
- A commit pin is not corpus admission.
- Labels, splits, expected outputs, reviewer notes, holdout data, model/search output, canary cohort identity and rollout state never become matcher semantics.
- `Unknown`, `Possible`, `Candidate`, `Partial`, `Conflict`, `Truncated`, `OutcomeUnknown`, cancelled and `NotEvaluated` never become proof, clean Negative or pass.
- Metrics, graph validity, review authorization, holdout authorization, submission, signature, publication, canary, activation, rollout, distribution and runtime correctness are independent gates.
- Response loss never proves no effect; exact reconciliation precedes retry.
- A signature proves exact bytes/key/profile binding, not semantic or runtime correctness.
- Publication creates `PublishedInactive`; activation is a separate exact-profile CAS.
- Last-known-good is explicitly designated, not inferred as previous/newest.
- Rollback creates new immutable effect/activation/closure records and never rewrites history.
- Applications/transports depend on `wow-service` only.
- No public success precedes mandatory retention, audit and resource closure.

## External candidate invariants

- `wow-cbm` depends directly only on `wow-core`.
- Every external result is `provenance=semantic_candidate`, `confidence=Candidate`, `negative_authority=unavailable`.
- Provider labels, rank, score, repetition, stable state and zero results never upgrade authority.
- Provider paths/URIs/revisions/symbols/spans remain `UnverifiedProviderLocator` until an exact project/reference owner maps them.
- Mapping proves locator-to-owner-record identity only; selection is explicit caller intent, not verification or edit authorization.
- Provider metadata remains a separate Candidate sidecar and never enters `ContextSemanticPack` truth.
- Provider failure is lane-local and never degrades exact local capabilities.
- No hidden fallback to another provider, stale cache, model, web, local search or broader query.

## E7-A frontend invariants

```text
one immutable FrontendOperationRegistry
one transport request -> one service operation
explicit session/workspace/project/profile registration
exact versioned unsaved document overlays
LSP 3.18 over stdio
MCP 2025-11-25 over stdio by default
current-user local daemon IPC
optional loopback-only MCP Streamable HTTP, disabled by default
```

- Registry negotiation can narrow but never add an operation or widen authorization/privacy.
- Missing owner implementations are not advertised.
- Workspaces are never inferred from cwd, Git, editor state, addon folders or WoW installation.
- LSP positions bind the exact overlay and negotiated UTF-16/UTF-8 encoding; stale versions require full resynchronization.
- Default MCP tools/resources are fixed and read-only. There is no generic `wow.call`, arbitrary tool/RPC/shell, prompt, sampling, elicitation, task execution or model-controlled authorization.
- MCP resources use exact `wow://` IDs, not floating current/latest or raw filesystem paths.
- Local daemon uses a current-user named pipe or Unix-domain socket; no TCP/remote listener exists by default.
- Disconnect is not cancellation. Progress is not completion. Response replay returns retained bytes and never reexecutes service.
- Final results, errors and state changes outrank progress/logs under bounded backpressure.
- Sessions isolate workspaces, overlays, authorization, private source, operations, provider access, results and response journals.
- Unsaved source is private and memory-only by default.
- E7-A working transports are not a public release; packaging/update/support remain E7-B.

## Documented host modes

The executable does not exist yet. Contracts define one `wow` binary with:

```text
wow <existing one-shot command>
wow transport capabilities
wow daemon run|status|shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local   # explicit local-only profile
```

The initial LSP profile covers lifecycle, explicit workspace folders, incremental text synchronization, pull diagnostics, negotiated push compatibility, hover, definition, references, document/workspace symbols, completion, signature help, guarded code actions and call hierarchy.

The initial MCP profile exposes only implemented read-only service operations as fixed tools/resources. Effecting provider, calibration, publication, activation, rollback, edit and release operations are absent.

## Current evidence state

Documentation defines contracts, fixtures and freeze gates only:

```text
Cargo workspace and Rust source: 0
implemented E0-E7 operations: 0
real Reference Packs/project generations: 0
real analyzer/diagnostic/search/context execution: 0
real calibration/core-pack lifecycle execution: 0
live external provider adapters/mappings: 0
live daemon/LSP/MCP client tests: 0
public release artifacts: 0
```

Missing implementations, adapter/library pins, exact fixtures/checksums, provider/client/platform probes, benchmarks, runtime observations and release evidence remain blocked or `NotEvaluated`, never passed.

## Launch path

```text
first runnable:
    E0-A -> E0-F

useful internal alpha:
    first runnable + E1 + E2 + E3

developer preview:
    internal alpha + E4 + minimal implemented E7-A frontend

governed beta:
    developer preview + E5; E6 optional when enabled

public supported v1:
    selected beta scope + implemented E7-A + complete E7-B gates
```

E5 governance and optional E6 do not block the first E0 executable. See [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md).

## Routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
- [`docs/DECISIONS.md`](docs/DECISIONS.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
- [`crates/README.md`](crates/README.md)
- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/DEPENDENCY_GRAPH.md`](crates/DEPENDENCY_GRAPH.md)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`crates/wow-service/e7/`](crates/wow-service/e7/README.md)
- [`apps/wow/e7/`](apps/wow/e7/README.md)

## License

MIT. Third-party, provider-returned, addon and Blizzard-source artifacts retain separate provenance, license, notice, privacy and redistribution decisions.