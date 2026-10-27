# Addon and load context

Delivery state: [CURRENT_STATUS.md](../../docs/CURRENT_STATUS.md). Project owns
bounded root/file admission, TOC/load selection, exact bytes, plan identity and
source projection.

For Slice 1A:

- replace profile-bound plan identity with `AddonLoadTarget` plus projection scope;
- use `physical_lua`: retain XML records/spans but do not open XML;
- expose a standalone requested-scope limitation projection;
- open roots component-by-component without following links and reuse existing
  inventory/`read_lua_library` admission, including case, UTF-8 and byte limits;
- require each Library root to yield at least one admitted Lua file;
- retain a nonsemantic root token plus root-relative path for every admitted file
  and file-specific failure;
- detect the same admitted file across Main/Library or multiple Libraries. Root
  ancestry alone is allowed when selected files are disjoint; equal relative paths
  in distinct roots are allowed;
- return admitted bytes/authority records to the single Service operation. Project
  does not construct public result DTOs or publication identities.

Later slices add dependency/order, full XML/chunk context, channel, triggers,
bindings and secure/special cases. Never rewrite user source.
