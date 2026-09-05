# Implementation status and update policy

This is the current implementation ledger. Older bootstrap schedules describe
planned owner contracts, not an instruction to recreate the workspace or retain
one compiler/client/upstream revision indefinitely.

## Implemented executable scope

The active Cargo workspace contains `wow-core` and `wow-reference` only.

- `wow-core`: deterministic identity, evidence, coverage and result primitives.
- `wow-reference`: deterministic reference view; generated API and UI topology
  JSON validation/import; source-bound API/topology bundle and development CLIs.
- Python source tools: explicit local Git snapshot inventory; declarative API
  documentation and XML/TOC topology producers and verifiers.
- Canonical WoW development skill and byte-identical host discovery copies.
- CI: current stable Rust, locked and updated compatible dependency resolution,
  Python tests and a Python-to-Rust executable source-bridge regression.

The integration test creates synthetic Git SHA-1 and SHA-256 repositories. It
checks actual Python producer output in the compiled Rust CLI, including null,
negative and exponent values, optional child collections, export attributes,
dirty worktrees, stale source selection, digest tampering and repeat publication.
Fixtures are tests, not evidence that an installed WoW client was exercised.

`wow-reference-source materialize` is no-clobber and idempotent for identical
bytes. It stages a file in the destination directory and publishes by an atomic
hard link. Filesystems without hard-link support return an error; no unsafe
replace fallback is used. Directory crash durability is not claimed.

## Update policy

There is no repository toolchain override. CI installs current stable Rust.
Manifest dependency requirements allow compatible SemVer updates. `Cargo.lock`
records one dependency resolution and is intentionally committed; `cargo update`
and the rolling CI lane evaluate newer compatible versions. Major API changes
still require an adapter change and tests. This is not a permanent version pin.

Moving Blizzard source selectors are resolved for each operation. Exact revision,
version, file hashes and producer identity describe that operation only. The
scheduled source-bundle job clones current Gethe/live, then reads a coherent
snapshot. It does not replace the need to select the user's requested flavor.

The current manifest producer reads Git objects directly, not worktree contents
or `git archive`, so export-ignore/export-subst cannot change the observed corpus.
Raw source-wire JSON is distinct from core identity JSON. The source-wire profile
preserves the Python producer's numeric lexemes; it does not assert mathematical
normalization across arbitrary JSON producers.

## Optional context

No private knowledge provider is a build or runtime dependency. No provider
endpoint, token or document corpus belongs in the public tree. The generic
operator-only retrieval bridge, managed source auto-update CLI and GitHub-only
manifest acquisition are **not implemented in this workspace**. The current
skill describes safe manual research without pretending those commands exist.

Removing identifiers from the current tree does not remove historical commits,
old clones, caches or third-party copies. No history rewrite is part of this
repair. Sanitizing links alone cannot guarantee that prose is non-attributable.

## Not complete

I0-A/I0-B have executable slices, not certification of every planned owner
acceptance/property/resource gate. Persistent channel publication is not proven.
The upstream version-check script is not an EmmyLua analyzer adapter. `wow-emmy`
is not an active workspace member. The public `wow` binary, service, project,
rules, persistence, graph, search, provider transport and release lifecycle
remain subsequent implementation work. R0 and all platform/client/install/signing
release gates remain incomplete. No WoW runtime or supported-release claim.

## Next bounded task

Implement the real I0-C analyzer adapter against an updateable upstream dependency,
with source-identity and compatibility probes. Separately implement the generic
managed source updater and optional operator context port. Keep new implementation
behind tested owner boundaries; do not activate skeleton crates to inflate status.
