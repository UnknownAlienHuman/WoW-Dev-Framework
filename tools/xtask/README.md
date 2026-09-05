# Native repository maintenance

`xtask` is an internal development utility, not the public `wow` application or
an alternate analysis/service owner. It has no framework crate dependencies and
uses the existing dependency closure (`serde_json`, `sha2`) plus Git. It is not
included in a supported product bundle. No external interpreter is invoked.

## Commands

```sh
cargo xtask check
cargo xtask sync-skill --check
cargo xtask sync-skill --write
cargo xtask check-source /path/to/checkout live
cargo xtask manifest /path/to/wow-ui-source HEAD live /path/to/new-manifest.json
cargo xtask verify-manifest /path/to/manifest.json /path/to/wow-ui-source origin/live
cargo xtask verify-library /path/to/native-output --require-input-complete
```

`check` and `sync-skill` accept `--root DIR`; otherwise the framework source root
is used. Check is read-only. It scans distributable tracked/untracked files for
forbidden interpreter assets/invocations, validates JSON and verifies identical
skill discovery copies. It does not certify every planned conformance gate and
is not a defense against intentionally obfuscated programs.

Skill writes are explicit, preflight all paths, reject symlinks, use a single
writer lock and same-directory replacement, then read back. Three target files
are not one crash-atomic transaction; a partial failure is an error and is repaired
by a subsequent explicit sync. Tests/checks never rewrite expected fixtures.

`check-source` only checks an explicit checkout and public HTTPS origin. It
reports both Git revisions, branch and dirty state, never prints the remote URL,
and never fetches/resets/stashes/switches or changes the checkout. A different SHA
is not automatically classified as behind: review/fetch and offer a safe update.
Private/SSH authentication and managed clone/auto-update remain separate work.
The same command can inspect a local EmmyLua checkout by passing its branch.

`manifest` inventories one exact Git snapshot using raw blobs, per-repository
Git object hashing and independent SHA-256. Export attributes and dirty files do
not affect it. Default selected extensions are Lua, XML, TOC and XSD plus required
`version.txt`. Nonregular entries reject even outside the selected extensions.
Bounds: 200,000 entries, 32 MiB per file, 256 MiB selected bytes, 64 MiB manifest.
Verification independently rebuilds the inventory; unknown/changed data fails.
No host paths, URLs, credentials, timestamps or permanent build IDs enter it.

`verify-library` checks the native build/report schema, selected/admitted/failed
source counts, file inventory, final bytes/hashes and mapping ranges. With
`--require-input-complete`, any source admission failure is an error. Projection
issues still return partial. Source hashes in sidecars identify recorded evidence;
this validator alone does not independently prove their upstream provenance or
language-server semantics. The current-source workflow runs the actual loader.

Exit status: 0 verified; 2 invalid/failed; 3 drift, differing/stale revision or
partial projection; 4 network freshness unavailable (`check-source` only).
Verification is read-only. Manifest publication is new-only, not crash-durable
store publication. Git subprocesses have output bounds and deadlines; no shell
runner, repository hooks or lazy fetching is used for local object reads.

## Retirement boundary

The interpreter-based source producers and tests were removed by explicit owner
direction, not renamed, archived, wrapped or hidden behind Rust. Existing v1 JSON
importers remain supported as compatibility readers. Full native topology
production and the former upstream source-manager/probe command family have not
been ported wholesale and must not be advertised as implemented.
