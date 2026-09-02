# E6-B CLI output, exit codes, files, and streams

**Status:** normative.

## Output modes

### `envelope-json`

Writes exact canonical service envelope bytes plus one LF. No banner, progress, warning, or second JSON document appears on stdout.

### `text`

Faithful noncanonical projection preserving:

```text
service and validation status
provider/descriptor/external-state class and exact IDs
result/candidate/artifact identities
Candidate provenance/confidence and no negative authority
provider-local score/rank labels where disclosed
mapping status and exact owner generation/root
explicit selection origin and nonauthority state
context status and separation of external attachment from framework facts
partial/conflict/truncation/continuation/OutcomeUnknown
privacy/license/credential/retention/audit/closure state
mandatory nonclaims
```

Text never calls a Candidate verified source, proven relation, API truth, runtime result, or authoritative absence.

### `artifact`

Writes exactly one service-returned eligible artifact without wrapper/newline modification. Raw provider databases, credentials, private endpoints, session handles, provider cursors, private source, or internal owner handles are never ordinary artifact output.

## File output

```text
validate explicit path policy
-> create staging file without forbidden link/device traversal
-> write exact bytes
-> flush/durability action under frozen platform profile
-> atomic replace when supported/required
-> verify size/digest when required
```

Output paths never derive from provider path/URI/symbol/snippet, mapped source, or selection note.

## stdout/stderr

- stdout contains requested bytes only;
- stderr contains bounded diagnostics/progress when enabled;
- no ANSI/color in machine modes;
- default diagnostics omit credentials, private endpoints, session handles, cursors, private locators/snippets/source, and raw owner/store details;
- locale/terminal width do not alter canonical output.

## Broken pipe

Stop promptly, never invoke service again, never switch output mode/destination, and never report successful delivery. Durable service/provider artifacts retain their exact state.

## Exit codes

```text
0    Complete or NoChange; completed validation Valid;
     exact mapping validation Valid; explicit selection record completed;
     context completed under requested profile

1    completed validation Invalid; checked selection Rejected/Invalid where
     the command profile represents it as a completed invalid result

2    CandidateOnly, Partial, Blocked, ConflictBlocked, Truncated,
     NotEvaluated, MultipleMappings, NoMappingPartial, Deferred

3    structured request/provider/session/state/result/mapping/selection/
     context/cache/retention/privacy/license/domain failure

4    OutcomeUnknown or internal owner/service/closure/canonicalization/
     serialization/post-service output failure

64   CLI/config/input/output-path failure before service invocation

130  Cancelled
```

A successful provider query normally exits 2 because `CandidateOnly` is not a proven result.

## Command-specific mapping

- provider/generation/result/mapping/selection/cache validation `Valid`: 0; `Invalid`: 1; `NotEvaluated`: 2;
- provider query with candidates or zero-result Candidate envelope: 2;
- `ExactMapped` completed mapping: 0;
- `MultipleMappings`, `NoMappingPartial`, mapping `NotEvaluated`: 2;
- explicit `Selected` receipt: 0; `Rejected`: 1; `Deferred`: 2;
- context `Complete`: 0; partial/conflict/truncated/NotEvaluated: 2;
- provider unavailable optional degradation: 2 unless exact service payload reports a structured infrastructure failure requiring 3;
- `OutcomeUnknown`: 4;
- transport parse/path error before service: 64;
- cancellation: 130.

## Cancellation

Signals map to typed cancellation. App does not issue a second reconcile/cancel/query automatically. A cancelled artifact is not emitted as ordinary complete artifact unless the exact service/profile explicitly marks a safe eligible artifact; default is envelope output.

## Determinism

Identical service bytes and output profile produce identical stdout bytes, required text field order, stderr policy, and exit code. Terminal, locale, clock, provider latency, cache state, and host paths cannot change canonical output.
