# Explicit disk input acquisition

The E0-D source-acquisition adapter is `wow-project::disk`, separate from the
immutable source registry. It does not extend E0 into scanning or TOC/XML parsing.

`ProjectInputDirectory::open` registers the **explicitly selected** directory.
The private directory capability only exposes bounded configuration/artifact reads
and `read_lua_inventory`. It cannot expose an OS handle, write, enumerate, spawn or
resolve a moving source selector. The service coordinates this port and passes
only retained typed inputs to the existing project/analyzer owners. Application
code depends only on `wow-service`.

`ProjectDiskFile` names a canonical portable relative path. An optional expected
source digest/length pair pins source bytes; JSON metadata artifacts require that
pair. Paths and inventory collisions are admitted before source acquisition.
`read_lua_inventory` assigns the caller's explicit Main/Library role and returns
`ProjectInputFile` values with digests computed by the existing source-identity
implementation. `into_workspace_input` transfers an admitted Library file to
Emmy without reopening the filesystem or copying its source body.

## Dependency decision

Add `cap-std` and `cap-fs-ext` major 4 to this owner only, with actual resolved
versions retained in Cargo.lock. Use directory-relative no-follow opens rather
than canonicalize/check-prefix followed by an ambient open, which races path
replacement. These dependencies implement the platform adapter; framework code
remains `forbid(unsafe_code)`. No new framework dependency edge is introduced.
Only `fs::Dir` and read-only operations are used, not dependency network/write APIs.
Full supply-chain/release acceptance remains separate from a successful build.

## Error and identity boundary

Acquisition failures use Inventory phase and fixed messages: `SourceReadFailed`,
`SourceBudgetExceeded`, `SourceChangedDuringRead`, `SourceReadCancelled`, plus the
existing missing-file/path/encoding/digest/length/case-collision codes. Failed or
cancelled loading returns no partial inventory. No host path enters an error,
configuration identity, file ID, project generation or source registry.

This is a read-byte snapshot, not atomic multi-file or Git-revision provenance.
Read caps, no-follow behavior, cancellation and exact format are documented in
[the application schema](../../apps/wow/FILES_INPUT.md). Persistent publication,
filesystem watching, complete-addon enumeration and source update remain outside
this port. Existing embedded input APIs remain available without host access.
