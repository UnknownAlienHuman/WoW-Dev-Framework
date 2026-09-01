# E3-C CLI output, exit codes, and stream discipline

**Status:** normative.

## Output modes

### `envelope-json`

Canonical default.

- stdout is exactly the canonical `wow-service` operation result JSON bytes plus one final LF;
- no BOM, banner, progress, color, log, prompt, path, or extra whitespace;
- app does not add/remove/reorder semantic fields;
- valid service Failure/Cancelled results also use stdout;
- stderr remains empty for a successfully parsed command that produced a serializable service result.

The final transport LF is not part of the service result digest unless the service serialization contract explicitly includes it; freeze both service bytes and CLI stdout bytes.

### `text`

Noncanonical human projection from public service result fields only.

Must show, when present:

```text
operation and service status
symbolic selector kind and exact resolved publication/generation IDs
exact ContextUniverseSet/root/profile IDs
artifact IDs and validation state
partial/truncated/NotEvaluated state
coverage/conflict/omission/loss counts and blockers
budget and continuation state
resource closure failure when operation failed
```

It must not:

- infer safe/working/runtime verified/tests passed/release ready;
- hide source denial, conflicts, omitted scopes, or continuation expiry;
- reconstruct map/skeleton/context semantics;
- print source excerpts unless they are inside an explicitly returned rendered artifact and the user chose artifact mode;
- change exit code/status.

### `artifact`

- requires exactly one eligible validated `RenderedContextArtifact` in the service result;
- writes exact artifact bytes unchanged;
- no final newline is added unless already present in artifact bytes;
- no envelope, header, status, progress, ID, warning, or log on stdout;
- exit code still reports complete versus partial/truncated/failure;
- forbidden for status/map/inspect/validate and ambiguous multi-artifact results.

The app treats artifact bytes as opaque validated output. It does not parse/re-render source content.

## Exit codes

Existing E0 commands retain their frozen mapping.

### `context status`

```text
0   complete/available status result
2   partial or NotEvaluated status result
3   structured request/acquisition/unavailable/deferred failure
4   internal owner/service/result/serialization/output failure
64  CLI/config transport error before service invocation
130 cancelled
```

### `context map`, `inspect`, `build`, `continue`, `render`

```text
0   complete
2   partial, truncated, or not_evaluated
3   structured request/selector/generation/profile/capability/artifact/retention failure
4   internal owner/context/service/closure/serialization/output invariant failure
64  CLI/config/artifact-input parser or transport error before service invocation
130 cancelled
```

### `context validate`

```text
0   validation operation complete and artifact Valid
1   validation operation complete and artifact Invalid
2   validation result Partial or NotEvaluated
3   structured request/exact-owner-generation/unavailable/deferred failure
4   internal parser/owner/context/service/closure/serialization/output failure
64  CLI/config/artifact-input transport error before service invocation
130 cancelled
```

Exit 1 is not a service internal failure. It represents a successfully executed negative validation result.

## Failure-class mapping

Exit 3 examples:

```text
unknown exact project/store/publication/root/profile
expected-current guard mismatch
independent selected publications incompatible
exact continuation generation/receipt unavailable
required capability unavailable
exact-owner validation generation unavailable
operation unsupported/deferred
```

Exit 4 examples:

```text
owner result violates contract
context artifact/result violates invariants
resource close fails
canonical serialization fails
output write/broken pipe prevents defined output completion
internal invariant/panic boundary result
```

## Parser/startup errors

Use exit 64 and stderr only. Do not fabricate a service JSON result when service was never invoked.

Error text is bounded and safe:

```text
command/flag/error code
expected grammar category
no source/artifact body
no credential/private path echo beyond an explicitly permitted sanitized argument label
```

For explicit input/config file errors, do not copy arbitrary full path into canonical/structured output. A local stderr diagnostic may use a sanitized basename only under the app profile.

## Broken pipe and write failures

E3-C v1 behavior:

- never reinvoke service;
- never emit a second result;
- stop writing immediately;
- map output failure to exit 4 unless cancellation signal already owns exit 130 under the frozen platform adapter;
- service-side resources are already closed before the app receives the result;
- no partial JSON followed by an error JSON;
- no partial artifact followed by text.

A future platform-specific benign broken-pipe policy requires a new app profile/version and cannot silently change v1.

## Cancellation and signals

- one cancellation source per command;
- first supported termination signal requests cancellation;
- no new service call after signal;
- service Cancelled result maps 130;
- cancellation before service invocation maps 130 with stderr empty or one bounded local diagnostic according to the frozen adapter;
- no partial/double output;
- second signal/forced termination behavior is platform-specific operational handling, not semantic success.

## Logging and progress

Default canonical modes emit no progress. Optional diagnostic logging, if later activated, is stderr-only, bounded, privacy-filtered, disabled in byte-golden tests, and never changes service request/result/exit mapping.

## Determinism

For the same service result and output profile:

- envelope JSON stdout bytes identical;
- artifact bytes identical;
- text required fields/counts/status identical, with any noncanonical wrapping frozen per profile;
- exit code identical;
- no dependence on terminal width/color, locale, timezone, cwd, host, clock, temp root, or logging.

## Output validation

Before writing:

- validate selected output mode is permitted for command/result;
- envelope JSON bytes match service canonical serializer/digest;
- artifact ID/profile/digest/length and eligibility match result;
- text projection references only public result fields;
- predicted/actual stdout byte limit passes;
- no private/source fields outside permitted artifact output.
