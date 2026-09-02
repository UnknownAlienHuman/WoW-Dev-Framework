# AGENTS.md — `wow-service`

## Current state

```text
planned service documentation: complete through E7-B
service implementation: not started
first service implementation package: I0-F / E0-F
repository next package: I0-A / wow-core E0-A
```

Do not implement E7 service operations before prerequisite owner packages. Later documentation defines the final interface, but implementation follows the dependency and launch order.

## Required routing

Read repository and crate instructions, the service router, dependency graph, workstreams, workspace plan, implementation handoff, conformance commands, completion matrix, then exactly one service package:

```text
E0-F -> root E0 service contract
E1-D -> e1/
E3-C -> e3/
E4-C -> e4/
E5-B -> e5/
E5-C -> e5c/
E6-B -> e6/
E7-A -> e7/
E7-B -> e7b/
```

Read current external WoW engineering KB routes for patch-sensitive work and actual addon instructions for addon-facing operations.

## Common ownership

`wow-service` coordinates public owner contracts. It owns request validation, one-time selector resolution, exact retained acquisition, cross-owner compatibility, use-case ordering, durable operation state, service-owned records, conservative envelopes, retention, audit, and close-before-success.

It never owns lower semantic algorithms, physical storage, frontend framing, build execution, signing keys, distribution APIs, installation mechanics, or migration implementation.

## Selector discipline

- Resolve a permitted symbolic selector only at the service boundary and only once.
- Replace it with an exact retained identity and resolution receipt before owner invocation.
- Never select latest, newest, best, highest score or metric, previous, first, last, sole, same-name, nearest, default, LKG, or LKR.
- Never silently substitute another profile, generation, owner, provider, mapping, release, installation, or rollback target.
- Continuation, replay, reconnect, retry, and reconciliation reopen the original exact identities and cumulative budgets.
- A stale expected-current CAS fails; do not rebase silently.

## Owner-port discipline

- Use narrow typed owner ports.
- No raw SQL, database connection, transaction callback, parser, process, session, client, filesystem root, mutable graph or project handle, provider API, command runner, signing SDK, distribution client, installer, or helper handle.
- If an owner seam is missing, request the smallest exact seam rather than reproducing it in service.
- Validate owner, profile, generation, schema, capability, coverage, conflict, privacy, license, retention, and close state on input and output.
- Owner-neutral projections are allowed only where a normative seam contract defines their exact data and proof ceiling.

## Effect discipline

- Register `OperationId + CanonicalRequestDigest` before every externally observable or durable effect.
- Persist exact prepared, dispatch, commit, no-effect, unknown, read-back, delivery, retention, audit, and close receipts as applicable.
- Same operation ID with a different request digest fails.
- Response loss is not proof of no effect; `OutcomeUnknown` blocks blind repetition.
- Provider, build, sign, upload, channel CAS, install, migration, helper, rollback, revocation, and retirement effects require exact owner reconciliation.
- No public success before mandatory validation or read-back, retention, audit, and reverse-order resource closure.
- No detached or background retry, upload, build, update, cleanup, polling, or continuation.

## Authorization and credentials

- Authorization is separate from semantic proof and transport access.
- GitHub, repository, OS, CI, CLI, editor, client, model, file, commit, tag, process, and transport identity is not review, holdout, signing, provider, release, install, or rollback authorization.
- Review, holdout, core signing and publication, canary and activation, provider use, release build and signing, channel publication, update, rollback, revocation, retirement, and incident scopes remain independent.
- Service requests and results contain nonsecret references and receipts only.
- Private keys, tokens, passwords, cookies, endpoints, environment blocks, signing sockets, provider cursors, and process handles never enter canonical requests, fixtures, logs, or envelopes.

## Status and authority discipline

Preserve exact operation payload states and conservative outer status. Never hide or coerce:

```text
Candidate and Possible
Partial, Truncated, Busy, Conflict, and Blocked
NotEvaluated, Cancelled, Failed, OutcomeUnknown, and ResynchronizationRequired
PublishedInactive, canary, rollout, activation, LKG, rollback, and revocation state
provider mapping, caller selection, and context sidecar scope
build, reproducibility, evidence, signature, bundle, channel, install, update,
LKR, rollback, revocation, retirement, and incident scope
```

Empty output is not `NoChange` or clean Negative without exact owner proof.

## E7-A discipline

- Canonical IDs are `wow-service/e7-a/frontend-session-operation-registry` and `apps/wow/e7-a/frontend-transports`.
- Registry, capability, and schema sets are exact and content-addressed; runtime can only narrow.
- Session changes create immutable generations; each request binds one exact session, workspace, project, profile, and document overlay state.
- Workspaces and projects are never inferred from cwd, Git, editor, WoW installation, document URI, or MCP root.
- `wow-project` alone owns document bytes, versions, file identity, and coordinate mapping.
- LSP 3.18 uses incremental `textDocument/didChange`; a full-document change is an exact replacement. Stale and out-of-order changes preserve explicit resynchronization.
- Exact and Candidate editor locations remain distinct. Clean zero requires owner negative authority.
- Code actions are guarded data; service does not apply, save, or execute edits.
- Progress is nonsemantic. Streams bind exact artifact, digest, consumer, sequence, and cumulative budgets.
- Disconnect, cancellation, output loss, and replay preserve exact operation and effect state.
- Default MCP 2025-11-25 exposure is fixed and read-only, without prompts, sampling, elicitation, tasks, arbitrary tools, provider effects, governance effects, source mutation, or release effects.
- No generic service invocation or transport-specific semantic fork.

## E7-B discipline

- Source identity is the exact complete tree, not a tag, branch, filename, working directory, or version string.
- Release plan freezes lockfile, toolchain, target, features, dependencies, build scripts, native tools, environment, deterministic path and time profile, and executor.
- Build goes through a typed executor, never arbitrary command, environment, network, SQL, provider API, or callback input.
- Reproducibility requires at least two independent unsigned builds.
- Artifact validation, required tests, SBOM, provenance, licenses, notices, checksums, signatures, bundle, support matrix, candidate, channel, and installation are independent records.
- Platform signing occurs after unsigned digest freeze.
- Channel object upload, public read-back, manifest publication, and expected-current CAS are separate effects. Provider metadata does not define trust.
- No in-place asset replacement, implicit latest selection, or automatic stable promotion.
- Updates are explicit and staged. Check, materialize, verify, stage, backup, migrate, helper handoff, current CAS, self-check, LKR, cleanup, and rollback remain separate.
- Running Windows executable replacement is owned by the exact installation and helper port, not service filesystem or process code.
- Store and configuration migrations use registered owner operations, verified backup and restore, crash recovery, and explicit rollback compatibility.
- Support claims match exact tested target, OS, protocol, store, schema, data-pack, WoW-profile, client, feature, and resource matrices.
- Revocation, retirement, and incident state remain distinct and immutable.

## Application and tool boundary

Applications and tools depend only on `wow-service`, invoke one operation per semantic request, and never access lower owners or protected adapters directly. Service does not parse CLI flags, local-daemon frames, LSP or MCP messages, write stdout or stderr, or construct process exits.

## Security

Service semantic code cannot execute source, repository, build, release, installer, or migration scripts; raw shell, SQL, RPC, MCP, tools, models, provider APIs, or arbitrary callbacks. Source, provider, client, and release text remains data. All bytes, collections, nesting, time, memory, calls, queues, streams, and retries are bounded by exact profiles.

## Documentation versus implementation

Do not add service Cargo or Rust code, workflows, placeholder modules, fake owner effects, fake reviewers, vaults, providers, builders, signers, publishers, installers, or passing measurements until prerequisite packages and first-commit freeze gates exist.

When implementation begins, activate only the selected service slice and populate every required freeze field before the first corresponding Rust commit.

## Completion report

```text
owned implementation package and operation set
exact prerequisite, profile, generation, target, and fixture inputs
ports and service-owned records
new dependencies and features
fixtures, checksums, and evidence
commands with pass, fail, skipped, or NotEvaluated
idempotency, response loss, read-back, retention, audit, and close
security, privacy, license, authorization, and credential state
launch and completion state advanced or unchanged
remaining exact blockers
```

Do not start another service package while the current worktree remains open.
