# Project manifesto

WoW Dev Framework helps addon developers understand Blizzard APIs, check their
Lua code, and find the source they need. Its foundation is **EmmyLua and a native
Rust port of Ketho's annotation pipeline**. The purpose is a useful development
tool that remains easy to understand, change and finish.

## What the product must deliver

- Generate a usable `.lua` annotation library for the selected Blizzard source
  and client flavor, preserving the useful meaning of Ketho's output.
- Check an addon with EmmyLua and that library; explain actual problems at their
  source locations.
- Search Blizzard UI and addon source for symbols, text and relevant TOC/XML
  relationships; return readable excerpts and navigation to the original code.

These workflows must work from ordinary project inputs without VS Code. CLI is
the complete local entry point; focused stdio LSP/MCP access serves the same
workflows. General Lua language intelligence belongs to upstream EmmyLua.

## Principles

- **Useful behavior earns its cost.** Every feature and component must serve a
  current developer task. Existing code, an old plan or a test does not justify
  keeping a feature. Finish the useful workflow before adding adjacent features.
- **Reuse the substance.** Keep EmmyLua's engine and Ketho's domain behavior.
  Port only the integration the product needs; do not reproduce an editor,
  language server, interpreter or general tooling platform.
- **One clear implementation.** A responsibility has one cohesive implementation.
  Remove redundant representations, forwarding layers and configuration. The
  goal is understandable behavior with fewer moving parts, not a line quota or
  compressed code. A small layer can still be unnecessary.
- **Replace and remove.** There is one current internal model. Derived indexes
  and annotations are disposable and regenerable; old caches and fixtures do
  not create a backward-compatibility obligation. User source and configuration
  remain intact. History belongs in Git, not disabled alternatives or archives.
- **Evidence follows behavior.** Validate the result a developer relies on.
  Tests and reports help establish that result; their count, signatures or
  completion statuses are not product features.

## Requirements that matter

Setup uses understandable source roots, flavor, TOC and library paths with
useful defaults. Common operations must stay responsive, reuse work and update
results when source changes. Output preserves the selected source and accurate
locations; incomplete or unsupported analysis is visible. Static analysis does
not claim runtime, combat or taint safety.

Source is read without executing it. Respect configured roots and user files,
bound work and output, and allow cancellation. Editors and agents receive valid
protocol responses. The product needs no Lua, Node, Python or VS Code runtime.

## Outside the product

No internal legacy compatibility or migration platform; cryptographic result
proofs, signed/governed packs or certification system; historical lineage,
promotion or fingerprint platform; external model/provider framework; Wasm
algorithm plugins; generic backup/retention administration; daemon/HTTP hosting;
custom installer, self-updater or release executor. These are removed scope,
not unfinished obligations to carry into the next plan.

[IMPLEMENTATION.md](IMPLEMENTATION.md) owns code placement, reuse and the
replacement approach. The [queue](REFACTOR_QUEUE.md) orders work. This manifesto
is the developer's standing product context; old package specifications cannot
expand it. [ADR-058](DECISIONS.md) records the scope change.
