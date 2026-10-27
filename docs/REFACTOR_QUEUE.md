# Implementation order

Read [current status](CURRENT_STATUS.md) first. Architecture defines scope,
[IMPLEMENTATION.md](IMPLEMENTATION.md) defines owners, and
[WOW_LINTER.md](WOW_LINTER.md) defines domain behavior. This is the only ordered
queue; PR and Issue history cannot expand it.

A slice is complete only when ordinary input reaches the real owner and produces
the named visible result. Build gates are not product observations.

<a id="1-first-usable-ordinary-addon-check"></a>

## 1. First usable ordinary addon check

Do not implement the complete Blizzard loading model before the first command.
Deliver the smallest correct `LuaTypes` path, then add context separately.

### 1A. Explicit physical-Lua `wow check`

Effective input after one optional explicit config plus CLI overrides:

- addon root;
- explicit canonical relative TOC;
- exact flavor `retail`;
- one or more explicit annotation-Library roots;
- optional `emmy_diagnostics`, `format`, `fail_on`.

Defaults: upstream Emmy diagnostics, text, `fail_on: error`. Slice 1A exposes no
WoW-check option and runs zero WoW rules.

```text
CLI/config
  -> strict merged input
  -> one public Service ordinary-check operation
  -> AddonLoadTarget + physical_lua Project plan
  -> requested-scope blockers + admitted path authority
  -> compiled-pin Main + per-root Library snapshots
  -> same ConfiguredAnalyzerSession requirements/diagnostics
  -> internal check_addon
  -> root-relative result or typed pre-result error
```

#### Project selection and admission

- Replace retained `ProfileIdentity`/mandatory-Interface plan identity and
  validation with `AddonLoadTarget { retail, optional positive target Interface,
  optional TocLoadContext } + projection scope`. Slice 1A supplies flavor only;
  legacy callers adapt a real profile, the direct route never fabricates one.
- Complete T23 metadata-by-effect classification first. Display/localized/custom
  and known non-selection metadata are observations; malformed or selected-
  context-changing directives remain invalid/located.
- `physical_lua` parses TOC/direct Lua, retains XML records/spans and does not open
  XML. Selected XML is a located deferred blocker. Malformed XML cannot erase
  direct Lua diagnostics. Empty Main is valid: complete when nothing is selected,
  partial when selected deferred input exists. No dummy source.
- Add the standalone scope mapper in Service. Missing direct Lua named by a safely
  parsed TOC, required dependency, unresolved included condition, deferred XML or
  selected context uncertainty blocks `LuaTypes`; optional absence, repeated TOC
  occurrence, Interface without target, `LoadWith` and unused metadata do not.
  Never use package-only mapping or global completeness as the direct contract.
- Missing/unreadable root or explicit TOC, unsafe path/bytes, unrecoverable TOC
  syntax, config/source/Library budget refusal or malformed Library is pre-result
  `invalid_input`, not partial.

#### Safe roots, files and snapshots

- Read config and source only through a narrow Project API wrapping crate-private
  component-wise no-follow `open_workspace`; do not use ambient
  `ProjectInputDirectory::open`.
- Each root gets request-local `InputRootId = role + ordinal`, original display
  label, normalized lexical absolute authority descriptor and held directory.
  Reject root descriptor text above 32 KiB and unsupported UNC/device prefixes.
- For each unique admitted Lua derive nonserialized `InputPathAuthority` by
  joining normalized root descriptor and canonical relative path, slash-normalized
  and Unicode-lowercased with Project's portable case policy. Reject duplicate
  authorities across Main/Libraries. Do not use inode/file ID, hard-link,
  content-digest, filesystem canonicalization or snapshot ID as path authority.
- Do not reject root ancestry. Disjoint nested roots are valid; equal relative
  paths in distinct roots are valid when joined authorities differ. Repeated TOC
  records are not duplicate files.
- Require each Library root to yield Lua. Reject duplicate root authority keys and
  duplicate whole snapshot IDs. Fixed aggregate ceilings: 64 Library roots,
  8,192 Library files, 256 MiB; retain existing per-root/Main ceilings.
- Use `EmmyBackendIdentity::compiled_pinned()`, Main universe `Project`, one
  universe `BlizzardUi` snapshot per Library root and canonical role-relative
  paths. Absolute/path-authority data never enters semantic identity or output.

#### Service and Emmy ownership

- One public Service operation owns Project preparation, plan/Main/authority
  validation, requested blockers, snapshot construction, snapshot/root mapping,
  configured-owner requirement collection and lower invocation.
- Frontends never open source, build snapshots/lower requests or attach blockers.
  Keep the exact-snapshot request/check internal after callers move.
- Remove unconditional chunk incompleteness/hard-coded `Partial`. On the same
  `ConfiguredAnalyzerSession`, walk registered Main expression AST, match
  `LuaLiteralExpr`/`LuaLiteralToken::Dots` with no enclosing `LuaClosureExpr`, and
  retain the lowest-starting exact path/content/span per affected file. No raw
  text scan or second parser/session. Service validates records against Main and
  limits only affected files. Injection is Slice 1B.

#### Strict and canonical diagnostic configuration

- Read at most one explicit no-follow config, 256 KiB. Reuse duplicate-key/depth-
  bounded `StrictJson`; reject unknown outer fields and duplicate scalar options.
  No implicit discovery.
- CLI scalar values replace config scalars; repeated CLI Libraries replace the
  array. Merge before validating required fields.
- Pinned `EmmyrcDiagnostic` is permissive. Prevalidate exactly `disable`, `enable`,
  `globals`, `globalsRegex`, `severity`, `enables`; compile every regex. Reject
  unknown nested keys, invalid regex and `diagnosticInterval`. At the pinned
  revision that field is LS debounce only, not code-analysis configuration.
- Replace raw-JSON config equality with canonical analysis semantics in
  `EmmyAnalysisConfiguration::equivalent`: preserve `enable`; sort/dedup
  `disable`, `enables`, `globals`, `globalsRegex`; sort severity by diagnostic
  code; omit `diagnosticInterval`. Omitted diagnostics, `{}`, reordered or
  duplicated set-like entries reuse the same owner; real semantic changes force
  cold rebuild. LS debounce remains frontend state.
- No `.emmyrc` discovery or lenient fallback.

#### Errors, result and exit

- Carry stable code, role, snapshot/`InputRootId`, role-relative path and bounded
  message through Project/Emmy/Service. `LibraryHealthFailed` is caller input, not
  `ComponentUnavailable`; never recover origin from strings/synthetic paths.
- Public operation error codes are exactly `invalid_input`, `cancelled`,
  `internal`. Message <= 4 KiB. `root_index` is Library-only and zero-based;
  paths are role-relative. Do not enlarge global `ServiceError` into a provenance
  framework.
- Deterministic order: diagnostics by role/path/span/code/message; limitations by
  code/location/message. Explicit threshold rank Error > Warning > Information >
  Hint; preserved `Unknown` meets every threshold.
- Pre-encode escaped text/JSON under 32 MiB, then one write. Complete/partial
  result -> stdout. Admitted-format pre-result error -> stderr. Early CLI grammar
  error -> text stderr. Final cancellation checkpoint after encoding/before first
  byte. After output starts, no retry/replacement/second envelope.
- Result-size refusal -> invalid_input/64. Serializer, cancellation-handler setup
  or I/O failure -> internal/4. Exit precedence: 64, 130, 4, 2, 1, 0. Findings
  remain in partial output. Human text visibly escapes controls; JSON stays exact.

#### Observe before declaring Slice 1A complete

Record exact commands/results for:

1. valid clean typed call and invalid typed call;
2. top-level ellipsis partial plus named/local/anonymous-function, string/comment
   near-negatives;
3. required-dependency or deferred-XML partial preserving diagnostics;
4. safely referenced missing child Lua partial;
5. clean empty Main and deferred-XML empty Main partial;
6. malformed Library with structured root/path;
7. duplicate authority rejected; disjoint nested roots and equal relative paths in
   distinct roots accepted;
8. unknown diagnostic key, invalid regex and rejected `diagnosticInterval`;
9. omitted diagnostics vs `{}` and reordered/duplicated set-like fields reuse one
   owner; a real semantic diagnostic change rebuilds cold;
10. text/JSON result/error channels, size bound and exit precedence.

<a id="source-traced-replacement-boundary"></a>

### Replacement boundary

The target route is `wow-project -> public wow-service ordinary operation ->
wow-emmy/internal check_addon`. Do not adapt
`LocalCommand -> LocalProjectBackend -> generic Service::check`, fabricate its
Reference/Rules/publication prerequisites or preserve a compatibility mode.

<a id="exact-owner-split"></a>

### Owner split

- Project: safe roots/files, target/scope plan, exact bytes, path authority,
  source projection and source-located acquisition errors.
- Service: trusted assembly, requested blockers, exact snapshots, structured
  operation errors and result.
- Emmy: one Lua parser/type/diagnostic owner and same-session requirement facts.
- Frontends: precedence, display roots, protocol framing, rendering and exit only.
- Path-sorted snapshots are analysis identity, not WoW runtime order.

### 1B. Dependencies, XML and chunk injection

After 1A works:

- required/available optional dependency closure;
- exact selected package/file order beside snapshots;
- source-projectable XML virtual units and exact mappings;
- ordinary `(addonName, addonTable)` helper only for requirement-marked chunks;
- one reviewed XML handler adapter;
- physical+virtual, XML-only and valid empty scopes;
- malformed included XML remains local and preserves independent diagnostics;
- coherent add/edit/remove/rename and TOC/XML/Library updates.

### 1C. Special loading

Then add independently:

- Stable/PTR/Beta and `OnlyBetaAndPTR`;
- `LoadWith` as reverse trigger, never dependency;
- implicit bounded bindings and mapped `keystate` callbacks;
- secure/global/unit environments, bootstrap/load-first and unsupported
  `SuppressLocalTableRef`;
- source Interface zero compatibility and raw unsupported `Secure`;
- known lifecycle/error/retrieval metadata and machine SavedVariables declarations
  without reading user data.

Donor behavior stays scoped; do not claim exact client timing without authority.

## 2. Generated Library semantics and selected WoW checks

1. One supported builder and ordinary Library reader.
2. One optional typed `source-report.json`; plain Lua remains valid.
3. Correct `Environment`, `MayReturnNothing`, `NilableContents`, stride, mixin and
   selected restriction/Secret semantics.
4. Add one useful direct WoW check only after semantic binding is correct. Run
   selected checks only; no provider/capability/evidence platform.
5. Preserve maps and unresolved/lossy projection honestly.

Observe generated and plain-Lua Libraries through the same public check.

## 3. Thin LSP/MCP adapters

- LSP calls the same public Service owner with exact unsaved text/version,
  negotiated positions, version gaps, standard empty results and stale rejection.
- MCP advertises only implemented check/search/context tools; bounded `_meta` and
  valid unknown notifications do not terminate the session.
- General Lua completion/hover/definition/references/rename/signature/symbols stay
  upstream EmmyLua.

Record exact framed sequences before `observed`.

<a id="direct-source-lookup-and-remaining-platform-removal"></a>

## 4. Direct source index, search and context

1. One rebuildable SQLite database with exact selected bodies, symbols/spans and
   consumed relations.
2. Query in a later process without Project/Emmy/Graph replay.
3. Source-body FTS and exact symbol lookup with deterministic located hits.
4. Readable exact indexed context and direct relations.
5. Transactional replace/update/delete; distinguish complete empty, unavailable/
   partial and truncated.
6. Real SQLite interruption/lock-wait cancellation where supported.

Observe a body term absent from filenames, a symbol and one TOC/XML relation after
restart.

## 5. Remove displaced platform code

Only after callers move, remove compatibility/certification reports, unused
publication/replay, generic component/capability/presentation results,
recognizer pack/plan/matcher/proof round trips, unconsumed graph fingerprints,
nested SQLite/hex/JSON, generic Store lifecycle, Search federation and Context
packs. Remove exclusive options/fixtures/prose in the same slice. Do not rename or
wrap the platform.

## 6. Measure one real workflow

After the three public workflows run, measure one selected real Blizzard source:
file count, elapsed time, database size and first demonstrated bottleneck. Fix it
and rerun the same input once. Do not create a benchmark/telemetry/certification
platform.
