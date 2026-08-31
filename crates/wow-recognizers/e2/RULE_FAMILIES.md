# E2-B core rule families

**Status:** normative active-rule inventory and proof boundaries.

Every rule emits universal graph roles/relations only. Literal API/member names below are exact reviewed structural conventions in the frozen fact profile; they are not a platform-version verdict.

## 1. TOC/package family

### `core.toc.package@1`

Inputs: exact `TocPackageFact` and manifest source evidence.

Outputs:

- `addon_package` entity proposal;
- `toc_manifest`/`toc_variant` entity proposals;
- `contains`/`defines` relations according to graph registry.

No directory-name inference.

### `core.toc.file_order@1`

Inputs: ordered `TocFileFact` records under one exact variant.

Outputs:

- file entities when supplied by project facts;
- `loads` and `loads_before` relations for exact manifest order.

Does not infer runtime frame creation or conditional load success.

### `core.toc.dependencies@1`

Inputs: `TocDependencyFact`.

Outputs:

- `depends_on` or `optional_depends_on` relation;
- unresolved target remains Possible/coverage blocker.

### `core.toc.load_on_demand@1`

Records exact LOD/bootstrap metadata/roles only. It must not assert that full addon state or frames exist at `ADDON_LOADED`.

### `core.toc.saved_variables@1`

Emits `state_root` entity proposals and package ownership/declaration relations from exact TOC declarations. Account/character scope remains an attribute.

## 2. XML family

### `core.xml.template@1`

Inputs: exact `XmlTemplateFact`.

Outputs `xml_template` entity and package/file ownership.

### `core.xml.object@1`

Inputs: `XmlObjectFact`.

Outputs frame/region/object entity, definitions, ownership, and explicit object parent relation.

### `core.xml.inherits@1`

Inputs: resolved `XmlInheritanceFact`.

Outputs `inherits`/`references_template`; unresolved or multiple targets remain Possible/ambiguous.

### `core.xml.script@1`

Inputs: `XmlScriptFact` with exact object/template and handler identity.

Outputs `sets_script` plus ownership/handler relation. Append/prepend/inherited semantics remain explicit attributes. Embedded code is never executed or reparsed.

## 3. Frame/mixin construction family

### `core.lua.create_frame@1`

Match exact resolved `CreateFrame` call with ordered arguments.

Potential outputs:

- `factory`/frame entity proposal;
- `created_by` relation from frame/object to call/function/factory as registry defines;
- `parent_of` only for an exact resolved parent and object-axis direction;
- `references_template` for exact literal/resolved template arguments;
- name/type/template attributes only when exact.

Dynamic name, parent, or template produces Possible/ambiguity. Variable names never substitute for arguments.

### `core.lua.create_from_mixins@1`

Match exact resolved `CreateFromMixins` call. Emit instance entity/`instantiates` and `mixes_in` proposals for exact resolved mixins. Dynamic argument lists remain Possible.

### `core.lua.mixin_assignment@1`

Match exact resolved `Mixin(target, ...)` structure. Emit `mixes_in` proposals when target/mixins resolve. It does not claim inheritance, method availability, or runtime safety beyond supplied facts.

## 4. Signal/event family

The rules preserve the current structural distinction required by the engineering KB.

### `core.signal.native_frame_event@1`

Match exact method calls equivalent to:

```text
Frame:RegisterEvent
Frame:RegisterUnitEvent
```

Outputs:

- event entity/reference proposal from exact literal/resolved event key;
- `registers_event` relation from exact frame/owner/handler context;
- unit tokens as ordered attributes when exact.

No table-overload assumption. Secret-capable payload accessibility is outside this rule.

### `core.signal.native_event_registry_bridge@1`

Match exact `EventRegistry:RegisterFrameEventAndCallback*` family. Emit a distinct native-frame-event bridge subscription and handle/owner relations when exact.

It is not a plain custom callback.

### `core.signal.custom_registry_producer@1`

Match exact `EventRegistry:TriggerEvent` with exact event key. Emit a custom signal producer relation.

### `core.signal.custom_registry_subscription@1`

Match exact `EventRegistry:RegisterCallback` with exact event key and callback/owner.

- If a compatible exact producer exists in the declared closed scope, emit `subscribes_callback`/producer-consumer relations as `Derived`.
- If producer lookup is incomplete, absent under partial coverage, dynamic, or ambiguous, emit only an unresolved/Possible subscription proposal and explicit blocker.
- Never reinterpret `PLAYER_LOGIN`, `UNIT_AURA`, or another plausible native name as a custom producer without `TriggerEvent` evidence.

### `core.signal.cvar_callback@1`

Match exact `CVarCallbackRegistry:RegisterCallback`. Emit a distinct CVar subscription relation with exact CVar key/callback/owner when known.

## 5. Script/hook family

### `core.hook.set_script@1`

Match exact `SetScript` call. Emit `sets_script` relation with object, script name, handler, and ownership evidence.

### `core.hook.hook_script@1`

Match exact `HookScript` call. Emit `hooks` relation with `hook_kind=script_posthook` (or frozen exact semantic kind). It does not claim protected/forbidden/taint safety.

### `core.hook.secure_posthook@1`

Match exact `hooksecurefunc` forms:

```text
hooksecurefunc(globalName, callback)
hooksecurefunc(table, methodName, callback)
```

Emit `hooks` relation for exact target/callback; dynamic target is Possible. Do not recognize global assignment/override as a safe hook. No combat/taint guarantee.

## 6. Library family

### `core.library.libstub_require@1`

Match exact resolved `LibStub("Name-Version")` or reviewed `GetLibrary` form.

Outputs library entity proposal and `requires_library` relation. Version string is exact evidence, not proof of loaded revision or ownership.

### `core.library.libstub_new@1`

Match exact reviewed `LibStub:NewLibrary` form. Emit library declaration/creation relation with version when exact. It does not infer upstream repository or license.

### `core.library.embed@1`

Match exact structural embed call only when receiver/library/target are resolved and the pack declares the convention. Emit `embeds_library`; otherwise Possible or no match.

Folder names such as `Libs/` do not prove an embed relation.

## 7. State family

### `core.state.saved_variable_root@1`

Consumes `TocSavedVariableFact` and emits exact `state_root` identity. Lua references can link to the root only when symbol resolution matches the declared root in the same selected project/variant.

### `core.state.literal_path_read@1`

Match field/index chains rooted at an exact SavedVariables root with literal keys. Emit `state_path` entity and `reads_state` relation from exact function/source owner.

### `core.state.literal_path_write@1`

Same for assignments/writes; emit `writes_state`.

Dynamic key behavior:

- retain exact resolved prefix/root;
- mark unresolved suffix/whole path Possible;
- do not create an exact path from source text or runtime examples.

## 8. Explicitly deferred rule families

```text
AceAddon/oUF/WeakAuras/BigWigs/Details/Plater-specific factories
OnInitialize/OnEnable/OnDisable lifecycle role heuristics
plugin/region/style/element registries
slash-command and arbitrary message-bus patterns
Secret guards and unsafe sinks as graph roles
repository-specific module/service naming
complete Blizzard UI framework recognizers
runtime-discovered relations
```

They require E5 calibration packs, multiple pinned corpora, mutation evidence, and universal graph outputs. Diagnostic Secret legality remains `wow-rules`, not recognizers.

## 9. Per-rule mandatory declaration

Each active rule freezes:

```text
rule ID/version
exact fact/graph schema profiles
required capabilities and closed scope
allowed convention literals and why
clause/capture/output schema
Derived/Possible proof conditions
ambiguity and no-match policy
positive/near-negative/partial/dynamic fixtures
rename/path/name/literal mutation behavior
output/complexity budgets
producer partition identity
```
