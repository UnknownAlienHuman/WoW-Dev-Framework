# Blizzard source manifest

A moving selector is resolved once per operation. Its exact SHA identifies only
the bytes inspected in that operation, not a permanent client dependency.
The native maintenance inventory reads a materialized local Git checkout:

```sh
cargo xtask manifest "$WOW_UI_SOURCE_DIR" HEAD live .wow-dev/source-manifest.json
cargo xtask verify-manifest .wow-dev/source-manifest.json "$WOW_UI_SOURCE_DIR" origin/live
```

Create the destination parent first; the output file must not exist. The producer
includes Lua, generated API docs, XML, TOC, XSD and `version.txt`, in bytewise path
order. Each record retains canonical path, semantic class, length, Git object
algorithm/ID and independent SHA-256. Git object hashing supports both SHA-1 and
SHA-256 without a compiled client version. No worktree bytes, export substitutions,
source execution, local paths, credentials or wall-clock time enter the manifest.

The v1 fixed default selection is retained. Optional legacy extension/limit flags
are not aliases for the new positional command. Bounds are documented in
[xtask](../tools/xtask/README.md). Nonregular or unsafe source paths are rejected.
Verification rebuilds from the exact revision and compares all fields and digest;
when a selected current local ref differs, it returns 3 instead of rewriting
historical evidence. A local ref check does not establish network freshness.

Use `cargo xtask check-source <checkout> <branch>` to compare an explicit public
HTTPS origin. It is read-only and reports unverified freshness on network failure.
Managed cloning/automatic updating and GitHub-only acquisition are not implemented
by these commands. A local clone is preferred; resolve a new revision for a new
operation, never mix files from different source revisions.

An inventory proves selected byte identity, not semantic completeness or runtime
behavior. The native Ketho generator consumes an exact TOC/revision independently
and retains source hashes in its report. The source workflow supplies the same
resolved revision to both operations and does not claim full ReferenceView
persistence or native XML/TOC topology generation.
