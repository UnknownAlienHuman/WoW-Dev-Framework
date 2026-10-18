# Scope decision

## ADR-058 — A focused WoW development tool

Accepted by explicit owner direction, 2026-10-18 UTC. EmmyLua, native Ketho
annotations, editor-independent addon checking and Blizzard UI source search
replace the E0–E7 platform plan. Conflicting compatibility, cryptographic,
provider, plugin, lineage and release-system obligations are retired even where
their implementation still exists. They are not mandatory unfinished work.

[ARCHITECTURE.md](ARCHITECTURE.md) is the sole product manifesto.
[IMPLEMENTATION.md](IMPLEMENTATION.md) maps the current code and replacement
approach; [REFACTOR_QUEUE.md](REFACTOR_QUEUE.md) orders tasks. Keep scope and
implementation details in those documents instead of duplicating their rules
here or adding a decision record for each helper.

The superseded decisions remain in [Git history](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/blob/5aa5f6bc8421eece38309b2fffe85d7f60377f22/docs/DECISIONS.md).
