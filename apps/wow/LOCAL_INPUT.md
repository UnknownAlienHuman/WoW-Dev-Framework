# Executable local status/check

```text
cargo build -p wow-cli --bin wow
wow status --config project-input.json --format text --detail capabilities
wow check --config project-input.json --project my-addon --format json
wow check --config project-input.json --project my-addon --file project-file:main/init.lua
```

`apps/wow` depends on `wow-service`, JSON output and Ctrl-C handling only. Unknown
commands/options and duplicate scalar options are errors; no daemon, source
scan, stdin source, include/interpolation, shell, apply or source-write mode exists.
`--config` is mandatory and explicit. `--help` and `--version` require no config.

## Input: `wow-service/local-project-input/1`

The configuration is a **materialized input package**, not saved diagnostic output.
It contains these exact fields; unknown fields reject:

| Field | Value |
|---|---|
| `schema` | `wow-service/local-project-input/1` |
| `project_id` | Existing `wow-project::ProjectId` spelling, e.g. `my-addon` |
| `workspace_id` | Exact `workspace:...` identity |
| `source_origin_id` | Exact `project-origin:...` identity |
| `logical_root` | Canonical relative logical root, never a host directory |
| `profile` | Full existing `ProfileIdentity` object |
| `reference_view` | Full existing `ReferenceView`, including its verified self-digest |
| `analyzer` | Declaration below |
| `main_files` | Nonempty explicit source array |
| `library_files` | Nonempty explicit annotation Library array |

Each source record has `path`, `text`, `content_digest` (`sha256:...`) and
`byte_length`. `text` is the exact UTF-8 file content encoded as a JSON string;
length and digest refer to its **decoded UTF-8 bytes**, not the JSON escape spelling.
Paths are logical owner paths; `--file` accepts the resulting `project-file:<path>`
ID, not a host path or glob. Main and Library remain separate analyzer universes.

`analyzer` has `compatibility_report_json` (the original complete report as a JSON
string), `accepted_pin_id`, `configuration_digest`, `contract_id`,
`fixture_contract_id` and `library_contract_id`. The existing analyzer importer
verifies the supplied report; the project owner rejects a different compiled
revision/tree. No compatibility digest, profile, reference record or source is
invented by the CLI. Embedders may instead use `LocalProjectInput::new` with
existing `ProjectInputBundle` and `ReferenceView` owners.

Host file/manifest materialization is not implemented by this checkpoint. An
operator/producer must supply this package explicitly; the application does not
infer it from cwd, editor configuration, addon folders or a client installation.

## Execution

`status` validates input/configuration and reports components without running the
analyzer. Until a check materializes a snapshot, project state is partial and
`current_context` is absent. It does not claim an on-disk published generation.

`check` registers one service request, publishes the supplied project **in memory**
through `ProjectPublisher`, runs the existing Emmy analyzer and E0 rule owner,
and returns the normal service findings/presentation plus exact owner receipts.
No input source or persistent current pointer is written. `current` refers only
to this explicitly supplied one-shot input publication. An exact generation is
checked against its derived target and never substituted. File/rule selection is
service-owned. The complete generic owner report is labeled `whole_project`;
service raw findings and rule evaluation presentation follow the requested scope.
The rule receipt retains the complete E0 provider report for the selected files.

E0 rules remain fixture-profile-only. Repository profiles can obtain actual
analyzer diagnostics, but unsupported WoW rules are explicit `NotEvaluated` and
make the result partial. No fixture semantics are promoted to live-client truth.

## Output and bounds

JSON is service bytes plus one LF. Text preserves identities, component state,
raw findings, display-root count, evaluations/blockers and deferred operations.
Exit codes: `0` available/clean, `1` findings, `2` partial, `3` unavailable/request
failure, `4` internal/output failure, `64` argument/configuration failure,
`130` cancellation. No parser/source diagnostic prose is echoed on failure.

Acquisition reads at most 32 MiB + 1, independent of metadata. Each Main/Library
inventory allows at most 1,024 files, 1 MiB per file and 16 MiB total. JSON output
is capped at 32 MiB before writing. Ctrl-C requests cooperative cancellation;
checks run around project analysis and within the rule owner. The current analyzer
call cannot be preempted mid-call. Once output writing begins, it is never replaced
by a second result. A broken pipe never repeats the service call.

This is executable E0 code, not full E0-F/R0, source-provenance, performance or
supported-release acceptance. No new test suite is part of this checkpoint.
