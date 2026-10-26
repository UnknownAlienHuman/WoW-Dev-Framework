# TOC trigger, product-channel and binding-callback trace

**Original trace base:** `337b9b6e676af6735f416e8527176ed17e865172` (`main` after PR #255)
**Consistency review base:** `0784d0cb5f2c709fa7d6778f6ad7448df7a8cb77` (`main` after PR #256)
**Scope:** product-channel, reverse-trigger, binding-callback and sentinel/security traces T19–T21

This source trace supplements
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and the active
[implementation register](IMPLEMENTATION_TRACE_REGISTER.md). It records load
facts that are neither ordinary dependencies nor TOC-listed Lua/XML units. It is
not a runtime-client emulator and does not claim an executed addon workflow.

## Verdict

The #255 trace addressed dependency aliases, selected execution context,
three-valued predicates and the separate ordered-unit handoff. Five additional
source-backed gaps remain before ordinary Project acquisition can be considered
complete:

1. `OnlyBetaAndPTR` requires a product channel distinct from flavor/game type;
2. `LoadWith` is a reverse load trigger, not a dependency;
3. a selected addon can have `Bindings*.xml` absent from its TOC and containing
   executable callback bodies; the reviewed donor supplies an automatic-selection
   model whose exact client parity remains unverified;
4. known lifecycle/persistence metadata (`DefaultState`,
   `SavedVariablesMachine`) is classified as an unknown blocking directive;
5. the pinned Blizzard source contains `Interface: 0` and `Secure: 1`, while the
   current parser hard-rejects zero and has no reviewed meaning for `Secure`.

The existing parser already supports account/per-character SavedVariables
**declarations** and never opens user SavedVariables data files. Preserve that
boundary. Extend declaration scope; do not add data ingestion or schema inference.

### Evidence boundary

Blizzard files below prove that directives/documents occur and show their source
locations. The reviewed Wowless loader supplies comparative product-gating,
reverse-trigger, binding-selection and callback-wrapper behavior. Framework
admission policy covers unsupported `Secure`, sentinel Interface and explicitly
selected source. State those boundaries directly; do not collapse them into a
client-authoritative runtime claim or add a generic proof/evidence DTO.

## Evidence traced

### 1. Product channel is not client flavor

The pinned Blizzard source contains
[`Blizzard_PTRFeedback.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PTRFeedback/Blizzard_PTRFeedback.toc)
with `## OnlyBetaAndPTR: 1`.

Current `TocLoadContext` contains game-type matches, family, game, text locale,
game/glue location and global/secure environment. It has no stable/PTR/beta
product channel. `toc.rs` does not recognize `OnlyBetaAndPTR`, so the directive
becomes `UnknownDirective`; every load issue except unresolved optional dependency
blocks completeness.

The pinned Wowless loader independently models this as a package-loadability gate
against its product agent (`wow_ptr`/`wow_beta`). This is useful comparison
evidence, not proof of exact client parity.

**Decision:** add an explicit product channel to the ordinary load target, for
example `Stable | Ptr | Beta`. It is separate from:

- retail/Mainline versus Classic flavor/game type;
- target Interface;
- game/glue location;
- global/secure execution context.

Adopt the reviewed donor rule for `OnlyBetaAndPTR: 1`: PTR/Beta includes,
Stable excludes and an omitted channel leaves the gate unresolved. Scope a
limitation to results that depend on unverified client parity; never infer channel
from repository branch, folder, Interface number or addon title.
Reject malformed values rather than treating them as false.

### 2. `LoadWith` is a trigger relation, not a prerequisite

The pinned Blizzard source contains
[`Blizzard_ProfessionsTemplates.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ProfessionsTemplates/Blizzard_ProfessionsTemplates.toc)
with:

```text
## LoadOnDemand: 1
## LoadWith: Blizzard_ProfessionsCrafting, Blizzard_ProfessionsCustomerOrders
```

Current Project metadata projection has required/optional dependencies and
`LoadOnDemand`, but no `LoadWith` relation. `toc.rs` classifies the directive as
unknown. The pinned Wowless comparison records reverse trigger edges and loads
the `LoadWith` package after the named trigger package completes loading. That
establishes the reviewed donor model, not every detail of client lifecycle timing.

`LoadWith` must not be folded into required or optional dependency closure:

- it does not prove that the trigger package is available before the `LoadWith`
  package's code;
- the trigger package does not gain access to symbols declared later by the
  `LoadWith` package merely because the relation exists;
- a missing/unavailable trigger does not invalidate explicitly selected source,
  but automatic runtime reachability is unresolved;
- cycles and duplicates require bounded explicit handling rather than recursive
  dependency semantics.

**Decision:** retain a typed source-located reverse load-trigger relation with
source package, named target, resolution state and declaration order. Use it only
for load/reachability queries and checks that request it.

### 3. Binding documents are implicit source with callback semantics

The pinned
[`Blizzard_PingUI.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Blizzard_PingUI.toc)
does not list a bindings file. The same directory contains
[`Bindings.xml`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Bindings.xml),
whose `<Binding>` bodies contain Lua and use the implicit `keystate` parameter.
The convention occurs in multiple Blizzard addon directories.

The pinned Wowless loader searches an exact finite `Bindings<suffix>.xml` list
for the selected product/flavor, loads the selected document after addon files,
and its XML evaluator compiles each body as `function(keystate) ... end`. This is
comparative donor behavior. Blizzard source independently proves that unlisted
binding documents and bodies using `keystate` exist, but the open source does not
establish the client's exact discovery precedence or load timing.

Current Project acquisition follows TOC-listed Lua/XML and nested XML references.
It has no automatic sibling bindings discovery. Its XML index recognizes inline
Lua only for `Ui/Script` and `Ui/.../Scripts/*`; a root `<Bindings>` document and
its `<Binding>` bodies do not become `XmlInlineLua` units.

**Decision:** add a bounded exact bindings adapter using the existing safe root,
quick-xml parser and source maps:

1. derive a finite candidate list from selected addon directory and explicit
   flavor/product suffix rules;
2. select at most one exact candidate by documented precedence; reject case-fold
   collisions and same-precedence ambiguity;
3. treat absence as normal;
4. capture through `ProjectInputDirectory`/`ProjectInputFile` bounds;
5. parse `<Binding>` bodies as distinct mapped callback units with the reviewed
   source-and-donor-backed `keystate` parameter;
6. bind each unit to addon/package identity, effective environment, source span
   and selected bindings document identity;
7. classify malformed selected bindings XML or unsupported constructs as located
   binding-analysis limitations while preserving independent addon files;
8. never rewrite XML or expose synthetic wrapper locations.

A binding body is not a top-level TOC chunk and not the external addon chunk. It
must not be inserted into top-level TOC execution order or receive `(addonName,
privateTable)` ellipsis semantics. Model a separate callback phase. The donor's
after-files ordering is comparison evidence only; without client-authoritative
loader evidence it cannot support a negative availability or exact timing claim.
Static analysis also cannot prove that the binding ever fires.

### 4. Known metadata must not create false global partial status

The pinned Blizzard source uses `## DefaultState: enabled` broadly, including
[`Blizzard_AddOnList.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_AddOnList/Blizzard_AddOnList.toc).
That file also uses `## SavedVariablesMachine: g_addonCategoriesCollapsed`.

Current `toc.rs` recognizes only `SavedVariables` and
`SavedVariablesPerCharacter`; `saved_variables.rs` has only `Account` and
`Character` scopes. Both `DefaultState` and `SavedVariablesMachine` become
`UnknownDirective`, and `Loader::issue` marks every such issue blocking.

**Decision:**

- recognize `DefaultState` as bounded lifecycle metadata. For an explicitly
  selected root it does not suppress static analysis and must not make unrelated
  Lua/type checking partial;
- add `Machine` SavedVariables declaration scope and parse
  `SavedVariablesMachine` under the existing identifier/source/bounds rules;
- continue to store declarations only. Never open account, character or machine
  SavedVariables content, infer schemas, or initialize variables as non-nil;
- distinguish known nonblocking lifecycle/persistence metadata from unknown
  directives that can alter source selection, environment, namespace, order or
  callback availability.

### 5. Official `Interface: 0` and unresolved `Secure: 1`

The pinned Blizzard source contains
[`Blizzard_ReforgingUI_Classic.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_ReforgingUI/Blizzard_ReforgingUI_Classic.toc):

```text
## Interface: 0
## Secure: 1
## LoadOnDemand: 1
## AllowLoadGameType: classic
```

Current `toc.rs` rejects every Interface value equal to zero before useful source
records can reach analysis, and it treats `Secure` as an unknown blocking
directive. The pinned Wowless TOC parser retains numeric Interface values but the
reviewed loader path does not establish a distinct `Secure` interpretation. No
reviewed current Project or donor source proves that `Secure: 1` is equivalent to
`UseSecureEnvironment: 1`.

Two conclusions are source-supported:

1. zero is present in official selected source, so hard-coding “positive only” as
   syntactic validity is incorrect for general Blizzard-source admission;
2. `Secure` is potentially context-changing and cannot be downgraded to cosmetic
   metadata or guessed to mean another directive.

**Decision:**

- parse and retain `Interface: 0` as an explicit sentinel/unknown compatibility
  claim rather than a malformed integer. It never matches a positive target and
  cannot prove compatibility. For explicitly selected source, preserve
  independent analysis and emit a scoped compatibility limitation. Automatic TOC
  discovery must not prefer it over a positive compatible candidate without a
  documented product rule;
- retain `Secure` as a typed source-backed directive with exact raw value/span and
  an **unsupported semantic state** until a reviewed source establishes its
  meaning for the selected product. Do not alias it to `UseSecureEnvironment`,
  `LoadIntoEnvironment`, signing, taint or private-table policy by name;
- any selected unit whose required context depends on unresolved `Secure`
  semantics remains explicitly limited. Independent syntax/type diagnostics
  survive;
- update the validated Project context identity when either Interface sentinel or
  `Secure` metadata changes.

This is intentionally conservative: preserving unknown semantics is safer than
fabricating the wrong environment and later rebuilding every context-sensitive
check.

### 6. Already supported or not established as a defect

- `SavedVariables` and `SavedVariablesPerCharacter` declaration parsing, source
  records and identifier validation already exist. Only machine scope is missing.
- `AllowLoadTextLocale` already has a condition kind and explicit locale input.
  No occurrence was found by the reviewed pinned-source search; retain it and do
  not claim runtime parity from absence in one revision.
- `LoadOnDemand` already participates in package phase projection. Its interaction
  with triggers/bootstrap/explicit selection still needs observation; do not
  recreate the phase enum.
- Binding discovery/body extraction is absent; this does not invalidate existing
  ordinary UI XML maps.
- Exact searches of the reviewed source did not find `RequiredFaction`,
  `RequiredLocales`, or `RequiredGameAccount*`. This is not a permanent claim
  about all products/revisions; unknown future context-changing directives remain
  located limitations.

## Required owner changes

### `wow-project`

- extend the ordinary load target with explicit product channel;
- add typed source-located reverse trigger and known lifecycle metadata;
- retain official zero Interface as unknown compatibility, not malformed input;
- retain unresolved `Secure` metadata without guessing semantics;
- add `Machine` SavedVariables declaration scope;
- discover at most one selected `Bindings*.xml` under the held addon root;
- extract exact mapped binding callbacks and context without another XML parser;
- include binding/channel/trigger/Interface/security facts in validated Project
  context and rebuild identity.

### `wow-emmy`

- reuse `ConfiguredAnalyzerSession` and existing virtual registration;
- register mapped binding callbacks through a narrow `keystate` adapter, not
  external-chunk ellipsis replacement;
- keep synthetic wrappers/context facts out of user output/API provenance;
- do not invent behavior for unresolved `Secure` metadata.

### `wow-service`

- carry channel/trigger/binding/compatibility/security limitations separately
  from language findings;
- preserve useful type diagnostics when automatic reachability or callback/
  security context is unavailable;
- selected checks request only facts they consume.

### CLI/frontends

- expose product channel where necessary; a normal explicitly selected installed
  client may use a safe Stable default;
- render excluded/unresolved channel, zero-Interface compatibility, binding and
  security-context limitations without internal receipt identities;
- use the same exact Project context from CLI, LSP and MCP.

## Verification boundary

This review follows source producers/readers and compares pinned Blizzard source
with the pinned Wowless donor. No loader, addon check, editor, key binding or game
client was run.

Before T19–T21 become observed, record ordinary commands/protocol inputs that
cover:

1. Stable exclusion and PTR/Beta inclusion for `OnlyBetaAndPTR`;
2. a `LoadWith` reverse trigger without dependency semantics;
3. unlisted `Bindings.xml` producing mapped diagnostics with `keystate`;
4. no-bindings absence and malformed bindings preserving independent findings;
5. `DefaultState`/`SavedVariablesMachine` without false global partial or user
   data access;
6. explicit official `Interface: 0` source preserving analysis but reporting
   unknown compatibility;
7. `Secure: 1` producing a scoped unsupported-context limitation rather than a
   guessed environment or generic unknown directive.
