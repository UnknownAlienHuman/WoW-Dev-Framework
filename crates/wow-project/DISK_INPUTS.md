# Rooted disk input acquisition

`wow-project::disk` owns read-only file acquisition. Service selects inputs and
passes the resulting typed files to Project/Emmy. Application code does not open
Lua files itself. The separate [TOC reader](../../apps/wow/TOC_INPUT.md) uses this
same file reader for its selected load closure.

## Selection and reading

`ProjectInputDirectory::open` registers an explicit directory. Below it:

- `read_lua_inventory` reads a caller-selected nonempty Main or Library subset;
- `read_lua_library` discovers `.lua` files beneath a configured Library root;
- configuration/artifact readers retain their own finite transport ceilings.

Library discovery sorts relative paths before reading, ignores ordinary non-Lua
files and rejects links/special files, empty output and case-colliding Lua paths.
It is limited to 16,384 directory entries and 32 directory levels. The producer,
Library acquisition and Service Library workspace use `wow-core::library_limits`:
4,096 files, 8 MiB per file, 64 MiB total. Main keeps its existing 1,024 / 1 MiB /
16 MiB bounds. Analyzer-output and replay budgets are separate.

Explicit `ProjectDiskFile` entries may carry an expected source digest/length
pair. Directory discovery needs only a root, not per-file metadata or a generator
report. Both selections use the same UTF-8/path/collision/byte admission and
return `ProjectInputFile` values. `into_workspace_input` transfers Library text
to Emmy without reopening the filesystem.

## Filesystem and error boundary

Reuse the existing `cap-std`/`cap-fs-ext` held-directory and no-follow opens; do not
canonicalize/check a path and then reopen it through an ambient absolute path.
No raw handle, write or process capability is exposed. Cancellation is checked
between directory entries and file-read chunks. Source is never executed.

Errors retain the existing Inventory codes and fixed messages. Failed or
cancelled acquisition returns no partial successful inventory. User files and
configuration are not changed. This captures bytes per file, not an atomic
multi-file snapshot or Git-revision proof; concurrent tree changes may fail.

See [the input guide](../../apps/wow/FILES_INPUT.md) for current Service fields.
Ordinary addon configuration, diagnostic transport and persisted direct reads
remain separate work; adding Library discovery does not complete those workflows.
