# `wow-reference-builder`

Follow root `AGENTS.md`, `docs/ARCHITECTURE.md`, and
`docs/IMPLEMENTATION.md#native-annotation-generation`.

This app owns only command parsing, bounded local Git reads and writing a new
output directory. `wow-reference` owns source facts; `wow-annotations` owns
projection and rendering. Keep the single `generate` workflow direct.

Do not add pack manifests, operation journals, signatures, promotion states,
backup/restore, installers, compatibility commands or another parser. Preserve
explicit revision/TOC/environment selection, non-execution of donor Lua,
cancellation, finite input/output bounds, partial-result reporting and new-only
output.
