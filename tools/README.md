# Tools

Internal tools are thin clients or deterministic validators around owned framework and service contracts. They are not alternate implementations of product semantics.

## `xtask`

Implemented native repository maintenance: [commands, scope and limits](xtask/README.md).
It has no framework crate dependencies and is not a product/service or release
implementation. It owns the single current policy/skill/source-inventory check
implementation. Development snapshot IO is explicit, not a hidden analysis port.

## `wow-release`

Read [`wow-release/README.md`](wow-release/README.md).

E7-B defines `wow-release` as the internal release-engineering client for exact source, plan, build, evidence, signing, bundle, support, candidate, channel, update-manifest, revocation, retirement, and reconciliation operations.

```text
tools/wow-release -> wow-service
```

It does not:

- run Cargo, rustc, linker, shell, PowerShell, or arbitrary processes directly;
- traverse source or create archives independently;
- read signing, distribution, build, CI, or provider credentials;
- call raw GitHub or distribution-provider APIs;
- mutate installations, stores, configuration, or update state;
- select latest, newest, previous, highest, first, or same-name artifacts;
- contain a second release pipeline.

Each valid command invokes exactly one exact E7-B service operation. The tool is excluded from the default public product bundle unless a separate administrative artifact and support profile explicitly includes it.

## Deterministic repository validation

Repository contract, manifest, dependency, fixture, checksum, documentation-link, and forbidden-file validation is specified in [`../docs/CONFORMANCE_COMMANDS.md`](../docs/CONFORMANCE_COMMANDS.md).

The implementation may live in an owned development validator or service-backed `wow-release` command, but there must be one canonical implementation. Validation commands compare committed expected bytes and never rewrite normative fixtures or checksums.

## Other future utility classes

Utilities may be introduced only when an implementation package owns them and their exact command, input, output, dependency, security, and release behavior is defined. Possible classes include:

- Reference Pack build and differential evaluation;
- fixture minimization and mutation;
- Ketho, Numy, and LuaLS parity adapters;
- corpus manifest and license capture;
- upstream compatibility probes;
- schema compatibility simulation;
- deterministic output comparison;
- benchmark and agent-task evaluation runners.

## Rules

- internal tools import only their documented owner or `wow-service` dependencies;
- no generic shell, process, tool, RPC, plugin, callback, or model executor;
- no hidden network, current, default, project, provider, update, or credential discovery;
- no secret material in arguments, public configuration, fixtures, logs, or results;
- one command maps to one owner or service operation unless the composite is itself a documented operation;
- no CI or release automation before exact implemented commands and evidence exist;
- public-bundle inclusion requires an explicit artifact, security, compatibility, and support profile.

## Current state

```text
planned documentation: complete through E7-B
implemented internal maintenance package: tools/xtask
product/release clients: not implemented
active owner crates and next work: docs/IMPLEMENTATION_STATUS.md
```
