# EmmyLua, annotations, and diagnostics

EmmyLua is the single Lua parser and semantic analyzer. `wow-emmy` adapts its
public `emmylua_code_analysis` APIs and exposes normalized syntax, symbol,
reference, call, local-flow, and diagnostic facts to framework owners. It keeps
the selected Lua 5.1 configuration, physical/virtual file identity, canonical
UTF-8 byte spans, and incremental session semantics. Do not add a second parser,
AST, analysis cache, interpreter, or full Lua language server.

Upstream `emmylua_ls` already implements ordinary editor handlers such as
hover, completion, definition, references, rename, signature help, symbols, and
diagnostics. Its handler modules are private to that crate; semantic analysis
is public through `emmylua_code_analysis`. Use the upstream server for general
Lua editor features. Framework-owned CLI/stdio operations add WoW source and
project context where needed; they do not need to clone the upstream handlers.

## WoW annotation library

Ketho/vscode-wow-api is the donor for annotation semantics. The native pipeline
uses `wow-reference` for bounded, nonexecuting Blizzard source admission and
normalization and `wow-annotations` for typed projection and Emmy-style `.lua`
output. Select one exact Blizzard source revision, flavor, and relevant TOC/API
documentation. Keep raw or unsupported data visible in reports; partial input
must not look like an absent API. Generated stubs are inert analysis libraries,
not addon runtime files.

The existing source-to-library example and Service builder provide the current
generation paths. Users can add the resulting library through the normal library
configuration of EmmyLua or another compatible analyzer. Do not require VS
Code, install an extension, mutate editor settings, or create a separate
annotation loader. Regenerate derived library/report outputs when source,
corrections, catalogs, or projection semantics change.

## WoW diagnostics and scope

Built-in EmmyLua diagnostics remain upstream behavior. Existing WoW-specific
project/reference operations can combine normalized analyzer facts with the
selected source/profile data through their current owners. Report exact source
locations and preserve unknown/unsupported results. Static findings do not
establish runtime, combat, protected-action, Secret, or taint safety.

This document describes product responsibilities, not a requirement for a new
provider framework or a full catalog of diagnostics. Verify actual implemented
operations in their owner code and current application surface. Add a rule only
for a concrete developer need that existing owners cannot answer. For changed
semantics, check one representative positive and negative consumer case; use a
full corpus run only when making a corresponding release coverage claim.
