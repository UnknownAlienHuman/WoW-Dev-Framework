# WoW Dev Framework

**Rust-first, editor-independent code intelligence, diagnostics, context, and agent tooling for World of Warcraft addon development.**

> **Planned architecture:** complete through E7-B.
>
> **Implementation frontier:** executable `wow-core` foundation and `wow-reference` source bridge. Full package acceptance remains incomplete. Next owner: the real `wow-emmy` adapter.
>
> **First runnable gate:** E0-A through E0-F (`wow status` and `wow check`).
>
> **First public target intent:** Windows x86-64 MSVC, not supported or advertised until the full implementation, platform, client, installation, update, rollback, and release matrix passes.
>
> Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

WoW Dev Framework is designed to provide exact, generation-bound, evidence-preserving technical context over addon repositories, Blizzard API/UI source, diagnostics, typed graphs, search, lineage, migration evidence, static impact, Project Maps, L0/L1/L2 context, governed structural-recognizer evolution, optional external semantic candidates, editor frontends, and reproducible public releases.

It is not a generic RAG product, source-edit executor, repository-specific heuristic engine, model-authority layer, provider database owner, arbitrary MCP/tool host, runtime injection platform, or automatic unreviewed release system.

## Contract stack

```text
E0  shared identities/evidence/results and first diagnostic vertical slice
E1  ReferenceStore/View, annotations, Reference Pack build/validation
E2  typed graph, recognizers, TOC/XML/load/project indexing and ProjectStore
E3  Blizzard UI source, Project Map/L0/L1/L2 context and service/CLI
E4  exact-generation search, lineage/migration/static impact and service/CLI
E5  calibration, independent review/holdout and immutable core publication lifecycle
E6  optional external Candidate bridge, exact owner mapping and context sidecar
E7-A one `wow` binary: CLI, local daemon, LSP 3.18, MCP 2025-11-25,
     sessions, project-owned overlays, progress, streams and recovery
E7-B reproducible build, evidence/signing, bundle/channel,
     install/update/rollback, support, revocation, retirement and incidents
```

Detailed state and implementation order:

- [`crates/MANIFEST.json`](crates/MANIFEST.json)
- [`crates/WORKSTREAMS.md`](crates/WORKSTREAMS.md)
- [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
- [`docs/WORKSPACE_AND_BUILD_PLAN.md`](docs/WORKSPACE_AND_BUILD_PLAN.md)
- [`docs/IMPLEMENTATION_HANDOFF.md`](docs/IMPLEMENTATION_HANDOFF.md)
- [`docs/CONFORMANCE_COMMANDS.md`](docs/CONFORMANCE_COMMANDS.md)
- [`docs/PROJECT_COMPLETION_MATRIX.md`](docs/PROJECT_COMPLETION_MATRIX.md)

## Planned public product

One public executable:

```text
wow <one-shot command>
wow daemon run|status|shutdown
wow lsp --transport stdio
wow mcp --transport stdio
wow mcp --transport streamable-http-local   # explicit, loopback-only, disabled by default
```

User-facing release operations:

```text
wow version
wow release status
wow release verify bundle --input <PATH>
wow installation validate
wow update check
wow update plan
wow update apply
wow update rollback
wow update reconcile
```

The internal `wow-release` tool performs exact service-backed release engineering and is excluded from the default public bundle.

## Core invariants

- One result or effect binds one exact coherent profile and generation set.
- Reference, user project, Blizzard UI source, calibration, core publication, external provider, session overlay, runtime, history, release, and installation remain separate universes.
- EmmyLua is the sole correctness-path Lua parser and analyzer.
- TOC, XML, source, archive, reference, provider, and protocol input is bounded and nonexecuting.
- Graph facts retain producer, evidence, confidence, coverage, conflicts, generation, axis, and partition ownership.
- Production recognizers are universal structural rules and never branch on repository, addon, owner, path, popularity, labels, splits, reviewers, holdouts, canaries, providers, or models.
- `Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed, `OutcomeUnknown`, `ResynchronizationRequired`, and `NotEvaluated` never become proof, clean Negative, complete, or pass.
- Similarity, rank, score, repetition, top or sole result never creates exact authority.
- Metrics, graph validity, review, holdout, signature, publication, canary, activation, rollout, distribution, installation, and runtime correctness remain independent gates.
- Response loss never proves no effect; exact reconciliation precedes redispatch.
- Applications, transports, and internal clients depend on `wow-service` only among framework crates.
- No public success precedes required validation/read-back, retention, audit, and reverse resource closure.

## E6 external-provider boundary

Every provider result remains:

```text
provenance = semantic_candidate
confidence = Candidate
negative_authority = unavailable
```

Provider paths, URIs, revisions, symbols, spans, labels, scores, summaries, and zero results remain unverified provider evidence until exact project/reference owners map the locator against one retained generation. Mapping proves locator identity only. Caller selection is not verification, and provider metadata remains a separate sidecar outside `ContextSemanticPack` truth. Provider failure is lane-local and never disables exact local capabilities.

## E7-A frontend boundary

The canonical service contract is [`crates/wow-service/e7/CONTRACT.json`](crates/wow-service/e7/CONTRACT.json), and the canonical host contract is [`apps/wow/e7/CONTRACT.json`](apps/wow/e7/CONTRACT.json).

```text
one immutable FrontendOperationRegistry
one transport request -> one service operation
explicit session/workspace/project/profile registration
exact versioned project-owned unsaved document overlays
wow-local-jsonrpc/1 local daemon
LSP 3.18
MCP 2025-11-25
```

- Runtime negotiation can narrow but never widen the operation registry, schemas, authorization, privacy, or resource policy.
- There is no generic `call_service`, arbitrary tool/RPC, shell, script, plugin, or model proxy.
- The local daemon uses current-user Windows named pipes or Unix-domain sockets; no default TCP or remote listener exists.
- LSP and MCP are explicit host modes and never silently auto-start or fall back to the daemon.
- E7-A defines incremental LSP `textDocument/didChange`; a full-document change is an exact replacement. Advertisement requires exact overlay, position-encoding, version, resynchronization, client, and resource conformance.
- Exact and Candidate definitions/references remain distinct. Clean zero requires owner negative authority.
- Code actions are guarded data; the framework does not automatically apply, save, or execute them.
- Default MCP tools and resources are fixed and read-only. Prompts, sampling, elicitation, tasks, arbitrary tools, provider effects, governance effects, source mutation, and release effects are absent.
- Disconnect is not cancellation. Progress is not completion. Response replay returns retained bytes and never reexecutes service.
- Sessions isolate workspaces, overlays, authorization, private source, provider access, operations, results, streams, and response journals.

## E7-B release boundary

A compiled binary is not a release. The exact planned pipeline is:

```text
source tree + Cargo.lock + Rust toolchain + dependency closure
-> typed isolated build
-> at least two independent unsigned builds
-> reproducibility comparison
-> artifact self-description and complete required tests
-> SBOM/provenance/license/notices/checksums
-> portable and platform signatures
-> deterministic ReleaseBundle
-> exact support matrix and immutable ReleaseCandidate
-> provider-neutral publication and public read-back
-> guarded channel CAS and signed update manifest
-> staged verified install/update/migration/self-check
-> explicit LastKnownRunnable
-> exact rollback/revocation/retirement/incident lifecycle
```

Private signing, build, distribution, provider, and installation credentials never enter the repository, public configuration, fixtures, logs, bundles, or canonical results. GitHub Releases may be a distribution adapter, but a repository, tag, CI job, account, successful upload, or asset name is not release authorization or artifact trust.

Updates are explicit by default. Check, download, verify, stage, backup, migrate, activate, self-check, designate LKR, clean up, and roll back are separate exact states. The public app never overwrites its running executable or executes arbitrary installer commands. Windows replacement uses the exact verified installation-owner helper protocol.

## Current executable state

The active workspace contains `wow-core`, `wow-reference`, `wow-annotations`
and the internal `xtask` maintenance tool. The Rust Ketho path loads current
Blizzard documentation and generates annotation libraries without an external
interpreter. `cargo xtask check` validates repository policy and synchronized
skills; `manifest`/`verify-manifest` inventory exact local Git snapshots.
Existing API/topology JSON importers remain native compatibility readers. The
retired legacy topology producer has not yet been replaced by a full native one.

See [the implementation ledger](docs/IMPLEMENTATION_STATUS.md) for implemented
commands, update policy and explicit nonclaims. The planned public `wow` binary,
real analyzer adapter and full release gates are not implemented.

## Launch path

```text
R0  E0-A through E0-F: first `wow status` / `wow check` executable
A0  E1 through E3: useful exact local analysis and context
A1  E4 plus an implemented E7-A frontend: developer preview
B0  E5 and optional enabled E6: governed beta
V1  selected scope plus complete E7-A/E7-B Windows release evidence
```

## Next implementation step

Implement I0-C behind the `wow-emmy` adapter, without a permanent upstream
revision or client-build dependency. Preserve native source and wire-compatibility tests. Do
not activate placeholder crates or treat a compiled upstream as our adapter.

## Routes

- [`AGENTS.md`](AGENTS.md)
- [`docs/README.md`](docs/README.md)
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md)
- [`docs/PROVENANCE_AND_COVERAGE.md`](docs/PROVENANCE_AND_COVERAGE.md)
- [`docs/DECISIONS.md`](docs/DECISIONS.md)
- [`docs/ROADMAP.md`](docs/ROADMAP.md)
- [`docs/LAUNCH_GATES.md`](docs/LAUNCH_GATES.md)
- [`crates/README.md`](crates/README.md)
- [`crates/wow-service/e7/`](crates/wow-service/e7/README.md)
- [`apps/wow/e7/`](apps/wow/e7/README.md)
- [`crates/wow-service/e7b/`](crates/wow-service/e7b/README.md)
- [`apps/wow/e7b/`](apps/wow/e7b/README.md)
- [`tools/wow-release/`](tools/wow-release/README.md)
- [`release/README.md`](release/README.md)

## License

MIT for framework-owned code. Third-party, provider-returned, addon, and Blizzard-source artifacts retain separate provenance, license, notice, privacy, and redistribution decisions.
