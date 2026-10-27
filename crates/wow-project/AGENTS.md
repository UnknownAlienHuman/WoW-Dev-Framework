# Addon context

Follow root [AGENTS.md](../../AGENTS.md), current status, the active queue slice
and the relevant implementation route.

Project owns root/TOC/source selection, exact bytes, plan identity and source
projection. For Slice 1A:

1. replace profile-bound plan identity/validation with `AddonLoadTarget` plus
   projection scope; do not fabricate `ProfileIdentity` or retain its digest;
2. parse metadata/conditions, capture direct TOC Lua and retain but do not open
   XML in `physical_lua` scope;
3. return exact root-relative files plus a plan whose standalone mapper can
   distinguish required blockers from optional/nonblocking observations;
4. validate Main files against the same plan before snapshot construction.

Later slices add dependency/order context, mapped XML/chunk context, then channel,
triggers, bindings and secure/special cases. Reuse Emmy for Lua semantics. An
ordinary check must not require Project publication, replay hydration,
fingerprints or proof graph.
