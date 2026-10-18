# Current architecture decisions

## ADR-058 — Reduce the product to its development workflows

**Accepted by explicit owner direction, 2026-10-18.**
[ARCHITECTURE.md](ARCHITECTURE.md) defines the complete current scope. This
supersedes the earlier E0–E7 completion plan and conflicting obligations in
package contracts, manifests, local decisions and Issues. In particular, old
internal compatibility, cryptographic result/pack governance, external providers,
Wasm algorithm plugins, lineage promotion and release/update infrastructure are
removed from the product. Their old acceptance work is not unfinished mandatory
scope. Source/configuration safety and normal protocol behavior remain required.

Previous ADR text is available in [Git history at the preceding review](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/5aa5f6bc8421eece38309b2fffe85d7f60377f22/docs/DECISIONS.md).
Do not copy it into an archive or use it to restore rejected features. The
current decisions below are the active register; superseded rules are not
reactivated merely because some implementation still contains them.

## Engine and annotation pipeline

Keep upstream EmmyLua as the one Lua parser/analyzer behind the existing narrow
adapter (the useful core of ADR-001/003/011). Keep Ketho as the primary donor for
native Rust annotation generation (ADR-004). Preserve semantic type details and
donor notices. Blizzard source selected for the requested flavor/revision
supplies current API facts. No VS Code, Node, Lua or Python runtime is required.
Use upstream language-server features instead of cloning their implementation.

## Local data

SQLite and FTS5 remain the local substrate (the useful core of ADR-015). Indexes
and generated libraries can be rebuilt from source/configuration. Keep one
current schema and remove previous internal variants and migrations with their
callers. No compatibility guarantee is implied by a pre-release fixture or old
cache. If rebuild inputs are absent, return an actionable error; preserve user
files. Transactions and ordinary file/resource lifetimes replace generic
publication, effect journals, leases and backup/rollback machinery.

## Small modules and usable interfaces

A component needs a concrete current responsibility (ADR-022). Crate count and
service-only forwarding are not architectural goals. Reuse the existing owner;
merge/delegate away redundant layers. Serialize once at the output boundary.
CLI provides complete local workflows; stdio LSP/MCP reuse them and follow their
standards. Do not create a profile for every feature combination. Unsupported
features are not advertised. General Lua intelligence remains EmmyLua's job.

## Truth and source safety

Keep the selected source/flavor and accurate source locations. Do not execute
indexed repositories, infer runtime safety from static findings, suppress
unknowns as clean results, or leak private source/context. Use bounded input,
parameterized SQL and atomic writes. These checks do not require signed results,
cryptographic proof graphs, or a new policy framework.

## Delivery and future changes

Use a normal Cargo build, the existing CI, focused relevant tests and a short
manual workflow when needed. Ship ordinary artifacts with notices and clear
build/platform information. No custom release executor or updater is required.
New features need a concrete developer task, a reason existing code cannot solve
it, and a named old path to replace when duplicating behavior. Update the current
architecture once; do not grow a second plan or an ADR for every helper.
