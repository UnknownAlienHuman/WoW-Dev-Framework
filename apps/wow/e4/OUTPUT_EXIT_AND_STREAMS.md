# E4-C CLI output, exit codes, files, and streams

**Status:** normative.

## Output modes

### `envelope-json`

Writes the exact canonical `wow-service` envelope bytes followed by one LF. The LF is CLI framing and is not part of the service semantic digest.

No progress, labels, banners or warnings precede/follow the JSON on stdout.

### `text`

Writes a faithful noncanonical projection from public service fields only. It must preserve exact IDs and all material uncertainty:

```text
status and validation state
exact generation/shard/snapshot/result IDs
candidate authority band and selection state
lineage proof ceiling/ambiguity/review state
migration not-applied/runtime-not-verified state
impact static-only/path/confidence/coverage state
partial/conflict/truncation/NotEvaluated/omissions
continuation availability
```

Text cannot invent a replacement, intended entity, direct edge, runtime breakage, severity or successful migration.

### `artifact`

Writes exactly one service-returned validated artifact byte sequence with no wrapper, newline conversion or metadata injection.

Allowed only when command/profile declares a single eligible artifact, such as a canonical query/result artifact, lineage snapshot manifest, validated migration recipe representation or deterministic report. Raw SQLite shard/database files are not emitted through ordinary artifact mode unless a future explicit secure export contract exists.

## Output destination

```text
--output -        stdout
--output <PATH>   explicit file
```

Default is stdout.

File output:

```text
validate explicit root/path policy
-> create bounded staging file without following forbidden links
-> write exact bytes
-> flush and apply platform durability policy
-> atomically replace destination when supported/required
-> verify final byte length/digest where profile requires
-> report success only after completion
```

No source/project path is inferred from artifact metadata.

## stdout/stderr

- stdout: requested output bytes only;
- stderr: bounded diagnostics/progress when enabled;
- sensitive query/source/review/migration content is omitted/redacted by default;
- no ANSI/color on stdout in machine modes;
- locale/terminal width do not change canonical JSON/artifact bytes;
- progress cannot change service request identity.

## Broken pipe

Broken stdout pipe:

- stops output promptly;
- never reruns service;
- never falls back to a file or text mode;
- never emits a second JSON document;
- maps to the frozen output-failure behavior unless cancellation profile explicitly classifies it;
- preserves no false successful delivery claim.

## Exit code map

```text
0    Complete or NoChange; completed validation state Valid;
     exact authoritative miss can still be successful when service status is Complete

1    completed validation state Invalid;
     completed review-validation semantic/authorization rejection when command profile maps it as a checked invalid payload

2    Partial, CandidateOnly, ConflictBlocked, Truncated or NotEvaluated

3    structured service-domain failure before/without internal corruption:
     invalid request/selector/profile/generation/shard/snapshot/candidate/continuation
     unavailable exact retained artifact/capability
     current acquisition instability
     review authorization unavailable/denied when represented as service failure
     retention admission failure

4    internal owner/service/lifecycle/closure/canonicalization/serialization/output failure

64   CLI/config/input/output path/transport parsing failure before service invocation

130  Cancelled
```

The exact mapping uses operation payload plus service status. App does not infer from human messages.

## Operation-specific rules

### Search query

- `Complete` with candidates or `NoCandidatesUnderExecutedLanes`: 0;
- `CandidateOnly`: 2;
- partial/truncated/not-evaluated/conflict: 2;
- exact shard unavailable: 3.

### Search select/context

- complete explicit receipt/context: 0;
- candidate guard mismatch/automatic selection request: 3;
- partial context: 2.

### Validate commands

- `Valid`: 0;
- `Invalid`: 1;
- `NotEvaluated`: 2;
- service/internal failure: 3 or 4 by typed error class.

### Lineage build/query

- accepted complete snapshot/query: 0;
- complete Candidate-only result: 2;
- unresolved conflict/partial/truncated/NotEvaluated: 2;
- unavailable comparison/snapshot: 3.

### Review validate/apply

- Valid/Authorized: 0;
- checked Invalid/Unauthorized/Rejected under the command profile: 1;
- NotEvaluated authorization: 2 or typed service unavailable 3 according to exact payload;
- stale apply target/request error: 3;
- internal publication/closure failure: 4.

### Migration

- candidate/validated recipe complete: 0;
- Candidate-only/Conflict/NotEvaluated: 2;
- Invalid recipe: 1;
- no apply command exists.

### Impact

- complete bounded static result: 0;
- truncated/partial/candidate/conflict/not-evaluated: 2;
- stale/missing exact plan/snapshot: 3.

## Cancellation

- signal before or during service call propagates typed cancellation;
- service `Cancelled` maps to 130;
- app closes transport resources and does not invoke service again;
- cancelled partial artifacts are never written as ordinary complete artifact unless the service/profile explicitly returns an eligible cancelled-state artifact and output mode names it; default is envelope only.

## Output failure after service success

If service succeeded but the app cannot deliver output:

- do not report exit 0;
- do not call service again;
- stderr may report stable result ID and bounded delivery error if safe;
- exact durable owner/service artifact remains under its existing identity;
- exit 4 for post-service output failure, or 64 when the destination was invalid before invocation.

## Determinism

For the same exact service bytes and output profile:

- envelope JSON/artifact output bytes are identical;
- text required fields/order/labels are identical;
- exit code identical;
- terminal, locale, path spelling, progress, timing and worker count do not alter semantic output.
