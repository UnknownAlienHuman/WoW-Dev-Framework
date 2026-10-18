# Explicit files input

`wow-service/local-project-files/1` is the current disk manifest for explicit
Main and Library files. It is a reader format, not the ordinary project setup
configuration tracked by #200. The reader opens only declared files relative
to the selected configuration directory; it does not scan directories, execute
source or write files. For the intended setup, #200 should resolve real project
and library inputs directly.

## File manifest schema

The schema requires `schema`, `project_id`, `workspace_id`, `source_origin_id`,
`logical_root`, `profile`, `reference_view`, `analyzer`, `main` and `library`.
The identity fields, profile, ReferenceView and analyzer declaration have the
meanings described for [materialized input](LOCAL_INPUT.md#input-wow-servicelocal-project-input1).
For this disk form, `profile` and `reference_view` are pinned JSON artifacts;
`analyzer` uses a pinned `compatibility_report` file plus the five binding
identifiers/digest described there.

`main` in `local-project-files/1` has `root` and a nonempty `files` list.
Selecting `toc` instead requires the separate [TOC schema](TOC_INPUT.md),
which also permits `load_context`; the files schema accepts neither option.
`library` has `root` and a nonempty list of declared Lua `files`.
Main and Library retain distinct project roles in the project/analyzer flow.

The config's parent directory is the base for physical paths. Inventory roots
and file paths are canonical relative paths; file paths are relative to their
inventory root. Source entries may include `content_digest` and `byte_length`
together to reject changed bytes. Each entry has `path`; a pinned entry adds
`content_digest` (`sha256:` plus 64 lowercase hex digits) and `byte_length` for
the exact file bytes. Without the pair, the reader captures current bytes and
their identity. Metadata artifacts (`profile`, `reference_view` and
`compatibility_report`) require both fields.

The reader admits 1–1,024 files per inventory, 1 MiB per file and 16 MiB total;
metadata artifacts are limited to 8 MiB and the config to 32 MiB. Only Lua
source is admitted for this explicit-file check. Omitted files are outside the
supplied input, not evidence of absence from the project. Paths are read with
the existing bounded, no-follow file reader. This route does not establish a
filesystem-wide snapshot or Git provenance. Command behavior and exit codes
are in [local checking](LOCAL_INPUT.md#execution-and-output).

`wow-service/local-project-toc/1` changes only the Main selection, while
`wow-service/local-primary-callables-files/1` adds the separately documented
[Primary callable recipes](PRIMARY_CALLABLE_INPUT.md). The selected TOC and
Primary callable variants are distinct current reader formats; #200 is not
required to preserve this manifest shape.
