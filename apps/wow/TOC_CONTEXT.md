# Conditional TOC selection

The `wow-service/local-project-toc/1` input accepts `main.load_context` alongside
`main.root` and `main.toc`. Commands are unchanged. This is an optional extension;
plain TOCs and the old inline/explicit-file input modes remain available.

```json
{
  "root": "addon",
  "toc": { "path": "MyAddon.toc" },
  "load_context": {
    "game_types": { "classic": false, "mainline": true, "standard": true },
    "family": "Mainline",
    "game": "Standard",
    "text_locale": "enUS",
    "location": "Game",
    "environment": "Global"
  }
}
```

This is the `main` fragment, not a complete config or a universal client profile.
Supply values for the **selected target**. None is inferred from Interface, flavor,
OS locale, folder names or a globally hard-coded client table. In `game_types`,
`false` means a known alias that does not match; an omitted alias means unknown.
Include every alias used in an admitted condition, including nonmatching ones.
The context is operator-declared, not independently attested against a running
client. All values and the complete selected `ProfileIdentity` enter plan identity.

## Selection

Supported clauses are `AllowLoadGameType`, `ExcludeLoadGameType`,
`AllowLoadTextLocale`, `AllowLoad` (`Game`, `Glue`, `Both`) and
`AllowLoadEnvironment` (`Global`, `Secure`). Whitespace/comma lists are supported;
values match case-insensitively. Values within one clause form an OR; multiple
clauses form an AND. Adjacent clauses are supported. `[Bootstrap]` retains its
existing static meaning and can accompany conditions.

`[Family]`, `[Game]` and `[TextLocale]` in file paths expand to the explicitly
supplied values, including forms such as `Init_[Game].lua`. Expansions are bounded
single components, not environment interpolation. Root confinement, path admission,
no-follow reading and the existing XML closure apply **after** expansion.

A condition can also select a metadata line. Only included metadata contributes
to Interface validation or dependency declarations. Skipped metadata is retained
with its decision and source span; variants are never merged. The parser's bounded
dialect treats unrecognized bracket syntax conservatively, including in metadata.

An excluded line remains in the receipt but its file is never resolved, opened or
added to analyzer Main. Missing expansion values are irrelevant to a conclusively
excluded line. Unknown clauses, missing required selectors and unclassified
game-type aliases instead produce `unresolved` plus explicit blocking issues.
An unknown clause is not dismissed merely because another clause excludes the
line. No guessing of the client's special handling of unknown game-type names.
If no analyzable Lua remains, acquisition fails rather than returning clean.

## Receipts and limits

JSON load records include `selection`, typed `conditions` and `declared_target`
(the path before variable expansion, without recognized suffixes). Existing
`target` is only the resolved selected path. Exact raw spans/digests remain;
TOC source text is still private to the retained plan. Text check output adds
excluded/unresolved record counts. Plan profile and digest domain advance to v2;
unchanged source under a different context cannot reuse the old generation.

At most 64 explicit game-type matches, 16 bracket clauses per line and 64 values
per condition; selector/path-component values are at most 64 ASCII bytes. Duplicate
game-type keys, path-bearing expansion values and invalid context shapes reject.
Existing source, closure, parse, cancellation and output limits are unchanged.

Package-wide filters are admitted as described below. Environment-changing
`LoadIntoEnvironment`, conditional XML attributes, client variant discovery,
inline XML Lua and runtime load success remain unimplemented. Their existing blockers stay
visible. This is static file selection, not full E2-C or live-profile WoW rules.

Syntax evidence reviewed on 2026-09-23: Gethe `live` at
`09b9db7948abc9b9648dedaab51eb0cf3ee67b31`,
`Interface/AddOns/Blizzard_SharedXML/Blizzard_SharedXML.toc`; BigWigs `master` at
`fe555692dca7375f2ab1f230e4282b487e1d70c4`, `BigWigs.toc` (adjacent conditions,
comma lists and embedded `[Game]`/`[TextLocale]`). No donor source is copied into
this implementation, and neither revision is a permanent client target.

## Package filters before source acquisition

The complete captured TOC is now checked for `## AllowLoad:`,
`## AllowLoadGameType:`, `## ExcludeLoadGameType:`, `## AllowLoadTextLocale:` and
`## AllowLoadEnvironment:` before resolving any referenced Lua/XML path. A filter
applies to the whole selected package even when it appears after file entries.
These use the same bounded predicates and explicit context as line conditions;
`Both` is unrestricted and needs no location context. No Interface/filename/client
alias inference is added. A missing filter imposes no restriction.

Only included conditional metadata contributes a filter. Excluded metadata is
inert; unresolved filter declarations, unknown filter names, empty/malformed values,
unknown aliases and duplicate active declarations refuse acquisition. The loader
does not guess first/last-wins behavior. Unresolved takes precedence over excluded.
At most 64 active/unresolved package-filter declarations are admitted.

A nonmatching package returns `project_target_excluded` (CLI exit 3); an ambiguous
or insufficiently specified selection returns `project_target_unresolved` (exit 2).
Both are structured local-operation failures in JSON/text, not clean checks, empty
analyzer snapshots or syntax-error usage output. No Main/XML/Library files or
analyzer calls are reached. Config and pinned metadata may already have been read.
Correct the chosen TOC or explicit context; the loader never chooses another variant.

For an admitted package, `package_gate` load records retain the source span,
raw digest, suffix conditions and evaluated gate. Text output includes their count.
No plan/generation is published for rejected acquisition. Load profile and digest
domain advance to v3; file bodies, existing fixtures and other input modes are unchanged.

Package-filter syntax review: the same Gethe revision cited above,
`Interface/AddOns/Blizzard_UIPanels_Game/Blizzard_UIPanels_Game_Mainline.toc`,
contains package-wide `AllowLoad` and `AllowLoadGameType`. Implemented behavior is
a bounded static dialect, not proof of client execution or full TOC completeness.
