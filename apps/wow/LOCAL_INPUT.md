# Materialized local check input

`wow status --config <file>` and `wow check --config <file> --project <id>`
currently accept `wow-service/local-project-input/1`. This is a materialized
input format for the existing reader, not the intended ordinary project setup
configuration tracked by #200. The #200 path should resolve root, flavor, TOC
and Library inputs from real project/source data; it should not fabricate a
`ReferenceView` to satisfy this format.

## Input: `wow-service/local-project-input/1`

The inline schema requires
`schema`, `project_id`, `workspace_id`, `source_origin_id`, `logical_root`,
`profile`, `reference_view`, `analyzer`, `main_files` and `library_files`.
Unknown fields reject. The project/workspace/origin identifiers and logical
root are owner values, not host paths. `profile` is a `ProfileIdentity`, and
`reference_view` is a validated `ReferenceView`.

Each Main or Library source record contains `path`, `text`,
`content_digest` and `byte_length`; the digest and length describe the UTF-8
bytes of `text`. Each inventory contains 1–1,024 Lua files, up to 1 MiB per
file and 16 MiB total. Main and Library keep their distinct project roles in
the same existing analysis flow. `--file` selects a logical
`project-file:<path>` ID.

The required `analyzer` fields are `compatibility_report_json`,
`accepted_pin_id`, `configuration_digest`, `contract_id`,
`fixture_contract_id` and `library_contract_id`. The current reader validates
the report against the compiled EmmyLua backend and passes the binding to
`wow-project`. `compatibility_report_json` holds the full report as a JSON string. They are current input requirements; #200 should derive the
needed analyzer binding from actual inputs and the compiled owner, without
inventing a Reference view or asking users to supply fake evidence.

For current disk transports, see [FILES_INPUT.md](FILES_INPUT.md); for the
source-to-Reference/annotation input, see [NATIVE_INPUT.md](NATIVE_INPUT.md).
Both converge on the same project/analyzer composition. Input is read as data;
analysis does not execute or modify source files.

## Execution and output

`status` reports configuration/components without running addon diagnostics;
its project state is partial until analysis produces a view. `check` publishes
the supplied input in memory, runs Project/Emmy and the selected rule owner, and
returns findings. It writes no source or persistent current publication.
Inline/file inputs still use fixture-only WoW rules; other profiles get Emmy
diagnostics with unsupported rules marked `NotEvaluated`/partial. The native
input has its own selected source-backed rules.

JSON output is the Service result plus one LF; text shows findings, locations,
component state and blockers. Exit codes: `0` available/clean, `1` findings,
`2` partial, `3` unavailable/request failure, `4` internal/output failure,
`64` argument/configuration failure, `130` cancellation. `--help`/`--version`
need no config; duplicate scalar or unknown options reject.

Config and output are each limited to 32 MiB. Ctrl-C requests cooperative
cancellation; checks run between files and analysis stages. One synchronous
upstream file diagnostic is not forcibly preemptible. Once output starts, an
output failure or broken pipe never repeats the operation.
