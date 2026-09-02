# Conformance command contract

**Status:** normative command surface to implement and freeze. No command below is currently available because the Rust workspace does not exist.

## Principles

- Commands are supported only after their exact tool/version/target/profile and output schema are frozen.
- Every command returns a nonzero result when a required check is failed, skipped, unavailable or `NotEvaluated` according to its command contract.
- Validation commands never rewrite fixtures, checksums, schemas or generated files.
- Generation/update commands are explicit and separate from validation.
- CI eventually invokes these exact commands; CI does not duplicate their logic.
- Output has a machine JSON mode with stable schema and a faithful text mode.

## Toolchain bootstrap

Freeze exact versions before use:

```text
rustc --version --verbose
cargo --version
rustfmt --version
cargo clippy --version
cargo nextest --version when nextest is required
```

The release profile records exact output and target components. Floating installer/toolchain behavior is not a conformance command.

## Repository contract validation

The implementation must provide one owned validator command, exposed through a thin service/tool route or an exact development binary, for these logical operations:

```text
contract check
manifest check
dependency-graph check
fixture check
checksum check
documentation-links check
forbidden-files check
```

Required final user-facing development command shape:

```text
cargo run --locked -p wow-release -- contract check --output-mode envelope-json
```

If the release tool is not yet implemented, package-owned Rust tests may temporarily provide the same checks; the manifest records the exact transition. The final V1 path uses one frozen validator implementation.

### `contract check`

Validate:

```text
strict JSON/schema and duplicate-key rejection
unique contract IDs/work-package ownership
public operation/tag/status/error closure
normative file existence
implementation/freeze state consistency
application/tool dependency rules
```

### `manifest check`

Validate root/crate/application/tool entries, documentation and implementation frontiers, active contracts/seams, launch sequence, target support state and no duplicate/missing package.

### `dependency-graph check`

Validate Cargo metadata against the normative dependency graph, active work-package slice and application/tool service-only rule. Dev/build dependencies receive their own classified report.

### `fixture check`

Validate fixture schema, unique case IDs, case coverage by owner test matrix, expected error codes, exact referenced profiles/artifacts and immutable bytes.

### `checksum check`

Verify every non-null member/bundle/checksum manifest. Before implementation freeze, required nulls are reported as blocking, not silently ignored. Tests do not populate them.

### `documentation-links check`

Validate internal relative Markdown routes, anchors where supported, code-fence balance, generated index closure and no stale next/frontier claims.

### `forbidden-files check`

Detect secrets, private keys/tokens, temporary probes/recovery manifests, unowned generated/build outputs, forbidden workflows, raw databases/source archives and placeholder Rust/traits/todos under package policy.

## Standard Rust checks

Once the corresponding workspace members are active:

```text
cargo fmt --all -- --check
cargo clippy --locked --workspace --all-targets --all-features -- -D warnings
cargo test --locked --workspace --all-features
cargo nextest run --locked --workspace --all-features
cargo doc --locked --workspace --all-features --no-deps
```

Exact feature/profile exclusions, if any, are documented rather than hiding failures. `cargo test` and nextest responsibilities are not called equivalent until their suite/runner semantics are frozen.

## Package acceptance commands

Each work package exposes a deterministic package acceptance suite. Required logical names:

```text
e0-a-acceptance ... e7-b-acceptance
```

Final command shape:

```text
cargo nextest run --locked -p <owner-package> --test <work-package-acceptance-target>
```

or an equivalent exact package-owned command recorded in the package contract. Every test matrix ID maps to an executable case/report.

## R0 first-runnable conformance

Required commands after I0-F:

```text
cargo build --locked -p wow
cargo test --locked -p wow-core -p wow-reference -p wow-emmy -p wow-project -p wow-rules -p wow-service -p wow

cargo run --locked -p wow -- status --output-mode envelope-json
cargo run --locked -p wow -- check --request <E0 fixture> --output-mode envelope-json
cargo run --locked -p wow -- check --request <clean fixture>
cargo run --locked -p wow -- check --request <finding fixture>
cargo run --locked -p wow -- check --request <partial fixture>
cargo run --locked -p wow -- check --request <conflict fixture>
cargo run --locked -p wow -- check --request <NotEvaluated fixture>
```

Additionally run malformed input, cancellation, deterministic repeated bytes, one/parallel worker, broken pipe, output-path failure and resource-limit cases.

R0 report must bind exact source/toolchain/profile/fixture/checksum IDs.

## Reference Pack conformance

After I1-D:

```text
cargo run --locked -p wow-reference-builder -- build --request <exact build request>
cargo run --locked -p wow-reference-builder -- validate --reference-pack <exact ID/path>
cargo run --locked -p wow-reference-builder -- rebuild-compare --reference-pack <exact ID> --request <same inputs>
```

Run restricted evaluator/adversarial source, corrections, unknown fields, annotations/parity, license, deterministic bytes, store crash/recovery/read-back and negative-authority coverage suites.

## Project/graph/context conformance

After I2/I3:

```text
cargo run --locked -p wow -- project validate <exact request>
cargo run --locked -p wow -- check <real admitted addon request>
cargo run --locked -p wow -- context map <exact request>
cargo run --locked -p wow -- context build <exact request>
cargo run --locked -p wow -- context validate <exact artifact>
```

Run TOC/XML/load variants, nonexecution, incremental invalidation, graph partitions/axes, Blizzard UI universe separation, L0/L1/L2 budgets, privacy/license, recovery and real addon corpus reports.

## Search/lineage/impact conformance

After I4:

```text
cargo run --locked -p wow -- search <exact request>
cargo run --locked -p wow -- lineage <exact request>
cargo run --locked -p wow -- migration validate <exact request>
cargo run --locked -p wow -- impact <exact request>
cargo run --locked -p wow -- search-to-context <exact request>
```

Run lane/ranking explanations, candidate ceilings, exact aliases/transitions, cross-generation ambiguity, negative authority, continuation and static-impact resource/cycle suites.

## E5 governance conformance

After I5:

```text
cargo run --locked -p wow -- calibration ...
cargo run --locked -p wow -- core-pack ...
```

The exact commands are defined in `apps/wow/e5` and `apps/wow/e5c`. Required reports include real admitted corpora, independent labels/reviews, holdout access/audit/consumption, mutations/metrics, submission revalidation, signatures, inactive publication/read-back, canary, rollout, activation/LKG, rollback/revocation/partition closure and response-loss at every effect.

No E5 command is in the default read-only LSP/MCP exposure.

## E6 external-provider conformance

When enabled:

```text
cargo run --locked -p wow -- external-candidate provider validate <exact request>
cargo run --locked -p wow -- external-candidate query submit <exact request>
cargo run --locked -p wow -- external-candidate result validate <exact ID>
cargo run --locked -p wow -- external-candidate mapping resolve <exact request>
cargo run --locked -p wow -- external-candidate selection record <exact request>
cargo run --locked -p wow -- external-candidate context build <exact request>
```

Run stable/mutable/opaque state, Candidate ceiling, provider-local score, zero-result, mapping ambiguity/negative authority, explicit selection, sidecar separation, credentials, outage/degradation and provider-on/off benefit benchmarks.

E6 may remain disabled without blocking local product conformance.

## E7-A host conformance

Required product invocations:

```text
cargo run --locked -p wow -- daemon serve --config <exact test config>
cargo run --locked -p wow -- lsp serve --stdio --config <exact test config>
cargo run --locked -p wow -- mcp serve --stdio --config <exact test config>
```

These are driven by deterministic protocol harnesses rather than manual editor observation.

### Daemon harness

Test exact handshake/framing/method registry, sessions, concurrent generation CAS, progress, streams, backpressure, reconnect, cancellation, crash/startup recovery, security and graceful shutdown on the selected platform endpoint.

### LSP harness

Test initialization/capability intersection, UTF-16 positions, full and optional incremental sync, stale versions, diagnostics, completion, signature help, hover, definition/references, symbols, call hierarchy, guarded code actions, cancellation/progress, malformed messages, stdout purity and shutdown.

### MCP harness

Test initialization, static read-only tools/resources, exact schemas, one-tool-one-service mapping, no generic/admin/model capabilities, roots-as-candidates, structured result fidelity, streams, cancellation, hostile payloads and shutdown.

### Cross-transport equivalence

For equivalent exact service requests/session snapshots, compare canonical service envelopes obtained through direct CLI, daemon, LSP and MCP adapters. Protocol projections may differ only in reviewed nonsemantic framing/mapping fields.

## E7-B release conformance

Internal exact pipeline commands:

```text
cargo run --locked -p wow-release -- source validate --source-request <file>
cargo run --locked -p wow-release -- plan validate --release-plan <file>
cargo run --locked -p wow-release -- build submit --release-plan <ID> --operation-id <ID>
cargo run --locked -p wow-release -- build get --build <ID>
cargo run --locked -p wow-release -- rebuild compare --build <ID> --build <ID>
cargo run --locked -p wow-release -- artifact validate --artifact-set <ID>
cargo run --locked -p wow-release -- sbom build --artifact-set <ID> --operation-id <ID>
cargo run --locked -p wow-release -- provenance build --artifact-set <ID> --operation-id <ID>
cargo run --locked -p wow-release -- sign request <exact fields>
cargo run --locked -p wow-release -- sign validate --signature <ID>
cargo run --locked -p wow-release -- bundle build --bundle-request <file> --operation-id <ID>
cargo run --locked -p wow-release -- bundle validate --bundle <ID/path>
cargo run --locked -p wow-release -- support validate --support-matrix <ID/path>
cargo run --locked -p wow-release -- candidate validate --candidate <ID/path>
cargo run --locked -p wow-release -- channel prepare <exact fields>
cargo run --locked -p wow-release -- channel publish <exact plan/guard/auth>
```

Public product checks:

```text
wow version
wow release status
wow release verify bundle --input <bundle>
wow installation validate
wow update check --channel <ID> --operation-id <ID>
wow update plan <exact fields>
wow update apply <exact plan/current guard>
wow update reconcile <exact operation/request>
wow update rollback <exact current/target/guard>
```

## Release rehearsal command

The final candidate command must provide one exact orchestration entry that validates an already prepared candidate or executes a separately authorized fixed rehearsal plan without arbitrary steps. Logical shape:

```text
wow-release candidate validate --candidate <ID>
```

The release record references the individual build/evidence/sign/bundle/platform/install/update/rollback reports; the client tool does not hide them behind a shell script.

## Windows target conformance

For `x86_64-pc-windows-msvc`, run on clean supported Windows VM/machine profiles:

```text
portable bundle verification
new install
version/status/check
local daemon handshake/session/reconnect
LSP and MCP harnesses
real admitted addon analysis
update from exact prior candidate
crash/failure injection around helper and migrations
rollback to exact LKR
revocation/update-manifest behavior
uninstall with user-data retention
```

Also test Unicode/spaces/long paths, ACLs, reparse points, devices/UNC/ADS, file locks/antivirus interaction, console/PowerShell/cmd, non-admin/elevation policy and abrupt termination.

## Benchmarks

Each benchmark command emits exact machine-readable input/hardware-class/profile/results and compares to frozen thresholds. Required groups:

```text
cold/warm startup
Reference Pack open/build
small/medium/large addon index and incremental overlay
check/search/context/editor requests
memory/disk/CPU and store growth
LSP/MCP/daemon latency/backpressure
external provider on/off when enabled
unsigned build/reproducibility/bundle time
install/update/rollback time and disk overhead
```

No benchmark threshold is populated from estimates.

## Fuzzing/property tests

Where implemented, exact commands cover parsers/framing/JSON/schema/TOC/XML/path/archive/manifest/coordinate/migration state machines. Fuzzing absence for a required parser is explicit until the corresponding gate is selected.

## CI mapping

Only after all commands exist and run locally:

```text
PR: contract/manifest/fixture/checksum, fmt, clippy, unit/integration/mutation, Windows build
nightly or scheduled only if explicitly owned: extended fuzz/bench/real corpus
manual release workflow: exact E7-B candidate and channel operations
```

The workflow calls these commands; no release logic, generic shell upload or unconditional publish is embedded in YAML.

## Current command status

```text
all commands in this document: specified, not implemented
all executable test/build/release evidence: NotEvaluated
```

The first command to make real is the E0-A package test/contract validator, followed by the R0 `wow status` and `wow check` path.