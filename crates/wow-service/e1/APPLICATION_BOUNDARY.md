# E1-D application and adapter boundary

**Status:** normative boundary between `wow-service` and `wow-reference-builder`.

## Principle

The application translates explicit CLI/filesystem concerns into typed service requests and executes typed materialization/finalization plans. It never recreates pack domain policy.

## Application-owned behavior

- command/argument parsing;
- reading explicit request/config JSON;
- opening the user-selected materialized source root through a root-confined adapter;
- creating an isolated staging directory;
- executing `PackMaterializationPlan` entries with safe file primitives;
- atomic rename/swap where supported;
- stdout/stderr and exit-code projection;
- bounded progress display;
- cleanup/quarantine prompts/policy supplied in the request;
- invoking reviewed external test adapters only when explicitly configured by the service request/profile.

## Service-owned behavior

- request validity and component/profile selection;
- build/validation/rebuild state machine;
- component invocation order;
- member set/layout/checksum/manifest/gate policy;
- pack eligibility;
- recovery instruction;
- canonical semantic output.

## Ports

Narrow adapters may include:

```text
MaterializedSourcePort
    list/open exact manifest-declared files under one root

PackStagingPort
    create session; write/copy declared entry; verify; list; remove/quarantine

AtomicDestinationPort
    observe destination; finalize staged root atomically when supported; reopen

ExternalProbePort
    run one exact reviewed oracle/consumer adapter request in isolated environment
```

No generic shell, arbitrary path read/write, process callback, SQL handle, network client, or repository executor.

## CLI commands

E1 dedicated binary:

```text
wow-reference-builder build --request <file> --source-root <dir> --output <dir> [--json]
wow-reference-builder validate --pack <dir> [--expect <file>] [--json]
wow-reference-builder rebuild-compare --request <file> --source-root <dir> --scratch-root <dir> [--json]
```

Exact spelling may change only with contract update and fixtures. No implicit default source/output/current profile.

## Exit classes

```text
0   operation completed and requested gate passed
2   request/config/usage invalid
3   candidate built or inspected but requested eligibility blocked/partial
4   validation failed
5   component/build failure
6   cancelled
7   security/path/integrity violation
8   operation unavailable for milestone/profile
```

Machine-readable JSON remains authoritative. Human text never changes semantics.

## Filesystem safety

- canonicalize and confine roots before access;
- reject source/output/staging overlap when unsafe;
- reject traversal/device/reserved paths and unsafe symlinks/reparse points;
- no destination deletion/replacement before validated staging completion;
- no writes outside staging/final destination;
- fixed safe file modes; no executables;
- verify bytes/digest after write;
- do not follow manifest paths not declared by service plan.

## External tools

The app does not discover commands from source repositories. Exact adapter executable/revision/config is repository-owned and request/profile-bound. Environment is sanitized; network/user settings/extensions are disabled unless a future reviewed profile explicitly requires and documents them.

## No publication

E1 CLI does not upload to GitHub, CurseForge, package registries, release channels, web storage, or an update feed. It does not sign or activate globally.

## Tests

- every command/argument/exit-code path;
- JSON/text semantic equivalence;
- source/output/staging confinement;
- path/symlink/collision/missing member mutations;
- atomic finalization failure preserves prior destination;
- cancellation and cleanup/quarantine;
- no shell/network/editor/source execution;
- no direct dependency on lower framework crates.
