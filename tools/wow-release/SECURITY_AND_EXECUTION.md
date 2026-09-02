# `wow-release` execution, credentials, output, and failure contract

**Status:** normative.

## Thin-client boundary

`wow-release` parses strict transport input, constructs one typed E7-B service request, invokes service exactly once, and emits one result. It does not implement or directly access:

```text
Git/source tree traversal
Cargo/rustc/linker/process execution
build sandbox or environment
SBOM/provenance scanners
signing keys or SDKs
GitHub/distribution APIs
archive creation/extraction
installer/helper/store migration
channel/current pointers
```

These are service/owner responsibilities.

## Configuration

Public tool configuration is explicit, strict, bounded and nonsecret. It may select exact service endpoint/profile, input/output policy and nonsecret authorization references. It has no include/interpolation/environment expansion, command substitution, plugin, dynamic library, script or arbitrary URL/API configuration.

Baseline direct mode hosts `wow-service` in-process or uses an explicitly documented E7-A local-daemon client profile. It never silently switches modes. The exact transport profile is visible in the request receipt.

## Credentials

Private signing/distribution/build/provider credentials are unavailable to the tool. Presence in environment, Git credential helper, `gh`, SSH agent, OS user session or CI does not authorize or expose them through the tool.

The tool accepts only a stable nonsecret `--authorization-reference` interpreted by service. It never reads arbitrary environment variable names, key files, certificate stores, sockets or credential helpers.

## Source and path inputs

Explicit source/release plan/manifest/bundle paths are transport data only. Service/source/bundle owners validate contents. The tool does not inspect Git state, recursively read source, extract archives or follow manifest URLs.

Input/output/config paths follow the exact platform path policy and reject forbidden traversal, symlink/reparse/device/UNC/ADS/cross-root behavior.

## Output

```text
envelope-json: exact canonical service bytes plus one LF
text: faithful bounded release lifecycle projection
artifact: exact service-approved artifact bytes
```

Text preserves every mandatory gate, skipped/`NotEvaluated` suite, reproducibility mismatch, signature/trust state, channel CAS/read-back, support scope, `OutcomeUnknown` and nonclaim.

stdout contains requested output only. stderr contains bounded redacted diagnostics. No progress line/banner on JSON/artifact stdout.

## Progress

When direct/daemon transport supports E7-A progress, the tool may render bounded progress to stderr or a selected machine progress sink. Progress is nonsemantic and cannot replace the final result. Source/provider text cannot create progress controls.

## Cancellation

Signals map to typed service cancellation. Cancellation does not run cleanup, repeat an effect or infer that a build/sign/upload did not occur. Final/`OutcomeUnknown` state remains visible.

## Broken pipe and output failure

Output failure never invokes service again. Durable build/sign/channel/revocation effects remain exact and are later retrieved/reconciled by operation ID.

## Response loss

If using the daemon and the connection is lost, the tool does not silently fall back to in-process execution or resubmit. `wow-release reconcile` uses exact operation/request identity.

## Security hard stops

```text
no arbitrary shell/process/Cargo arguments
no raw environment block
no private key/token/password/cookie/endpoint
no direct GitHub/provider API
no arbitrary HTTP request or upload callback
no source execution or repository hooks
no archive extraction/repacking in tool
no installer/update/source mutation
no latest/newest/force/ignore/skip gates
no retry after OutcomeUnknown
no hidden transport fallback
no telemetry/update/crash upload
```

## CI use

CI may invoke `wow-release` only after implementation and profile freeze. Workflow inputs are exact IDs/profiles/artifacts and protected authorization references resolved outside public payloads. CI logs/artifacts follow redaction/retention policy.

The workflow does not recreate release logic in shell and cannot call channel publish automatically merely because tests pass. Publication remains an explicit separately authorized operation.

## Exit behavior

Exit mapping distinguishes successful requested operation, blocked/partial/`NotEvaluated`, validation invalid/domain failure, `OutcomeUnknown`/internal/output failure, pre-service CLI/input/path failure and cancellation. Exact numeric values freeze with implementation/release compatibility.

## No public distribution

The `wow-release` binary/tool itself is excluded from the default public bundle. If distributed to release operators, it receives its own exact artifact/support/security/signature profile and never grants credentials by inclusion.