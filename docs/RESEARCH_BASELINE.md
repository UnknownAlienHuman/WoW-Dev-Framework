# Research baseline

**Status: research**
**Architecture source review date:** 2026-08-15

This document records the external inputs that informed architecture v8.0. It is not a claim that every listed revision is the newest available revision.

Optional operator context is advisory, disabled by default, and configured outside the repository; current Blizzard source remains authoritative.

## 1. Authority model

For released Reference Packs and production findings:

1. exact materialized Blizzard UI/API content for the selected build;
2. generated API documentation and UI implementation/XML/TOC in that snapshot;
3. reviewed digest-bound corrections;
4. runtime probes tied to an exact client build and scenario;
5. differential oracles;
6. pinned third-party implementations;
7. community reports and model inference.

Acquisition provider, repository popularity, or semantic similarity does not alter the authority order.

## 2. Core tooling snapshot used by v8.0

| Project | Checked revision | Architectural use |
|---|---|---|
| [Ketho/vscode-wow-api](https://github.com/Ketho/vscode-wow-api) | `d0b5b51fac4c52c493371b9b18e66ce604ea4326` | Annotation compatibility, curated correction behavior, parity fixtures |
| [EmmyLuaLs/emmylua-analyzer-rust](https://github.com/EmmyLuaLs/emmylua-analyzer-rust) | `aaaca68425d9362876228649b0b8d92f07654daa` | Upstream Rust syntax/semantic analysis and diagnostics host dependency |
| [LuaLS/lua-language-server](https://github.com/LuaLS/lua-language-server) | `7a73c7889c1ec981dfd76fba38f5096379f62f99` | Compatibility oracle and secondary annotation/diagnostic baseline |
| [Tencent/LuaHelper](https://github.com/Tencent/LuaHelper) | `15c9fac58a73c6f780257c81664a03b52d941f9d` | Donor lessons for selective invalidation and failure isolation; not a dependency |
| [NumyAddon/FramexmlAnnotations](https://github.com/NumyAddon/FramexmlAnnotations) | `b38f5ec0f9fbf493e31c4f060b6f3db2ef743c78` | FrameXML differential oracle and convention corpus |
| [spartanui-wow/wow-api-mcp](https://github.com/spartanui-wow/wow-api-mcp) | `81e96a304315bca2ac0ac1c064feaf97c323803e` | Minimal API query UX and parity task source |
| [DeusData/codebase-memory-mcp](https://github.com/DeusData/codebase-memory-mcp) | `e513beb487bea21105a031da328872c1b6f58eac` | Optional broad semantic/source candidate bridge |

The source architecture also recorded the then-checked Emmy crate versions:

```text
emmylua_code_analysis 0.25.1
emmylua_parser 0.29.1
```

These are research records, not automatic dependency selections. E0 must re-pin and compatibility-probe the chosen revision.

## 3. Blizzard UI samples used by v8.0

```text
then-current comparison snapshot
    revision: 31c7f7b9cc79e56c986b365c06a6afbcf3c9177b
    recorded build: 12.1.0 (69299)
    recorded date: 2026-08-13

historical comparison snapshot
    revision: f0084386950fe3dc31a1d61de33b364e268cf66b
    recorded build: 10.0.0 (46293)
    recorded date: 2022-10-25
```

These snapshots were used to demonstrate that semantic symbols may survive while package/container paths change. A production pack must use its own manifest, exact source digest, and current verification.

## 4. Representative framework/addon corpus

| Repository | Recorded revision | Calibration focus |
|---|---|---|
| [WoWUIDev/Ace3](https://github.com/WoWUIDev/Ace3) | `d295b12f8b889a30e86e0e901c5494df4b149c49` | addon/module factories, lifecycle, embedded libraries, events/messages |
| [oUF-wow/oUF](https://github.com/oUF-wow/oUF) | `b6d1005ea6b6e4cdf1e2d7729ada8d3a08986074` | style/element registries, frame factory, prototype, lifecycle |
| [WeakAuras/WeakAuras2](https://github.com/WeakAuras/WeakAuras2) | `7bb9d239987921e41166dc2395f6e31ec78c94d7` | namespaces, schemas, registries, triggers, factories, state machines |
| [BigWigsMods/BigWigs](https://github.com/BigWigsMods/BigWigs) | `2b8efea7b77801ed8b93e9a1720feb5b96eeb4d5` | domain modules, lifecycle, events, messages, declarative data |
| [Tercioo/Details-Damage-Meter](https://github.com/Tercioo/Details-Damage-Meter) | `c139bf364e92455a69933b242a1b743e7a696d6b` | plugin factories, registries, persistence, lifecycle, compatibility |
| [Tercioo/Plater-Nameplates](https://github.com/Tercioo/Plater-Nameplates) | `0b764e35bb3c6248097866a615c3a874c97ad290` | script/hook surfaces, nameplate factories, profile/script boundaries |

These repositories are calibration corpora and implementation evidence. They do not define Blizzard API contracts.

## 5. Comparative ecosystems

The architecture used other ecosystems only for transferable tooling patterns:

| Ecosystem | Recorded project/revision | Lesson |
|---|---|---|
| Roblox | `MaximumADHD/Roblox-Client-Tracker@adfa0709`, `JohnnyMorganz/luau-lsp@d5df9af2` | versioned client/API tracking plus generated types and a separate project tree |
| Minecraft | `FabricMC/yarn@ee985908`, `FabricMC/fabric-loom@b04ca661` | versioned mapping artifacts, deterministic cache, updates separate from normal queries |
| FFXIV | `aers/FFXIVClientStructs@2db63985` | patch-specific generated bindings and explicit failure when resolution is unavailable |

No code or platform semantics are imported merely because a pattern is useful.

## 6. Required revalidation before implementation

Before E0/E1 pins an upstream input:

1. identify the newest candidate revision required by the task;
2. inspect license and public API surface;
3. run a compatibility probe against our fixtures;
4. record changed behavior, diagnostics, schemas, and performance;
5. update the pinned revision only when the active milestone still passes;
6. retain last-known-good and rollback notes.

Before releasing a Reference Pack:

1. materialize the exact Blizzard source snapshot;
2. record acquisition provider and content digest separately;
3. confirm profile/build/Interface identity;
4. run APIDocumentation, FrameXML, annotation, restriction, and lineage validation;
5. publish capability/coverage and differential reports;
6. do not label a floating branch or “live” URL as a durable profile.
