# `wow-release` internal E7-B release tool

**Status:** implementation-ready documentation; no Rust code exists.

**Contract ID:** `tools/wow-release/e7-b/release-publisher-client`

`wow-release` is an internal thin client over `wow-service`. It exposes explicit release-engineering commands for source/build/evidence/signing/bundle/candidate/channel/revocation work. It does not contain release semantics, execute arbitrary shell commands, read private credentials, or mutate distribution/install state directly.

## Dependency boundary

```text
tools/wow-release -> wow-service
```

No direct dependency on lower framework crates, Cargo internals, signing SDKs, GitHub APIs, installer APIs, provider clients, store connections or filesystem build executors.

## Commands

```text
wow-release status
wow-release source validate
wow-release plan validate
wow-release build submit|get
wow-release rebuild compare
wow-release artifact validate
wow-release sbom build
wow-release provenance build
wow-release sign request|validate
wow-release bundle build|validate
wow-release support validate
wow-release candidate validate
wow-release channel prepare|publish|get
wow-release update-manifest build|validate
wow-release revoke
wow-release retire
wow-release reconcile
```

Each valid command invokes exactly one E7-B service operation.

## Inputs

Commands accept exact IDs and strict bounded request/profile files. They never accept raw commands, environment blocks, private keys, tokens, endpoints, arbitrary URLs, GitHub API payloads, SQL, installer scripts or callback plugins.

Source selection uses exact repository/commit/tree identities. Branch/tag convenience selectors, if ever accepted by a service profile, are resolved once to exact retained identities and guarded; the tool never treats them as release source proof.

## Build behavior

`build submit` sends an exact validated `ReleasePlan` ID/request to service. The build executor, sandbox, Cargo/rustc/linker invocation and output paths remain behind service-owned typed ports. The tool does not spawn Cargo or shell as a side effect of the command.

Developer convenience commands that directly run local Cargo belong to ordinary implementation development, not the governed public release path.

## Signing and publication

The tool supplies nonsecret authorization/key/provider references only. Service signs and publishes through narrow adapters. GitHub login, repository ownership, environment token presence, tag and CI identity do not authorize signing/channel effects.

## Output

Output modes:

```text
envelope-json
text
artifact
```

JSON is exact canonical service bytes plus one LF. Text preserves every gate/blocker/`NotEvaluated`/`OutcomeUnknown`/nonclaim. Artifact output is one exact approved public/internal release artifact under the consumer policy.

## Current state

```text
documentation frontier: E7-B
implementation frontier: not-started
public bundle inclusion: false by default
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```