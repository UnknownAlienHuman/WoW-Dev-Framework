# Local files input

```text
cargo build --locked -p wow-cli --bin wow
wow status --config workspace/project.json --format text
wow check --config workspace/project.json --project my-addon --format text
wow check --config workspace/project.json --project my-addon --file project-file:Init.lua
```

`--config` accepts either the existing inline `wow-service/local-project-input/1`
package or `wow-service/local-project-files/1`. The new schema reads declared files
from disk; no source bodies are embedded in configuration. The config's explicitly
selected parent directory is registered once by `wow-project`. All manifest paths
are relative to that retained directory, not cwd. Neither the CLI nor service
opens individual source files. No directory scan, TOC/XML interpretation, glob,
include expansion, environment substitution, URL fetch, source execution or write.

For selected TOC/XML source expansion use the separate [TOC_INPUT.md](TOC_INPUT.md)
schema; the explicit-file schema below does not change.

## File manifest schema

| Field | Value |
|---|---|
| `schema` | `wow-service/local-project-files/1` |
| `project_id`, `workspace_id`, `source_origin_id`, `logical_root` | Same logical identities as [inline input](LOCAL_INPUT.md); `logical_root` is not a disk path |
| `profile` | Pinned file reference to the existing `ProfileIdentity` JSON |
| `reference_view` | Pinned file reference to the existing `ReferenceView` JSON |
| `analyzer` | Same declaration as inline input, but replace `compatibility_report_json` with a pinned `compatibility_report` file reference |
| `main` | `{ "root": "addon", "files": [{ "path": "Init.lua" }, { "path": "UI/Frames.lua" }] }` |
| `library` | `{ "root": "library", "files": [{ "path": "api.lua" }] }` |

A **pinned file reference** has `path`, `content_digest` (`sha256:` plus 64 lowercase
hex digits) and `byte_length`. Hash and length cover the exact file bytes, including
whitespace and line endings. All three metadata artifacts must be pinned. Their
existing owner validators still check schema, self-digest, profile/generation and
compiled analyzer compatibility. No report, profile, source revision or reference
coverage is invented merely to make the CLI run.

Main and Library entries may also carry `content_digest` and `byte_length` as a
pair. Without them the reader captures current file bytes and computes their exact
identity; edits are therefore picked up on the next invocation without manually
rebuilding JSON. With them, changed content fails before project publication.
`root` is `.` or a canonical subdirectory relative to the config's directory.
`path` is relative to that inventory root. Logical file IDs exclude both physical
root paths. Moving an unchanged workspace does not itself change source identities.

The complete file list must be explicit and nonempty for each role. Only `.lua`
files enter this E0 acquisition path. Omitted files are outside the supplied
universe, not proven absent from an addon. Full TOC/XML/load completeness remains
unavailable. Main and Library are independently admitted and never merged.

## Acquisition and limits

`wow-project::disk::ProjectInputDirectory` owns a private `cap_std::fs::Dir`.
The only ambient open registers the caller-selected config directory. Every
subdirectory is opened by handle with `open_dir_nofollow`; every final file is
opened read-only with no-follow and checked as a regular file. Unix opens are
nonblocking to reject a racing FIFO replacement. Absolute/traversal paths,
Windows device names, alternate streams, backslashes, noncanonical spelling and
case-fold collisions within each declared inventory are rejected.

Limits: config 32 MiB; each metadata artifact 8 MiB; each inventory 1,024 files,
1 MiB per source, 16 MiB cumulative. Acquisition checks the stream itself, not
just metadata, and polls cancellation between chunks/files. Detected size or
modification-time changes during one read reject without retry. Expected digests
provide stronger frozen-input checks. File and directory handles close before
analysis begins. Errors use fixed messages, not OS error strings or host paths.

This captures exact **read bytes**, not an atomic filesystem-wide snapshot or a
Git-revision attestation. Concurrent in-place edits can escape timestamp-based
change detection; pin every file or use a stable checkout for frozen provenance.
The caller-selected directory is an explicit capability: its own resolution and
hard-linked regular files are not a sandbox against a hostile directory owner.
Network filesystem calls and analyzer calls are not forcibly preemptible.

Both transports use the same project/analyzer/reference/rule composition and
existing output/exit semantics. The fixture-only WoW rule restriction is unchanged;
unsupported live profiles remain `NotEvaluated`. No test matrix or fixture is
changed by this implementation; full R0 and host-race acceptance remain separate.
