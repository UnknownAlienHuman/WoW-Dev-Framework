# Platform source replay

`ProjectReplay` retains the exact source/package inputs needed to rebuild a current
Project without reopening the source checkout. It stores raw member bytes, selected
TOC/package plans, Main and Library files, configuration and expected semantic IDs.
It never executes source code or treats serialized receipts as authority.

## Read path

1. Bound and decode one strict DTO.
2. Rebuild source/package/configuration owners from retained bytes.
3. Run the real Emmy/Project producer path.
4. Compare configuration, Project, analyzer and replay identities.
5. Admit the stored Project/graph pair only after those checks succeed.

All current feature combinations use one DTO and one
`wow-project.live-replay` record. XML classification, declarations, signatures,
function-call budgets, hooks and other selections remain typed configuration;
they do not create storage or replay versions. The composed candidate receipt is
a separate record because it is a separate owner.

Replay is bounded by the current file, source-byte and store-record ceilings.
Cancellation is checked during capture, decoding and rebuild. Incompatible derived
stores are recreated from source and user configuration; no migration or archive
reader is maintained.
