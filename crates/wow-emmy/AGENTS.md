# `wow-emmy` contribution rules

Read root [AGENTS.md](../../AGENTS.md), the whole
[manifesto](../../docs/ARCHITECTURE.md), and the selected
[analysis route](../../docs/IMPLEMENTATION.md#addon-configuration-and-emmy-analysis).
Use [EmmyLua guidance](../../docs/EMMYLUA_UPSTREAM.md) for public APIs and pin changes.

Keep upstream types and calls inside this adapter; downstream owners consume
only the facts they use. Reuse one actual `EmmyLuaAnalysis` for the selected
Main/Library inputs. Current `AnalyzerSession` still owns separate engines;
[diagnostic guidance](DIAGNOSTIC_NORMALIZATION.md) explains why merely deleting
a constructor is insufficient. Standard Lua editor features use upstream's
server; its private handlers are not an SDK to import or copy.

Preserve Lua 5.1 configuration, Main/Library roles, virtual-source mappings,
UTF-8 half-open spans, document versions, cancellation and coherent updates.
Never publish half-applied or stale facts as current. Missing/partial analysis
is visible; static unresolved names or flow facts do not prove WoW API absence,
Secret/combat/taint safety or runtime behavior.

Keep input inert and rooted. Do not execute Lua, change editor settings, expose
private host paths or add a parser, analysis cache or session framework.
Keep useful bounds with the operation; do not add result-hash or fixture-freeze
requirements. Source and user configuration survive derived-data replacement.

Use the selected Issue's observable check and existing fixture helpers. Run a
larger consumer or platform check only for a concrete remaining risk or coverage
claim. Replace obsolete representations, options, exclusive tests and docs in
the same change; no old-format reader or archive.
