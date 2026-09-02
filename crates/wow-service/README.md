# `wow-service` contract router

**Status:** planned service documentation is complete through E7-B; no Rust code exists.

`wow-service` is the only production crate allowed to coordinate multiple framework owners into one transport-independent public operation. It validates exact requests, resolves permitted symbolic selectors once, acquires retained owner views, sequences narrow ports, maintains durable effect and reconciliation state, and emits conservative canonical envelopes. It never reimplements owner algorithms or frontend wire protocols.

## Contract routes

- **E0-F:** root contract — `status`, `check`.
- **E1-D:** [`e1/README.md`](e1/README.md) — Reference Pack build, validation, and rebuild comparison.
- **E3-C:** [`e3/README.md`](e3/README.md) — exact context acquisition and use cases.
- **E4-C:** [`e4/README.md`](e4/README.md) — search, lineage, migration, impact, selection, and context orchestration.
- **E5-B:** [`e5/README.md`](e5/README.md) — calibration runs, review authorization, sealed holdout, consumption, and promotion submissions.
- **E5-C:** [`e5c/README.md`](e5c/README.md) — core artifact, signing, inactive publication, canary, activation, rollout, LKG, rollback, revocation, deactivation, and partition closure.
- **E6-B:** [`e6/README.md`](e6/README.md) — optional provider configuration, session authorization, durable result catalog, exact owner mapping, caller selection, and exact-root context sidecar.
- **E7-A:** [`e7/README.md`](e7/README.md) — canonical frontend operation registry, immutable sessions, explicit workspaces, project-owned overlays, local daemon, LSP 3.18, MCP 2025-11-25, progress, cancellation, reconnect, backpressure, and isolation.
- **E7-B:** [`e7b/README.md`](e7b/README.md) — source and release-plan validation, build and reproducibility, evidence and signing, deterministic bundle, support matrix, release candidate, channel publication, update manifests, installation, migration, update, rollback, revocation, retirement, incidents, and effect reconciliation.

## Service ownership

Service may:

- validate strict public requests, schemas, profiles, budgets, and capabilities;
- resolve an explicitly permitted symbolic selector once and record the exact resolution receipt;
- acquire exact retained immutable owner views and artifacts in a fixed order;
- validate cross-owner generation, profile, capability, privacy, license, and retention compatibility;
- invoke narrow owner, authorization, provider-session, build, signing, distribution, installation, and support ports as defined by one documented service operation;
- register and reconcile `OperationId + CanonicalRequestDigest` effects;
- publish service-owned result, session, selection, release, and lifecycle records through generic store ports;
- preserve partial, conflict, `NotEvaluated`, cancelled, failed, `OutcomeUnknown`, blocker, omission, and nonclaim state;
- perform required fresh read-back, retention, audit, and reverse-order close before public success.

Service may not:

- parse Lua, TOC, XML, project source, provider responses, archives, or frontend frames outside the owning crate or adapter;
- open raw database, storage, filesystem, parser, process, session, client, build, signing, provider, distribution, or installation handles;
- reproduce reference, project, graph, recognizer, rule, search, context, provider normalization, locator mapping, build, archive, signing, publication, installer, migration, or rollback algorithms;
- select latest, newest, best, highest, previous, first, last, sole, same-name, nearest, LKG, or LKR by inference;
- infer authorization from GitHub, repository, OS, CI, CLI, tag, file, editor, client, model, process, or transport identity;
- expose private credentials, keys, tokens, endpoints, provider cursors, environment blocks, or installer internals;
- expose generic MCP, RPC, tool, shell, script, plugin, model, SQL, HTTP, process, or callback execution;
- turn provider, source, client, release, or transport text into semantic facts, selectors, profiles, tools, or authorization;
- frame CLI, local-daemon, LSP, or MCP protocols or write stdout and stderr;
- apply or save source edits automatically;
- perform hidden update checks, downloads, telemetry, crash upload, or remote configuration;
- start detached retries, uploads, builds, updates, cleanup, or continuation work.

## Active dependency slices

Owner dependencies are operation-specific and narrower than the maximum reviewed graph.

```text
E0-F
    wow-core + fixture reference, Emmy, project, and rule owners

E3 and E4
    exact project, reference, graph, context, search, and rule owners

E5-C
    wow-core + wow-store + wow-project + wow-graph + wow-recognizers

E6-B
    wow-core + wow-store + wow-project + wow-reference + wow-graph
    + wow-context + wow-cbm

E7-A
    only implemented owner capabilities named by the exact registry entry

E7-B release slice
    wow-core + wow-store + narrow external source, materialization,
    build, evidence, signing, distribution, installation, and support ports
```

E5 effecting operations remain under their own authorization profiles and are absent from default LSP and MCP exposure. E6 remains optional and disabled until a real adapter passes its gates. E7-B publication and installation administration is not exposed through default developer frontends.

## E7-A service boundary

Canonical IDs:

```text
wow-service/e7-a/frontend-session-operation-registry
apps/wow/e7-a/frontend-transports
```

The operation registry is immutable and content-addressed. Runtime negotiation narrows only. Sessions are coordination state, not semantic fact stores. Every request binds one exact session, workspace, project, profile, and overlay generation.

`wow-project` owns unsaved document bytes, versions, file identity, and coordinate mapping. LSP 3.18 uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Stale or out-of-order changes return explicit resynchronization state.

Editor operations preserve exact versus Candidate evidence, coverage, conflicts, and document version. Code actions are guarded data; service does not apply, save, or execute them. Progress is nonsemantic. Streams bind exact artifact, digest, consumer, sequence, and cumulative budgets. Disconnect does not prove cancellation or no effect.

## E7-B service boundary

Release states remain independent:

```text
source validated
release plan validated
build submitted and build complete
unsigned reproducibility established
artifact and required tests validated
SBOM, provenance, licenses, notices, and checksums complete
portable and platform signatures validated
bundle ready
support matrix and candidate ready
provider objects and channel published with public read-back
signed update manifest available
installation or update planned
installed or updated after migration and self-check
LastKnownRunnable designated
rolled back
revoked or retired
```

At least two independent unsigned builds are required for a reproducibility claim. Platform signing follows unsigned digest freeze. Channel and installation current records use exact expected-current CAS. Assets are immutable by digest. Updates are explicit and staged. Windows executable replacement is owned by the exact verified installation-helper port. Store and configuration migrations use registered owner operations with backup, crash recovery, and rollback compatibility.

A successful build, signature, upload, channel update, helper handoff, or installation cannot be returned as another gate's success. Response loss at any effect produces exact reconciliation state and can remain `OutcomeUnknown`.

## Applications and tools

```text
apps/wow                     -> wow-service only
apps/wow-reference-builder   -> wow-service only
tools/wow-release            -> wow-service only
```

Each semantic command, method, tool, or resource request invokes exactly one public service operation unless a composite workflow is itself a documented service operation.

## Current state

```text
planned service documentation: complete through E7-B
implementation frontier: not-started
first service implementation: I0-F after I0-A through I0-E prerequisites
repository next package: I0-A / wow-core E0-A
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
supported public release: none
```
