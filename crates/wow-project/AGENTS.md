# Addon context

Follow root [AGENTS.md](../../AGENTS.md),
[current status](../../docs/CURRENT_STATUS.md), the active
[queue slice](../../docs/REFACTOR_QUEUE.md#1-first-usable-ordinary-addon-check)
and the relevant [implementation route](../../docs/IMPLEMENTATION.md).

Project owns root/TOC/source selection, bounded bytes, exact user-path projection
and the load/context facts required by the active slice. Slice 1A extracts a
profile-free `AddonLoadTarget` from the existing reader, supplies explicit flavor
with no target Interface/context, retains TOC Interface as source metadata, and
selects physical Lua. It must not fabricate `ProfileIdentity` or its
publication-era digests. Classify metadata by effect before completeness. Later
slices add dependency/order context, XML/chunk mapping, then
channel/triggers/bindings/secure special cases.

Preserve exact mapped ranges and source-backed unknowns; reuse Emmy for Lua
semantics. Missing later-slice behavior limits only affected selected input or
checks. An ordinary check must not require Project publication, replay hydration,
fingerprints or proof graph.
