# Pinned real-addon fixture: `UnknownAlienHuman/roth-ui`

**Status:** normative E2-C read-only implementation fixture selection; expected generated manifests remain pending until code exists.

## Pin

```text
repository: UnknownAlienHuman/roth-ui
commit: 1656d4b9d33be914be2058460520e7423668d95c
package/TOC: Roth_UI / Roth_UI.toc
license: MIT, LICENSE.txt blob 7a12feda19dfade4fe3b3d617a223f56e2981240
fixture profile: user-addon-readonly-project-structure-v1
```

The fixture is pinned by commit and exact file blobs. Branch `main` is provenance only and must not be used as a floating test input.

## Why this fixture

The pinned package exercises E2-C structures that synthetic fixtures alone do not scale-test:

- Retail Interface metadata;
- one required dependency and multiple optional dependencies;
- account and per-character SavedVariables declarations;
- a long ordered TOC file list across embedded libraries, core systems, defaults, elements, units, and action-bar modules;
- XML files included directly by the TOC;
- XML `<Script file=...>` expansion for embedded libraries;
- nested relative XML script paths;
- first-party and vendored/dependency-like library structure that must not be classified by path names alone.

## Selected read-only scope

Mandatory fixture inputs:

```text
Roth_UI.toc
embeds/rLib/rLib.xml
Libs/LibSharedMedia-3.0/lib.xml
LICENSE.txt
all TOC-referenced source paths for existence/order/content-manifest validation
```

Implementation may materialize the complete pinned repository under the source snapshot budget, but closed golden assertions focus on package/TOC/XML/load/inventory behavior. Lua semantic facts are supplied only through `wow-emmy`; fixture code is never executed.

## Expected source-derived observations

The fixture is expected to establish, after implementation/freeze:

```text
one selected Roth_UI TOC variant for the configured Retail target
required dependency: oUF
optional dependencies: RothFont, RothLib
SavedVariables roots: Roth_UI_DB and Roth_UI_DB_Char with distinct scopes
exact TOC source order for every selected entry
XML load expansion from embeds/rLib/rLib.xml to six Lua script files
XML load expansion from Libs/LibSharedMedia-3.0/lib.xml to LibSharedMedia-3.0.lua
unresolved/missing/duplicate/path/case state according to the exact pinned source manifest
first-party versus embedded library/source roles without repository/path heuristics
```

These are project-source observations, not API, security, runtime, performance, or compatibility verdicts.

## Explicit nonclaims

The fixture does not prove:

- that the addon loads successfully in the WoW client;
- that all dependencies are installed or compatible;
- that every frame exists at `ADDON_LOADED`;
- event payload readability;
- taint, combat, protected, forbidden, or managed-object safety;
- performance or absence of runtime defects;
- E2-C parser completeness for every WoW addon convention;
- production semantics specific to Roth UI/oUF/LibSharedMedia/rLib.

Those require separate reference, rule, runtime, corpus, and compatibility evidence.

## Repository-name independence

Mandatory mutation:

```text
rename repository, package display metadata, directories, and irrelevant local identifiers
preserve exact normalized TOC/XML/Lua structural facts
expect identical universal facts/proposals except source identities and new project generation
```

Any E2-C parser, adapter, recognizer, or graph result that depends on `roth-ui`, `Roth_UI`, `oUF`, `Libs`, `rLib`, or another fixture name without an exact public/source semantic field fails the mutation gate.

Exact dependency names and TOC file paths remain source facts where the TOC contract says they are semantically meaningful. They cannot trigger hard-coded product behavior.

## License and source use

The repository license is MIT at the pinned commit. Preserve copyright/license notices for any source excerpt copied into the test corpus.

Preferred fixture strategy:

1. pin source file/blob IDs and expected generated fact/manifests;
2. avoid copying large implementation files into this framework;
3. copy only minimal licensed malformed/mutation snippets when a closed parser test requires bytes;
4. record source handles and hashes for all extracted fixture bytes;
5. never execute repository code or workflows.

## Freeze gate

Before the first E2-C Rust commit, freeze:

```text
complete selected source manifest and SHA-256
TOC/XML profile and normalized record IDs
selected package/variant ID
all referenced-file resolution results
load units/direct edges/reachability records
expected analyzer unit manifest boundaries
adapter/recognizer/graph proposal expectations selected for this fixture
repository/path/name mutation outputs
fixture scope and license/provenance manifest
expected ProjectIndexCandidate ID/digest
```
