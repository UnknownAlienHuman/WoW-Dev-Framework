# `status` operation

**Status:** normative E0-F state-reporting contract.

`status` answers “what exact framework state is configured and available?” It does not answer “is the project clean?”, “did tests pass?”, or “does this work in the WoW client?”

## 1. Request

```text
StatusRequest
    service_id
    project_id: optional
    detail_level: summary | capabilities
    output_budget
```

E0 configuration contains one exact fixture project. Omitting `project_id` is allowed only because the service configuration declares exactly one default project ID; the result records it explicitly.

No profile/current project is inferred from editor/client/filesystem/Git.

## 2. Operation sequence

```text
validate request/configuration/budget
-> inspect component registry and exact configured identities
-> read current published project snapshot identity (no diagnostic execution)
-> inspect reference/analyzer/rule registry identities and health
-> collect component capability summaries and important blockers
-> collect current/last-known-good/failed-target identities separately
-> collect deferred operations/capabilities
-> assemble, validate, canonicalize ServiceStatusResult
```

`status` need not acquire a full check `ServiceContextLease`, but every identity it reports must be internally consistent and explicitly scoped.

## 3. Required fields

### Service

```text
service ID/implementation/contract version
core/output schema versions
tool versions
service configuration ID/digest
configured budgets/cancellation policy
```

### Project

```text
configured ProjectId
current published ProjectGenerationId / ProjectSnapshot ID/digest
project health
project source/file count and manifest identity
selected ProfileIdentity / ReferenceGenerationId
last-known-good ProjectGenerationId if any
failed candidate/target ProjectGenerationId and failure code if any
```

### Reference

```text
ReferenceView / ReferenceGeneration identity
fixture/release eligibility
profile identity
reference capabilities/coverage/conflicts summary
important failed/partial partitions
```

### Analyzer

```text
accepted upstream pin/probe/config identity
AnalyzerSnapshot ID bound to current project generation
session/snapshot health
main/library workspace health
analyzer capabilities/coverage summary
unclassified/upstream compatibility blockers if any
```

### Rules

```text
RuleRegistry ID/digest
active RuleId/version set
technical severity and rollout policy
fixture policy ID/profile applicability
rule capabilities/known blockers
```

### Deferred

```text
later public operations
inactive components
deferred project/reference/analyzer/rule capabilities
first planned milestone
```

## 4. Component health

```text
Ready
    exact declared state/view available; no mandatory structural blocker

Degraded
    exact coherent state available, but one or more nonmandatory capabilities are partial/failed/not evaluated

Failed
    configured component structurally failed or cannot provide declared identity/view

Unavailable
    configured dependency/snapshot/implementation absent

Deferred
    intentionally inactive for current milestone
```

Health is component contract availability only.

## 5. Prohibited fields/claims

Without an external explicitly identified run record (not part of E0 status), do not emit:

```text
check_passed
tests_passed
project_clean
runtime_verified
safe
working
release_ready
```

Do not infer these from `Ready`, zero cached findings, or absence of a recent check.

## 6. Capability presentation

Summary detail includes:

- capability ID;
- owning component;
- effective status;
- selected important partition/coverage blockers;
- conflict/truncation/staleness indicator;
- current exact generation/snapshot context.

Capabilities must remain narrow. Do not collapse all reference/analyzer/project/rule state into one green/red flag.

## 7. Last-known-good and failed target

Example status shape:

```text
current_published_project_generation: P1
last_known_good_project_generation: P1
failed_target_project_generation: P2
failed_target_error: project_analyzer_update_failed
is_current_for_failed_target: false
```

Do not call P1 “P2 current” or suppress the failed target.

## 8. Deferred operations

E0 status must list:

```text
lookup
search
tree
skeleton
plan
patch_impact
index_repo
runtime_review
lsp
mcp
release_packaging
```

Each record:

```text
operation ID
state = Deferred
first milestone
required inactive components/capabilities
error code on invocation
```

## 9. Warnings

Status warnings are structured and deterministic, for example:

```text
component_degraded
last_known_good_retained_after_failed_target
reference_partition_partial
analyzer_capability_unavailable
unclassified_upstream_diagnostic_family
fixture_not_release_eligible
deferred_operation
```

Warnings do not contain a check status.

## 10. Canonical identity

Status result identity covers:

```text
service configuration ID
all exact reported component/snapshot/generation/registry identities
health states
capability summaries/blocker IDs
deferred records
schema/tool versions
budget policy
status result schema version
```

Exclude:

```text
query time
uptime
process/thread ID
temp path
human render ordering/text
live clock age unless represented as explicit noncanonical telemetry
```

## 11. Status result status

Status operation itself may be:

```text
available
partial
failed
cancelled
```

This is not `check` semantic status.

- `available`: requested status data complete for configured summary.
- `partial`: some optional component/status detail unavailable but core configured identities are coherent.
- `failed`: configuration/registry identity invalid; no coherent status result.
- `cancelled`: cancelled before publication.

## 12. Budgets/cancellation

- status summary must fit deterministic configured output budget;
- detailed capability output over budget returns partial status with explicit truncation, not omitted green summary;
- cancellation publishes no late status result;
- no component mutation/retry/background probing.

## 13. Required operations

```text
validate_status_request
collect_service_component_records
classify_component_health
collect_status_capabilities
collect_last_known_good_and_failed_targets
collect_deferred_operation_records
build_status_warnings
assemble_service_status_result
validate_service_status_result
canonicalize_service_status_result
```

## 14. Required tests

- healthy fixture status exact identities;
- Ready does not create check/test/runtime fields;
- degraded analyzer fact capability reports Degraded/blocker;
- failed target plus retained old snapshot separated;
- missing optional detail -> partial status;
- invalid configuration -> failed, no coherent result;
- deferred operations complete and ordered;
- fixture `release_eligible=false` visible;
- shuffled component/capability order -> identical bytes;
- temp path/time/message wording -> no identity change;
- output truncation explicit;
- cancellation no late result;
- status performs no diagnostics/rules/source IO.

## 15. Hard stops

- no check execution;
- no “pass/clean/safe/working” inference;
- no last-known-good relabel;
- no hidden floating identity;
- no deferred operation marked available;
- no status green boolean replacing exact capabilities;
- no source/client/editor/process mutation;
- no timestamp in canonical digest.
