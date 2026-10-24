# TOC trigger, product-channel and binding-callback trace

**Checked:** 2026-10-24  
**Source revision:** `337b9b6e676af6735f416e8527176ed17e865172`  
**Scope:** post-#255 completion of the ordinary TOC/load-context audit

This source trace supplements
[TOC_LOAD_CONTEXT_AUDIT.md](TOC_LOAD_CONTEXT_AUDIT.md) and the active
[implementation register](IMPLEMENTATION_TRACE_REGISTER.md). It records load
facts that are neither ordinary dependencies nor TOC-listed Lua/XML units. It is
not a runtime-client emulator and does not claim an executed addon workflow.

## Verdict

The #255 trace closed dependency aliases, secure/private-table context,
three-valued predicates and the separate ordered-unit handoff. Four additional
source-backed gaps remain before ordinary Project acquisition can be considered
complete:

1. `OnlyBetaAndPTR` requires a product channel distinct from flavor/game type;
2. `LoadWith` is a reverse load trigger, not a dependency;
3. a selected addon can have an automatically discovered `Bindings*.xml` that is
   absent from its TOC and contains executable callback bodies;
4. known lifecycle/persistence metadata (`DefaultState`,
   `SavedVariablesMachine`) is currently classified as an unknown directive and
   therefore makes the load plan falsely incomplete.

The existing parser already supports account/per-character SavedVariables
**declarations** and never opens user SavedVariables data files. Preserve that
boundary. Extend the declaration scope; do not add data ingestion or schema
inference.

## Evidence traced

### 1. Product channel is not client flavor

The pinned Blizzard source contains
[`Blizzard_PTRFeedback.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PTRFeedback/Blizzard_PTRFeedback.toc)
with `## OnlyBetaAndPTR: 1`.

Current `TocLoadContext` contains game-type matches, family, game, text locale,
game/glue location and global/secure environment. It has no stable/PTR/beta
product channel. `toc.rs` does not recognize `OnlyBetaAndPTR`, so the directive
becomes `UnknownDirective`; every issue except an unresolved optional dependency
blocks completeness.

The pinned Wowless loader independently models this as a package loadability
gate against its product agent (`wow_ptr`/`wow_beta`). This is useful comparison
evidence, not proof that Wowless exactly reproduces the client.

**Decision:** add an explicit product channel to the ordinary load target, for
example `Stable | Ptr | Beta`. It is separate from:

- retail/Mainline versus Classic flavor/game type;
- target Interface;
- game/glue location;
- global/secure execution context.

For `OnlyBetaAndPTR: 1`, PTR/Beta includes and Stable excludes. An omitted channel
makes the package gate unresolved; never infer it from a repository branch,
folder name, Interface number or addon title. Reject malformed values rather than
treating them as false.

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
the `LoadWith` package after the named trigger package completes loading.

`LoadWith` therefore must not be folded into required or optional dependency
closure:

- it does not prove that the trigger package is available before the `LoadWith`
  package's code;
- the trigger package does not gain access to symbols declared later by the
  `LoadWith` package merely because the relation exists;
- a missing or unavailable trigger does not make the package's source invalid for
  explicit static analysis, but its automatic runtime reachability is unresolved;
- cycles and duplicate trigger declarations need explicit bounded handling rather
  than recursive dependency semantics.

**Decision:** retain a typed, source-located reverse load-trigger relation with
source package, named target, resolution state and exact declaration order. For
an explicitly selected addon root, analyze the selected source while preserving
whether automatic runtime loading is unavailable, conditional or unresolved. For
a full selected package universe, use the relation only for load/reachability
queries and checks that request it.

### 3. Binding documents are implicit source with callback semantics

The pinned
[`Blizzard_PingUI.toc`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Blizzard_PingUI.toc)
does not list a bindings file. The same directory contains
[`Bindings.xml`](https://github.com/Gethe/wow-ui-source/blob/e7aa82019dc298e0351c7159d60c2a9b916b9b67/Interface/AddOns/Blizzard_PingUI/Bindings.xml),
whose `<Binding>` bodies contain Lua and use the implicit `keystate` parameter.
The same convention occurs in multiple Blizzard addon directories.

The pinned Wowless loader searches an exact finite `Bindings<suffix>.xml` list
for the selected product/flavor, loads the selected document after addon files,
and its XML evaluator compiles each body as `function(keystate) ... end`.
This is donor behavior to compare and port narrowly; it is not a reason to execute
addon source.

Current Project acquisition follows TOC-listed Lua/XML and nested XML references.
It has no automatic sibling bindings discovery. Its XML index recognizes inline
Lua only for `Ui/Script` and `Ui/.../Scripts/*`; a root `<Bindings>` document and
its `<Binding>` bodies do not become `XmlInlineLua` units.

**Decision:** add a bounded, exact bindings-document adapter using the existing
safe root and quick-xml/source-map owners:

1. derive a finite candidate list from the selected addon directory and the same
   explicit flavor/product suffix rules used by TOC selection;
2. select at most one exact candidate by documented precedence; reject case-fold
   collisions or ambiguous same-precedence candidates;
3. absence is normal and creates no limitation;
4. capture the selected file through `ProjectInputDirectory`/
   `ProjectInputFile` bounds and identity checks;
5. parse `<Binding>` bodies as distinct mapped callback units with only the
   source-verified implicit `keystate` parameter;
6. bind each unit to addon/package identity, effective execution environment,
   source span and selected bindings document identity;
7. classify malformed selected bindings XML or unsupported binding constructs as
   located limitations of binding analysis, preserving independent addon files;
8. never rewrite user XML or expose synthetic wrapper locations.

A binding body is not a top-level TOC chunk and not the external addon chunk. It
becomes callable after the addon's selected files are loaded and executes on a
binding event. Represent this as a callback/load-phase fact; do not insert its
body into the top-level TOC order or give it `(addonName, privateTable)` ellipsis
semantics. Its lexical context may see addon globals according to the same exact
selected project, but availability is not proof that the binding ever fires.

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
  selected addon root it does not suppress static analysis and must not make
  unrelated Lua/type checking partial. Retain the normalized source value and,
  where needed by a full package-universe query, a typed enablement-default state;
- add `Machine` to the SavedVariables declaration scope and parse
  `SavedVariablesMachine` with the same identifier/source/bounds rules;
- continue to store declarations only. Never open account, character or machine
  SavedVariables content, infer schemas from user data, or initialize variables
  as non-nil;
- distinguish known nonblocking lifecycle/persistence metadata from unknown
  directives that can alter source selection, environment, namespace, order or
  callback availability.

### 5. Already supported or not established as a defect

- `SavedVariables` and `SavedVariablesPerCharacter` declaration parsing, source
  records and identifier validation already exist. Only the machine scope is
  missing.
- `AllowLoadTextLocale` already has a condition kind and explicit locale input.
  No occurrence was found in the reviewed pinned Blizzard source search; do not
  remove it or claim runtime parity from absence in one revision.
- `LoadOnDemand` already participates in package phase projection. Its interaction
  with `LoadWith`, bootstrap and explicit selection remains to be validated; do
  not recreate the phase enum.
- Bindings discovery/body extraction is absent; this is not evidence that the
  existing ordinary UI XML source maps are wrong.

## Required owner changes

### `wow-project`

- extend the small ordinary load target with explicit product channel;
- add typed, source-located package trigger and known lifecycle metadata;
- add `Machine` SavedVariables declaration scope;
- discover at most one selected `Bindings*.xml` under the held addon root;
- extract exact mapped binding callback units and their context without a second
  XML parser;
- include binding document/unit identities and trigger/channel facts in the
  validated Project context handoff and rebuild identity.

### `wow-emmy`

- reuse `ConfiguredAnalyzerSession` and existing virtual-source registration;
- register mapped binding callback units through a narrow context adapter with
  `keystate`, not through external-chunk ellipsis replacement;
- keep synthetic wrapper diagnostics/context facts out of user output and WoW API
  provenance.

### `wow-service`

- carry channel/trigger/binding limitations separately from language findings;
- type-only checks may still report useful independent diagnostics when automatic
  reachability or binding context is unavailable;
- selected checks request only the trigger/binding facts they consume.

### CLI/frontends

- expose product channel only where necessary and keep a safe ordinary Stable
  default when the user explicitly selects the normal installed client target;
- render excluded/unresolved channel and binding limitations without exposing
  internal receipt identities;
- use the same exact Project context from CLI, LSP and MCP.

## Verification boundary

This review follows source producers/readers and compares pinned Blizzard source
with the pinned Wowless donor. No loader, addon check, editor, key binding or game
client was run.

Before T19/T20 can become observed, record ordinary commands/protocol inputs that
cover:

1. Stable exclusion and PTR/Beta inclusion for `OnlyBetaAndPTR`;
2. an explicit `LoadWith` package and its reverse trigger without treating it as
   a dependency;
3. an addon whose unlisted `Bindings.xml` produces mapped Lua diagnostics with
   the `keystate` parameter;
4. an addon without bindings, which remains complete for that feature;
5. malformed bindings XML preserving independent addon diagnostics;
6. `DefaultState` and `SavedVariablesMachine` without a false global partial;
7. a SavedVariables reference that remains nullable until user code initializes
   it, with no user data file access.
