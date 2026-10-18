# Product architecture

**Current owner decision: 2026-10-18.** This scope replaces the previous E0–E7
platform plan, including its mandatory compatibility, governance and release
lifecycles. [DECISIONS.md](DECISIONS.md) records the supersession. Existing code
still contains parts being removed; this is the target, not a claim that the
refactor is finished. Take one task from [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md).

## 1. The product

The primary engine is **EmmyLua**. Port Ketho's source loading, annotation
normalization and generation to Rust, make the result usable without VS Code,
and add practical search/navigation over Blizzard UI and addon source.

The product is useful when these workflows work:

1. Select Blizzard source and flavor → generate an ordinary `.lua` annotation
   library → load it in EmmyLua or another compatible consumer.
2. Select an addon root and TOC → use that library with EmmyLua → get diagnostics
   and source locations without hand-built evidence JSON or editor extensions.
3. Index a Blizzard UI checkout → find APIs, symbols, text and relevant TOC/XML
   relationships → read a small, located source excerpt or Markdown context.

CLI is the primary complete entry point. Stdio LSP/MCP may expose the same useful
operations. General Lua editor features come from upstream EmmyLua; extending
our WoW integration does not require reimplementing its language server.

## 2. Reuse, then simplify

| Responsibility | Existing code to reuse | Target boundary |
| --- | --- | --- |
| Lua syntax and analysis | `wow-emmy`, upstream `emmylua_parser` / `emmylua_code_analysis` | One analysis per input set; incremental updates and library roots. |
| Blizzard metadata and Ketho behavior | `wow-reference/src/native*`, `wow-annotations/src/{native,ketho,aliases,literals}*` | Restricted source reading → typed data → one native emitter. |
| Addon and Blizzard source | `wow-project` | Root/flavor/TOC, Lua/XML/load facts and correct file spans. |
| Search and storage | `wow-search`, `wow-store`, existing SQLite/FTS5 | One local index with direct tables; query without reparsing source. |
| Useful WoW relationships/diagnostics | `wow-graph`, `wow-recognizers`, `wow-rules` | Only facts consumed by search, navigation or actionable diagnostics. |
| Context and frontends | `wow-context`, `wow-service`, `apps/wow` | Small located results; plain CLI/stdio protocol adapters. |

This is a responsibility map, not a required number of crates. Keep algorithms
in their existing cohesive owner; merge a layer when it only forwards or copies
values. A one-owner use case may call that owner directly. Share a type or helper
only when real callers need it. Do not build a generic service/plugin framework.

Read the [Ketho map](KETHO_RUST_PORT.md) and [Emmy boundary](EMMYLUA_UPSTREAM.md)
for those changes. Upstream language-server handlers are private to `emmylua_ls`;
its public analysis API is not an importable LSP-handler SDK.

## 3. Inputs and generated data

Keep one readable project configuration: addon/source roots, flavor, TOC and
annotation-library/output paths as needed. Discover TOCs within the selected
root; report ambiguity. The tool derives internal IDs and analysis settings.
Users should not construct digests, compatibility reports, capability registries
or proof manifests to run a check or search.

User source and configuration are authoritative inputs. Index databases,
generated annotations and intermediate reports are **rebuildable derived data**.
Use one current format/schema. On mismatch, regenerate into tool-owned output
and switch only after success, or explain which input is missing. Do not retain
old readers, migrate historical cache epochs, or delete source/configuration.
Do not follow arbitrary paths from an old cache while rebuilding it.

SQLite transactions provide atomic index updates. Store plain records and FTS
rows directly; no SQLite image inside hex inside JSON inside another database.
Keep a small source revision/flavor/build-state record and file locations. Use a
content fingerprint only if an actual change-detection/cache operation needs it;
application-level signatures, self-hashing results and cryptographic chains are
outside the product. Normal reads need no durable operation/replay journal.

## 4. What is removed

These are scope removals, not deferred prerequisites for completion:

| Removed scope | Replacement |
| --- | --- |
| Per-feature schemas/host profiles and internal backward compatibility | One current model and explicit regeneration of derived data. |
| Result/pack signatures, attestations, authority envelopes, promotion/holdout/canary/LKG | Normal typed results, ordinary code review and focused behavior checks. |
| Content-addressed publication, retained epochs, generic leases/GC/backup/quarantine/migration workflows | A local rebuildable index, transactions and normal resource lifetimes. |
| External model/provider/Codebase Memory subsystem | Local Blizzard/addon search and actual source links. |
| Lineage/review/promotion, migration-impact proof and structural-fingerprint platform | Current-source search, definitions, references and useful source relationships. |
| Wasm guests/host and independently updateable algorithm plugins | Existing Ketho literal algorithm called natively. |
| Daemon, HTTP transport, release executor, self-updater/rollback/installer framework | CLI and stdio; ordinary Cargo build and downloadable release artifacts. |
| Full E0–E7 certification, frozen-pending/checksum/status matrices | The three real workflows above and checks for changed behavior. |

Remove implementation, callers, exclusive tests and obsolete documentation as
each queued cut lands. Never add a disabled fallback or new archive. Retain the
native literal implementation before deleting its Wasm shell. A feature earns
new scope only through a concrete user task that existing code cannot solve.

## 5. Correctness that matters

Parse source without executing Lua or repository scripts. Limit files, depth,
work and output; keep cancellation and useful defaults. Preserve flavor, source
revision, byte spans and unsaved document versions. Reject malformed paths and
respect configured roots. Keep unknown/partial data visible; text similarity is
not an API replacement, and static results do not prove combat/taint safety.
These are ordinary implementation requirements, not a new evidence subsystem.

Tests follow [TEST_STRATEGY.md](TEST_STRATEGY.md). Completion is measured by
[PROJECT_COMPLETION_MATRIX.md](PROJECT_COMPLETION_MATRIX.md), not old package
counts, all historic fixtures, or every planned language-server operation.
