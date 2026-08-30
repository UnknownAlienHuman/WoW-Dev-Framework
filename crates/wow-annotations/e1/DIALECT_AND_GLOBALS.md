# WoW dialect and global-library projection

**Status:** normative E1-C profile-bound dialect, standard/global symbol, environment, and analysis-library contract.

A Lua 5.1 label is insufficient to describe the World of Warcraft analysis environment. The generated dialect library is an exact profile-bound projection and does not mutate editor globals/settings.

## 1. Inputs

```text
exact ReferenceProfile / ReferenceGeneration
ReferenceView exact global/dialect/reference facts
versioned WowDialectProfile source contract
consumer capability profiles
restriction/source evidence/coverage
```

No discovery from local WoW installation, editor, installed addons, user settings, `_G` runtime dump, or oracle output.

## 2. Dialect profile fields

```text
WowDialectProfile
    profile_id/version
    exact flavor/Interface/build/source/reference applicability
    base Lua language assumptions
    allowed standard globals
    removed/unavailable standard globals
    Blizzard globals/namespaces
    require-like functions/symbols
    nonstandard syntax/symbol conventions if any
    secure/restricted/private environment globals
    global type/signature/reference links
    source/evidence/coverage
    consumer-specific projection requirements
    canonical digest
```

Unknown/missing fields remain partial; no fallback to generic editor defaults as truth.

## 3. Standard library policy

For each standard/global symbol record:

```text
symbol name/kind
available | unavailable | restricted | unknown
signature/type
profile/flavor applicability
source/reference evidence
projection status
```

Do not emit a standard function merely because ordinary Lua 5.1/LuaLS includes it. Do not remove one merely because an external pack omits it. Exact selected profile governs.

## 4. Blizzard globals and namespaces

Project exact global API systems/namespaces/types through:

```text
namespace/system declarations
function/method stubs
global variables/constants/types
consumer-neutral ownership/reference links
```

No runtime values/bodies. Global declaration cannot be inferred from third-party addon references.

## 5. Require-like and loading symbols

Some WoW/addon code uses nonstandard loading/module patterns. E1-C emits only exact reviewed dialect/library declarations supplied by the selected ReferenceData/dialect profile.

Do not make `require` available globally or configure module path unless exact profile contract says so. Project-specific `LibStub`/framework factories belong to project/dependency libraries, not universal WoW dialect by default.

## 6. Secure/restricted/private globals

Dialect projection can declare names/types and sidecar restrictions only when exact source facets exist. It cannot imply accessibility in all contexts.

```text
known restricted global
    declaration + exact restriction sidecar/projection status

unknown/conditional/private global
    sidecar/loss/NotEvaluated as profile defines
```

No use of types to declassify runtime Secret values.

## 7. Nominal Secret analysis module

A dedicated versioned module may declare:

```lua
---@class WowSecretValue
---@class WowSecretNumber: WowSecretValue
---@class WowSecretBoolean: WowSecretValue
---@class WowSecretString: WowSecretValue
```

Exact active declarations freeze after consumer probes.

Rules:

- analysis-only documentation prominent;
- no constructors, runtime methods, unwrap/access helpers, operators, or declassification semantics;
- categories emitted only when source/type lowering needs and consumers handle them;
- no claim runtime values are Lua objects;
- no permanent spell-specific declarations/whitelist;
- unknown facets remain sidecar/loss.

## 8. Access predicates and helpers

Exact known functions such as access predicates can be projected through ordinary exact callable facts from ReferenceView. The dialect module does not invent semantics or special analyzer behavior unless a separately tested type/lint contract supports it.

A declared predicate signature alone does not prove a guard dominates or that all runtime contexts permit access; that belongs to rules/analyzer flow.

## 9. Removed/deprecated globals

Do not emit an unavailable global into the active profile library merely to support historical code. Historical/deprecated/transition records can appear as explicit docs/sidecars or separate selected historical artifact, but profiles do not mix.

Consumer diagnostics for removed symbols are expected; do not suppress them.

## 10. Multi-flavor/profile separation

```text
Retail live artifact
PTR/beta artifact
Classic/flavor artifacts
historical artifacts
```

Physically/logically separate manifests and directories or exact profile roots. No union library combining incompatible signatures/globals/restrictions.

## 11. Project/dependency libraries

The dialect artifact contains platform-wide profile facts only. It does not include:

```text
arbitrary installed addons
Ace3/oUF/other framework declarations unless separately project-declared
project globals/SavedVariables
full Blizzard implementation source
user workspace libraries
```

`wow-project`/service assembles project-specific libraries later.

## 12. Consumer differences

EmmyLua and LuaLS may differ in:

```text
global/environment tags
module/namespace table inference
class inheritance/alias handling
unknown/any/nil behavior
special standard-library assumptions
```

Consumer profiles define exact emitted forms. Shared dialect artifact only if mandatory semantics verified for both; otherwise separate consumer artifacts/profile IDs.

## 13. Editor/config boundary

E1-C never:

```text
edits .vscode/settings.json
changes Lua.workspace.library or diagnostics globals
installs/enables extensions
adds global names through user config
turns off undefined-global/weak-union diagnostics
writes emmyrc/lua-language-server config
```

A higher application can produce a separate explicit generated config pointing to artifact files, under the service/app contract, without mutating user config.

## 14. Source mapping and coverage

Every dialect/global declaration links exact ReferenceView/dialect source/evidence/coverage. Profile-level assumptions with no exact source must be explicitly curated/versioned evidence or remain unsupported.

Coverage partitions:

```text
dialect.standard_globals
dialect.blizzard_globals
dialect.require_like
dialect.nonstandard_symbols
dialect.restricted_globals
dialect.nominal_secret_types
consumer.<id>.dialect_projection
```

## 15. Determinism

Equivalent profile/reference/dialect/consumer profiles yield identical semantic declarations/files/maps/loss. No dependence on editor/global scan, installed addons, environment variables, local client path, platform locale, or time.

## 16. Required operations

```text
build_wow_dialect_profile
validate_wow_dialect_profile
select_standard_global_declarations
select_blizzard_global_and_namespace_declarations
project_require_like_and_nonstandard_symbols
project_restricted_secure_private_globals
build_nominal_secret_analysis_module
build_dialect_projection_status_and_loss
validate_dialect_consumer_compatibility
```

## 17. Required tests

- allowed/removed/unknown standard globals;
- exact Blizzard namespace/global signatures;
- require-like absent/present profile cases;
- restricted/private/conditional global sidecars;
- nominal Secret classes no runtime/declassification methods;
- access predicate ordinary declaration without flow claim;
- historical/profile separation;
- installed addon/editor/user config cannot affect output;
- EmmyLua/LuaLS dialect differences;
- no config/diagnostic mutation;
- deterministic artifact across hosts/locales/environments.

## 18. Hard stops

- no generic Lua/editor defaults as platform truth;
- no installed addon/project globals in platform library;
- no profile/flavor union;
- no editor settings/globals mutation;
- no diagnostic suppression;
- no runtime wrapper/declassification/whitelist claims;
- no full Blizzard source injection;
- no environment/local-client discovery.
