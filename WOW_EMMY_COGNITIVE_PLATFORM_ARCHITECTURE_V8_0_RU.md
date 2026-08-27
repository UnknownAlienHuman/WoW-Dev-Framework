# WoW Emmy Cognitive Code Intelligence Platform
## Архитектура и план реализации v8.0

**Статус:** заменяет v7.0 и предыдущие противоречивые варианты.  
**Дата проверки источников:** 2026-08-15.  
**Автор проекта:** Neomorph.  
**Основной язык реализации:** Rust.  
**Назначение:** актуальная WoW-aware среда анализа, поиска, графа и агентной разработки аддонов для Retail/Midnight.

---

# 0. Окончательное решение

Мы реализуем не очередной справочник и не ещё один универсальный RAG. Нужны три тесно связанные, но раздельные части:

```text
wow-emmy-ls
    Rust host/companion вокруг emmylua_code_analysis;
    generic Emmy diagnostics + WoW diagnostics в одном project generation.

wow-index
    точный API/UI/project index;
    TOC/XML/load graph;
    универсальный WoW domain graph;
    поиск, lineage, skeletons, Secret facets.

wow-cbm-bridge
    optional bridge к установленному Codebase Memory;
    CBM даёт broad semantic source discovery;
    wow-index подтверждает WoW relations и contracts.
```

Формула данных:

```text
Blizzard UI snapshot
    ├─ raw APIDocumentation catalog
    ├─ Ketho-compatible annotation pack
    ├─ FrameXML/UI graph
    ├─ Secret/restriction facets
    └─ build lineage

addon repository
    ├─ Lua syntax/semantic facts
    ├─ TOC/XML/load facts
    ├─ framework conventions
    ├─ project graph
    └─ generated ProjectMap

CBM
    └─ broad semantic/source candidates

all three
    → exact search
    → trees/skeletons
    → diagnostics
    → impact analysis
    → compact agent context.
```

## 0.1. Принципиальные решения

1. **EmmyLua не форкается.** Мы закрепляем upstream commit и используем публичный Rust crate `emmylua_code_analysis`.
2. **Плагинного API внешних diagnostics в текущем Emmy source не найдено.** Поэтому чистая реализация v1 — собственный Rust host `wow-emmy-ls`, вызывающий Emmy diagnostics и затем WoW providers. Параллельно готовится небольшой upstream PR с compile-time `DiagnosticProvider` trait.
3. **Ketho используется как проектный образец, формат совместимости и differential oracle.** Настройки VS Code/LuaLS и автоматическое подавление diagnostics не копируются.
4. **Numy FramexmlAnnotations используется как differential oracle и corpus conventions.** Его PHP/regex implementation не становится production dependency.
5. **Codebase Memory не модифицируется через его SQLite.** В проверенном source нет публичного runtime ABI для произвольных domain nodes/edges. Поэтому v1 хранит небольшой WoW graph самостоятельно и соединяет ответы по стабильным source handles.
6. **Никаких правил вида `if addon == ElvUI`.** Именованные framework packs содержат декларативные recognizers универсальных ролей.
7. **Один компактный storage substrate:** immutable `reference.sqlite`, mutable `project.sqlite` в WAL и bounded `petgraph` subgraphs в памяти.
8. **Exact/migration/lineage precede fuzzy search.** Текстовая похожесть никогда не объявляется доказанной заменой.
9. **Secret metadata не теряется в annotations.** Raw Blizzard metadata и generated annotations — разные projections.
10. **Любой человек получает одинаковую систему.** Нет зависимости от конкретного ПК, VS Code profile, личных карточек или заранее установленных аддонов.

---

# 1. Привязка к десяти требованиям

| Требование | Реализация v8.0 |
|---|---|
| 1. Rust-дополнение EmmyLua по принципу Ketho | `wow-emmy-ls`, `wow-reference-builder`, Ketho-compatible annotations, WoW diagnostic providers, compat probe |
| 2. CBM WoW graph или собственный graph | optional CBM query bridge + собственный SQLite domain graph; upstream ExternalFacts proposal |
| 3. Blizzard UI и популярные аддоны, универсальные связи | UiGraph, multi-parent model, declarative recognizers, history/lineage, corpus Ace/oUF/ElvUI/WA/BigWigs/Details/Plater |
| 4. Качественный поиск | exact → alias/deprecation/lineage → shape → prefix/fuzzy → FTS5 → graph → CBM semantic fallback |
| 5. Reuse-first | Emmy, Ketho parity, Numy parity, CBM, SQLite, quick-xml, official gh/GitHub MCP; собственные bridges и domain logic |
| 6. Secret и multi-repository indexing | raw restriction facets, Secret-local lint, project/reference/external corpora |
| 7. Agent package и GitHub | `AGENTS.md`, official `gh`, official GitHub MCP read-only, managed external corpus workflow |
| 8. Маленькая БД | SQLite B-tree + FTS5 + adjacency tables; no graph server/vector DB |
| 9. Другие игры | Roblox tracker/Luau LSP/Rojo, Yarn/Loom, FFXIVClientStructs/Dalamud lessons |
| 10. Старые документы | explicit recovery matrix in §19; strong decisions retained, obsolete complexity removed |

---

# 2. Что реально делают исследованные инструменты

## 2.1. Ketho/vscode-wow-api

Проверенный commit:

```text
d0b5b51fac4c52c493371b9b18e66ce604ea4326
2026-06-24
```

Его build pipeline:

```text
Blizzard UI snapshot
→ load Blizzard_APIDocumentation
→ load Blizzard_APIDocumentationGenerated in TOC order
→ intercept APIDocumentation:AddDocumentationTable
→ apply curated field patches
→ emit LuaCATS/LuaLS annotations
→ add Event/CVar/Enum/Wiki/FrameXML/resources projections.
```

Ценность для нас:

```text
annotation layout;
type lowering;
Ketho patches;
Event/CVar/Enum projections;
regression fixtures;
expected editor behavior.
```

Не переносится:

```text
VS Code extension requirement;
mutation of user/workspace settings;
auto-addition of globals;
diagnostic-triggered weakUnionCheck;
diagnostic suppression;
full FrameXML injection into editor library.
```

## 2.2. LuaLS

Проверенный commit:

```text
7a73c7889c1ec981dfd76fba38f5096379f62f99
2026-08-14
```

LuaLS остаётся:

```text
compatibility oracle;
second diagnostic baseline;
consumer of generated annotations;
fixture target for Ketho parity.
```

Проверенный plugin surface в первую очередь трансформирует source text через `OnSetText`; это не чистый API внешнего diagnostic provider. Поэтому LuaLS plugin не является основой WoW rule engine.

## 2.3. EmmyLua Analyzer Rust

Проверенный commit:

```text
aaaca68425d9362876228649b0b8d92f07654daa
2026-08-14
emmylua_code_analysis 0.25.1
emmylua_parser 0.29.1
```

Публичный Rust API уже позволяет:

```text
создать EmmyLuaAnalysis;
настроить Emmyrc;
добавить main/library workspaces;
обновлять файлы;
строить index;
получать FileId/URI;
запускать built-in diagnose_file;
получать semantic model через compilation.
```

Проблема текущего source:

```text
LuaDiagnostic::diagnose_file вызывает фиксированный внутренний check_file;
список built-in checkers hardcoded внутри crate;
публичной регистрации внешнего diagnostic provider не найдено.
```

Следствие:

```text
v1: собственный host вокруг публичной analysis library;
после built-in diagnose_file запускаются наши WoW providers;
результаты нормализуются и публикуются одной диагностической лентой.
```

Это не fork analyzer-а. Обновление Emmy означает изменение pinned dependency, запуск compat probe и при необходимости изменение одного adapter crate.

## 2.4. Tencent/LuaHelper

Проверенный latest commit:

```text
15c9fac58a73c6f780257c81664a03b52d941f9d
2024-11-04
```

Проект устарел как dependency, но полезен как donor:

```text
parallel per-file passes;
selective invalidation;
last-known-good cache;
разделение editor frontend и Go analysis server;
устойчивость при ошибке одного файла.
```

Мы не включаем LuaHelper в поставку и не копируем его protocol.

## 2.5. NumyAddon/FramexmlAnnotations

Проверенный commit:

```text
b38f5ec0f9fbf493e31c4f060b6f3db2ef743c78
2026-08-09
```

Проект автоматически строит FrameXML annotations для live/PTR/beta/classic и умеет смешивать annotations в source с сохранением line numbers. Он извлекает, среди прочего:

```text
global frames;
templates;
intrinsic templates;
limited inheritance/children/KeyValues;
mixins;
CreateFromMixins;
EnumUtil.MakeEnum;
methods found by Lua patterns.
```

Сам README предупреждает, что output не полностью точен. Поэтому:

```text
Numy = differential oracle + fixtures;
our XML/Lua structural parser = production truth;
unknown/incomplete inheritance = coverage gap, not guessed parent.
```

## 2.6. SpartanUI wow-api-mcp

Проверенный commit:

```text
81e96a304315bca2ac0ac1c064feaf97c323803e
2026-02-06
```

Он читает готовый Ketho extension, regex-парсит annotations и даёт хороший минимальный UX:

```text
lookup API;
search API;
deprecations;
namespace;
widget methods;
enum;
event payload.
```

Использование:

```text
query UX oracle;
parity tests;
not canonical raw metadata ingestion.
```

## 2.7. Codebase Memory MCP

Проверенный commit:

```text
e513beb487bea21105a031da328872c1b6f58eac
2026-08-15
```

CBM силён как:

```text
fast repository indexing;
BM25 and semantic source search;
generic definitions/calls;
architecture, trace and impact;
cross-repository discovery.
```

Но проверенный Lua language spec описывает в основном function declarations/definitions, calls and chunk; Lua imports extractor обрабатывает `require("X")`. Публичного runtime interface для импорта наших arbitrary TOC/XML/FrameXML/domain facts в проверенном source не найдено.

Вывод:

```text
CBM remains broad source graph;
WoW graph is a small sidecar DB;
bridge joins results by stable source identity;
optional upstream PR may add derived-facts import later.
```

---

# 3. `wow-emmy-ls`: чистое Rust-дополнение EmmyLua

## 3.1. Физическая форма

```text
wow-emmy-check
    batch checker for CLI/CI/agents.

wow-emmy-ls
    thin LSP host built on the same library.

wow-emmy-core
    shared project actor, Emmy adapter and diagnostic provider registry.
```

Один project actor владеет одной mutable `EmmyLuaAnalysis`. Читатели получают immutable `ProjectGeneration` snapshots.

## 3.2. Почему не VS Code extension

Extension-only решение снова привяжет correctness к editor state. Правильный слой:

```text
Rust binary + library
    → works from CLI, CI, Codex, Claude, OpenCode and any LSP client;

optional editor extension
    → only bootstrap/status/UI;
    → no canonical data and no hidden configuration mutation.
```

## 3.3. Public provider boundary

```rust
pub trait WowDiagnosticProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn required_capabilities(&self) -> CapabilitySet;
    fn check(&self, ctx: &WowCheckContext<'_>, out: &mut Vec<WowFinding>);
}

pub struct WowCheckContext<'a> {
    pub file_id: FileId,
    pub source_path: &'a Path,
    pub source_text: &'a str,
    pub emmy: &'a EmmyLuaAnalysis,
    pub reference: &'a dyn ReferenceView,
    pub project: &'a ProjectSnapshot,
}
```

Providers never mutate Emmy state. They produce findings with source spans, evidence and coverage.

## 3.4. Diagnostic execution

```text
file update
→ update Emmy VFS/index
→ built-in Emmy diagnose_file
→ produce normalized generic diagnostics
→ get exact syntax/semantic facts for changed file
→ run enabled WoW providers
→ canonical root-cause merge
→ publish diagnostics for one generation.
```

First providers:

```text
wow.api.exists;
wow.api.deprecated;
wow.api.arguments;
wow.event.exists_payload;
wow.widget.method;
wow.toc.reachable;
wow.load.use_before_load;
wow.secret.local_operation;
wow.secret.unsafe_log;
wow.overlay.direct_blizzard_override;
wow.framework.duplicate_registration.
```

## 3.5. WoW dialect profile

Lua 5.1 alone is insufficient. Versioned profile contains:

```rust
struct WowDialectProfile {
    id: ProfileId,
    interface: u32,
    build: String,
    parser_version: String,
    allowed_std_globals: Vec<String>,
    removed_std_globals: Vec<String>,
    blizzard_globals: Vec<String>,
    require_like_functions: Vec<String>,
    nonstandard_symbols: Vec<String>,
    restricted_globals: Vec<String>,
    secure_environment_globals: Vec<String>,
    evidence_digest: Digest,
}
```

The generated profile configures Emmy and our own facts. Any disagreement is `dialect_gap`, never silent fallback.

## 3.6. Workspace assembly

```text
main workspace
    first-party addon files only;

library workspace
    generated WoW annotation pack;
    project-declared libraries;
    narrow Blizzard stubs referenced by the project;

excluded
    full Blizzard UI implementation tree;
    unrelated installed addons;
    generated source bodies.
```

Generated config is stored under `.wow/generated/<profile>/<generation>/emmyrc.json`. User editor config is never overwritten.

## 3.7. Ketho-compatible annotation pack

Output layout remains familiar:

```text
Annotations/Core/Blizzard_APIDocumentationGenerated/
Annotations/Core/Data/Enum.lua
Annotations/Core/Data/Event.lua
Annotations/Core/Data/CVar.lua
Annotations/Core/Widget/
Annotations/Core/ScriptObject/
Annotations/Core/Type/
Annotations/Core/FrameXML/
Annotations/Core/WowDialect/
```

Compatibility is verified against Ketho output for the same Blizzard snapshot. Exact byte equality is not required; canonical symbol/type equality is.

## 3.8. Raw metadata sidecar

Annotations cannot carry every Blizzard field. Reference pack also stores:

```text
all known APIDocumentation fields;
unknown fields as raw canonical values;
SecretArguments/SecretReturns/ConditionalSecret;
predicates;
source span;
Ketho correction provenance;
build applicability.
```

## 3.9. Emmy compatibility probe

Every candidate Emmy update is probed for:

```text
config keys and behavioral effects;
LuaCATS tags actually affecting inference;
diagnostic IDs/default severities;
incremental update behavior;
pull determinism;
large library performance;
WoW fixture noise.
```

Lost mandatory capability blocks automatic activation. New diagnostic families default to shadow until classified.

## 3.10. Upstream contribution

Desired Emmy compile-time extension:

```rust
pub trait ExternalDiagnosticProvider: Send + Sync {
    fn id(&self) -> &'static str;
    fn check(&self, model: &SemanticModel, ctx: &mut DiagnosticContext);
}
```

No dynamic Rust plugin ABI is proposed. Our product works without this upstream change.

---

# 4. Rust reference builder: Ketho principle without Ketho runtime dependency

## 4.1. Inputs

```text
Blizzard UI snapshot from any provider;
Blizzard_APIDocumentation;
Blizzard_APIDocumentationGenerated;
TOC/XML/Lua UI source;
Blizzard_Deprecated;
optional BlizzardInterfaceResources;
reviewed corrections;
Ketho and Numy outputs for differential checks.
```

Provider is provenance, not authority. Default release CI may materialize Blizzard UI through the Gethe mirror; a local official export or InterfaceExport can produce the same logical snapshot.

## 4.2. APIDocumentation ingestion

```text
emmylua_parser CST
→ restricted declarative evaluator
→ raw canonical Lua value tree
→ schema-aware lowering
→ ApiSymbol/Event/Table/Widget/Predicate facts
→ raw unknown-field preservation.
```

Arbitrary Lua is never executed.

Allowed evaluator subset:

```text
literals;
table constructors;
local bindings;
field/index access to known constants;
known registration calls;
bounded constant expressions.
```

Unknown construct produces an ingestion diagnostic and affects only its partition unless contract completeness is compromised.

## 4.3. FrameXML ingestion

```text
TOC order
+ quick-xml streaming parse
+ Emmy Lua syntax facts
→ templates/frames/regions/scripts/anchors/inheritance/mixins
→ package-local graph shard
→ source spans.
```

Parity corpus compares:

```text
our extracted mixins/templates/methods
vs Numy FramexmlAnnotations
vs Ketho FrameXML annotation slice.
```

A disagreement is retained with both evidence sources; no blind overwrite.

## 4.4. Corrections

Corrections are data:

```rust
struct CuratedCorrection {
    target: EntityKey,
    field: FieldPath,
    expected_source_digest: Digest,
    replacement: CanonicalValue,
    evidence: Vec<EvidenceRef>,
    reviewed_by: String,
}
```

Upstream source change expires the correction automatically.

## 4.5. Build profiles

The project supports multiple pinned reference profiles because addon TOCs may target 120001/120005 while current live evolves.

```text
one active project profile selected from TOC/config;
current profile for new development;
selected historical profiles for migration/lineage;
PTR profile physically separate and advisory only.
```

No mixed-profile diagnostics.

## 4.6. `ReferencePack`

```text
manifest.json
reference.sqlite
annotations/
source-map.sqlite
raw-apidoc.zst
ui-source-skeletons.zst
checksums.json
licenses/
```

Normal users download a validated pack. They do not need Ketho, PHP, VS Code, LuaRocks or Blizzard UI source locally.

---

# 5. Универсальный WoW graph

## 5.1. Важное уточнение: у функции нет одного `parent`

Для WoW нужно хранить несколько независимых родительских осей:

```text
lexical parent
    enclosing function/table/file;

ownership parent
    addon package/module/namespace/mixin;

load parent
    TOC variant and load unit;

object parent
    CreateFrame parent, XML parent/child;

inheritance parent
    XML inherits, CreateFromMixins, prototype/metatable;

registration parent
    event/callback/style/region/plugin registry;

lifecycle parent
    initializer/enable handler/encounter/module factory;

call parent
    proven or possible caller.
```

Поэтому search result показывает `owner_chain`, `load_chain`, `object_chain`, `registration_chain` и `call_neighborhood`, а не одно двусмысленное поле.

## 5.2. Node kinds

Open string registry, стартовый набор:

```text
repository
build
addon_package
toc_manifest
toc_variant
file
namespace
module
service
library
function
method
callback
event
api_symbol
enum
cvar
xml_template
frame
region
mixin
prototype
factory
registry
style
element
plugin
feature
state_root
state_path
extension_point
restriction_facet
runtime_finding
source_span.
```

## 5.3. Relation kinds

```text
contains
declares
defines
exports
loads
loads_before
depends_on
optional_depends_on
inherits
mixes_in
instantiates
parent_of
created_by
calls
possible_calls
registers_event
handles_event
triggers_callback
subscribes_callback
hooks
sets_script
references_template
uses_api
reads_state
writes_state
embeds_library
requires_library
owns
implements_role
replaced_by
moved_to
same_lineage_as
present_in_build
removed_in_build
runtime_touches.
```

## 5.4. Evidence contract

Every edge stores:

```text
source span or source artifact;
extractor/recognizer ID and version;
reference/project generation;
confidence = Proven | Derived | Possible;
coverage partition;
optional competing evidence.
```

No edge becomes `Proven` from name similarity or LLM judgment.

## 5.5. Function skeletons

```text
L0
    signature, owner/load chains, direct API/event/state roles, caller/callee counts;

L1
    signature, branches, loops, calls, early returns, state effects;
    implementation bodies collapsed;

L2
    exact source span.
```

Default agent read is L1. Full source requires explicit request.

## 5.6. Current vs historical Blizzard UI

Verified comparison:

```text
10.0.0 snapshot (2022-10-25)
    Interface had top-level FrameXML/SharedXML/GlueXML and AddOns;
    ActionButton.lua lived under Interface/FrameXML.

12.1.0 snapshot (2026-08-13)
    UI is materialized primarily as Interface/AddOns packages;
    Blizzard_ActionBar contains Shared/Mainline/WoWLabs and flavor TOCs;
    ActionButton.lua lives in Blizzard_ActionBar/Shared.
```

The semantic symbol `ActionButtonDown` survives the path/container migration while callback/security behavior around it evolves.

Therefore path identity is insufficient. `BuildLineage` combines:

```text
git rename/move evidence;
normalized AST fingerprint;
signature/receiver shape;
owner/load neighborhood;
call/API/event neighborhood;
deprecation/transition guides.
```

## 5.7. Trees exposed to the agent

```text
owner tree
load tree
object/template tree
call tree
event/callback tree
state flow tree
build lineage tree
impact tree.
```

Each is a bounded projection of the same graph, not a separate truth store.

---

# 6. Universal recognizer artifacts

## 6.1. Principle

A recognizer recognizes a structural convention and emits universal roles. It does not encode product semantics by addon name.

Bad:

```text
if repository == "ElvUI" then ...
```

Good:

```text
call receiver.NewModule("Name")
→ emit node role=module
→ emit created_by factory
→ attach lexical/owner/load evidence.
```

## 6.2. Core recognizers

```text
TOC package/load/dependency;
XML template/frame/parent/inherits/scripts;
CreateFrame factory/template/parent;
CreateFromMixins/Mixin assignment;
EventRegistry/CallbackRegistry/AceEvent registration;
SetScript/HookScript/hooksecurefunc;
LibStub/library embeds;
module/addon factory;
plugin/region/style/element registry;
SavedVariables/state roots and literal paths;
slash commands;
message buses;
flavor/edition partitions;
Secret guards and sinks.
```

## 6.3. Representative corpus and extracted principles

### Ace3

Observed principles:

```text
addon factory;
module factory and parent module;
embedded libraries;
initialization/enable queues;
lifecycle callbacks;
event/message services.
```

### oUF

```text
style registry;
element registry;
frame factory/prototype;
element enable/disable lifecycle;
event dispatch;
metatable-injected methods.
```

### ElvUI

```text
engine object;
module registry;
profile/private/global state roots;
service/library registry;
flavor partitions;
skin/overlay callback registry;
Secret helper aliases.
```

### WeakAuras

```text
private/public namespaces;
rich LuaCATS data schemas;
region/subregion registries;
trigger prototypes;
factories;
state machines;
serialization boundary.
```

### BigWigs

```text
domain module factory;
module identity and encounter owner;
lifecycle callbacks;
event handlers;
message bus;
declarative options/aura data;
local ephemeral state.
```

### Details

```text
plugin install factory;
plugin registries by kind;
public global registration;
persistent plugin database;
plugin lifecycle events;
version/compatibility checks.
```

### Plater

Used primarily to calibrate:

```text
script/hook surfaces;
nameplate object factories;
mod/hook registries;
profile and script payload boundaries;
large generated/user-script areas.
```

## 6.4. Named packs

Files may be called `ace3.json`, `ouf.json`, `elvui.json`, but they contain only declarative call/field/annotation patterns. Removing a pack reduces coverage; it never changes core semantics.

## 6.5. Why v8 does not use ast-grep as a second parser

v6.4 correctly demanded data-driven recognizers, but it assumed a more external Emmy arrangement. In v8 we embed Emmy and already have exact CST/semantic facts. A second Lua parser would create dialect and span disagreement.

Decision:

```text
one syntax source = emmylua_parser;
recognizer DSL operates on normalized SyntaxFacts;
no tree-sitter/ast-grep in correctness path;
CBM tree-sitter remains an external broad-search engine.
```

---

# 7. Codebase Memory bridge

## 7.1. What is possible now

```text
CBM indexes Blizzard/project/external repositories;
CBM finds semantic candidates and broad call paths;
bridge converts CBM path/symbol/span results to StableSourceHandle;
wow-index resolves the handle in exact project/UI graph;
merged answer separates CBM Candidate evidence from WoW Proven evidence.
```

## 7.2. What is not cleanly possible in the checked CBM source

No supported runtime interface was found for importing arbitrary WoW nodes such as TOC manifests, XML templates, frames, Secret facets or custom relation types into CBM's persistent graph.

Therefore prohibited:

```text
writing directly into CBM SQLite;
generating fake Lua files solely to trick CBM;
patching vendored CBM language specs in our installer;
treating CBM Lua CALLS as exact WoW call proof.
```

## 7.3. Own graph scope

Our graph is deliberately small and domain-specific:

```text
TOC/XML/load;
API/UI ownership;
callbacks/events/hooks;
state paths;
framework registries;
Secret/restriction facts;
lineage/patch impact.
```

It does not duplicate:

```text
source embeddings;
repository-wide BM25;
generic architecture clustering;
all-language cross-service graphs;
3D visualization.
```

## 7.4. Optional direct MCP client

`wow-cbm-bridge` may connect to a user-configured CBM MCP command through standard MCP transport. It does not own or update CBM.

```rust
trait CbmBridge {
    async fn ensure_index(&self, repo: &RepositoryIdentity) -> Result<CbmGeneration>;
    async fn semantic_candidates(&self, query: &str, scope: &RepoScope) -> Result<Vec<CbmCandidate>>;
    async fn trace_candidates(&self, seed: &StableSourceHandle) -> Result<Vec<CbmRelation>>;
    async fn coverage(&self, scope: &RepoScope) -> Result<CbmCoverage>;
}
```

No configured bridge means own exact search still works.

## 7.5. Upstream proposal

A separate schema in this package defines `DerivedFactsPack`:

```text
provider;
repository/revision;
nodes with path/span/key;
edges with relation/confidence/evidence;
namespace for custom relation kinds;
replace-by-generation semantics.
```

If CBM accepts such an import ABI, our sidecar graph can optionally be projected into CBM. Product correctness does not depend on acceptance.

---

# 8. Search engine

## 8.1. Search stages

```text
0. query classification
1. exact active canonical name
2. aliases/deprecation/replacement/build lineage
3. namespace/member/prefix
4. receiver/signature/return/restriction shape
5. normalized edit-distance/trigram candidates
6. FTS5 BM25 over docs/comments/L0-L1 skeletons
7. graph-neighborhood expansion
8. optional CBM semantic source candidates
9. deterministic reranking and evidence classification.
```

## 8.2. Structured scoring dominates names

Ranking signals in descending authority:

```text
explicit replacement/deprecation edge;
exact canonical/alias match in active profile;
lineage evidence across builds;
entity kind and namespace;
receiver and parameter/return shape;
restriction facet compatibility;
package/load affinity;
graph neighborhood overlap;
documentation BM25;
name similarity.
```

## 8.3. Result contract

```rust
struct SearchHit {
    entity: EntityKey,
    canonical_name: String,
    kind: String,
    active_profile: ProfileId,
    owner_chain: Vec<EntityKey>,
    load_chain: Vec<EntityKey>,
    source: SourceSpan,
    migration: Option<MigrationStatus>,
    confidence: EvidenceLevel,
    why: Vec<SearchSignal>,
    detail_handle: StableHandle,
}
```

## 8.4. Removed function behavior

If an exact active symbol is missing:

```text
search migration journal;
search historical lineage;
compare signature/receiver/restriction shape;
inspect current callers/usage examples;
return explicit replacement only when evidence supports it;
otherwise return ranked candidates labeled Candidate.
```

Never:

```text
"probably replacement" presented as fact;
auto-fix from text similarity;
negative `not_found` from a stale/partial profile.
```

## 8.5. Search examples

```text
wow search "ExactFunction"
    → exact function + owner/load/call summary.

wow search "RemovedOldFunction" --profile retail-120001 --target current
    → removed/moved/replacement/unknown-history result.

wow search "update bag slot UI"
    → structured package/API candidates;
    → optional CBM source examples;
    → exact verification before recommendation.
```

## 8.6. Skeleton-first search response

Default response is ≤2 KB:

```text
1–5 ranked hits;
ownership/load chain;
important inbound/outbound relations;
replacement/restriction status;
L0 summary;
handles for L1/full source.
```

---

# 9. Secret Value indexing and lint

## 9.1. Canonical model

Open facet registry:

```text
secret.return
secret.argument
secret.conditional
secret.predicate
secret.aspect
protected.action
hardware_event.required
combat.restriction
secure_hooks.allowed
forbidden.aspect
private_script_object
private_partition_member.
```

Unknown facet is preserved raw and marks affected checks `NotEvaluated`.

## 9.2. Secret type projection

For linting only, generated annotations may use nominal types:

```lua
---@class WowSecretValue
---@class WowSecretNumber: WowSecretValue
---@class WowSecretBoolean: WowSecretValue
```

Rules:

```text
AlwaysSecret return
    → nominal secret type;

ContextuallySecret return
    → ordinary|secret union + facet metadata;

NeverSecret
    → ordinary type.
```

This is an analysis approximation, not a claim that the runtime wraps values in Lua objects.

## 9.3. Secret-local MVP

Detect direct local misuse:

```text
arithmetic;
comparison;
concatenation;
table key/index;
unsafe serialization/logging;
branching where forbidden;
storing in containers that reject secret;
passing into forbidden argument;
losing an access guard.
```

Recognized guards/predicates include project/reference facts for:

```text
canaccessvalue;
canaccessallvalues;
issecretvalue;
C_Secrets predicate family;
known secure/insecure bridges.
```

## 9.4. Flow levels

```text
Level 0 contract
    API facets only;

Level 1 local
    expression/guard dominance inside one function;

Level 2 bounded interprocedural
    only stable direct calls and summaries;

Level 3 runtime evidence
    taint/error correlation.
```

Level 2 never blocks MVP shipment. Dynamic gaps remain explicit.

## 9.5. Overlay and Secret interaction

A callback hooked into Blizzard may receive secret values or run in a protected chain. Findings therefore combine:

```text
extension point context;
API facets;
argument use;
combat reachability;
load timing;
runtime evidence requirement.
```

---

# 10. Storage: small text + graph database

## 10.1. Physical files

```text
reference.sqlite
    immutable/read-only per profile;

project.sqlite
    WAL, rebuildable per repository/generation;

external.sqlite
    optional metadata/handles for downloaded external repos;

objects/
    content-addressed compressed source/skeleton blobs.
```

## 10.2. Why SQLite

One engine provides:

```text
exact B-tree indexes;
FTS5 BM25/trigram;
adjacency tables;
transactions;
WAL;
portable single files;
excellent debugging/inspection;
small operational footprint.
```

No Neo4j, Kuzu, AGE, Qdrant or separate search daemon in v1.

## 10.3. Graph tables

```sql
entity(id, stable_key, kind, name, name_norm, profile_id, container_id, source_span_id, attrs_json)
edge(id, from_id, relation, to_id, evidence_level, extractor_id, generation, source_span_id, attrs_json)
source_span(id, repo_id, revision, path, byte_start, byte_end, line_start, line_end, digest)
alias(entity_id, alias, alias_norm, kind)
lineage(from_entity_id, to_entity_id, relation, evidence_level, build_from, build_to)
restriction(entity_id, facet_kind, schema_version, payload_json, source_span_id)
```

Covering indexes:

```text
edge(from_id, relation, to_id);
edge(to_id, relation, from_id);
entity(kind, name_norm, profile_id);
alias(alias_norm, kind);
lineage(from_entity_id, build_to);
```

## 10.4. FTS

FTS indexes:

```text
canonical/alias names;
documentation;
L0/L1 skeletons;
comments around declarations;
framework role labels;
migration notes.
```

Source code semantic embeddings remain CBM's job.

## 10.5. In-memory graph

Only the requested bounded neighborhood is loaded into `petgraph`. Large project/reference graphs remain in SQLite. Roaring bitmaps may be added only after benchmarked need.

## 10.6. Retention

```text
current and configured target profiles;
compact lineage journal for older builds;
last N project generations;
external corpus metadata without duplicated repository source;
content-addressed garbage collection.
```

---

# 11. Indexing Blizzard UI, own addons and external addons

## 11.1. Universes

```text
Reference universe
    Blizzard UI/API profiles;

Workspace universe
    writable addon repositories;

Dependency universe
    declared libraries/addons, read-only;

External example universe
    GitHub repositories cloned on demand, read-only;

Installed runtime universe
    optional installed addons/SavedVariables/logs.
```

## 11.2. External corpus manifest

We do not vendor third-party addon source. Store:

```text
repo URL;
commit SHA;
license/SPDX when detected;
retrieved_at;
index generation;
recognized framework roles;
source digest.
```

## 11.3. Authority

```text
Blizzard UI/reference contracts
    platform authority;

our repository
    project truth;

external addons
    examples and pattern evidence only;

CBM semantic result
    candidate selector;

community code
    never API authority.
```

## 11.4. Patch impact

```text
new ReferencePack
→ API/facet/event/template/package lineage diff
∩ project uses/hooks/templates
→ affected modules/files/functions
→ L0/L1 study set
→ checks and manual runtime scenarios.
```

---

# 12. Official GitHub workflow for agents

## 12.1. Required tools

```text
GitHub CLI `gh`;
official GitHub MCP Server in read-only mode;
system Git;
our `wow` CLI.
```

GitHub MCP is configured with minimal `repos`/code-reading toolsets and `--read-only`. Write toolsets are enabled only for an explicit publishing/review task.

## 12.2. Discovery sequence

```text
1. wow search exact/current/migration
2. wow tree/trace in Blizzard UI
3. wow skeleton L0/L1
4. only then search GitHub examples
5. inspect repository license and commit
6. clone/index selected repo
7. compare universal patterns
8. verify proposed code against current reference.
```

## 12.3. Commands

```powershell
gh auth status

gh search repos "World of Warcraft addon <concept>" --limit 20

gh search code "FunctionName" --language Lua --limit 100

gh repo clone OWNER/REPO .wow-corpus/OWNER__REPO -- --filter=blob:none --depth=1

git -C .wow-corpus/OWNER__REPO rev-parse HEAD

wow index repo .wow-corpus/OWNER__REPO --universe external
wow search "FunctionName" --universe external
wow compare pattern <current-target> --against OWNER/REPO
```

`gh search code` currently uses the legacy code-search API and may differ from github.com; for regex/new-search behavior use GitHub web search or the official GitHub MCP `search_code` capability when available.

## 12.4. Copying policy

The agent records license before copying code. Architectural ideas and API usage can be compared; incompatible licensed source is not copied into our MIT project.

---

# 13. Lessons from other game ecosystems

## 13.1. Roblox Client Tracker + Luau LSP + Rojo

Current tracker and LSP remain actively updated in August 2026.

Transferable pattern:

```text
proprietary live client
→ current community tracker/API dump
→ generated local types/docs
→ language server
+
separate project instance tree/sourcemap.
```

For WoW:

```text
Blizzard UI snapshot
→ ReferencePack
→ wow-emmy-ls
+
TOC/XML/project graph.
```

Unknown platform types should degrade conservatively, not become fabricated errors.

## 13.2. Minecraft Yarn + Loom

Transferable pattern:

```text
versioned mapping artifact;
deterministic local cache/remap;
updates and diffs separate from normal IDE queries;
consumer tooling depends on artifact contracts, not raw upstream layout.
```

For WoW, ReferencePack and lineage journal play the mapping role.

## 13.3. FFXIVClientStructs/Dalamud

The project updates structs/signatures rapidly after game patches.

Transferable pattern:

```text
generated strongly typed bindings;
patch-specific revisions;
signature resolution with explicit failure;
no continued operation with unresolved addresses.
```

For WoW:

```text
unresolved API/UI symbol or attachment point
→ Unknown/NotEvaluated;
never an invented safe hook.
```

---

# 14. Agent-facing cognitive layer

## 14.1. Tools

Keep the surface compact:

```text
wow_status
wow_search
wow_tree
wow_skeleton
wow_context
wow_check
wow_patch_impact
wow_index_repo
```

`wow_search` routes across exact/migration/text/graph/optional CBM internally and reports which lanes were used.

## 14.2. `wow_tree`

Views:

```text
owner
load
object
inheritance
call
event
state
lineage
impact.
```

## 14.3. `wow_context`

Task context includes:

```text
ProjectMap ≤2 KB;
target owner/load chain;
relevant L0/L1 skeletons;
current API/restriction contracts;
external example handles only when requested;
known dynamic/coverage gaps;
checks required before completion.
```

## 14.4. Generated project memory

`.wow/ARCHITECTURE.generated.md` is generated from project graph, ≤2 KB and safe to inject into every agent session. It is not a manual personal card.

Contents:

```text
load skeleton;
module/service/state owners;
registries/extension points;
critical invariants;
active reference profile;
known dynamic gaps;
open workaround debt.
```

## 14.5. Read discipline

```text
search/outline first;
L0/L1 before full source;
no full Blizzard file unless exact span is insufficient;
external example source only after exact current API/UI study;
all material conclusions include source handles.
```

---

# 15. Open-source and Blizzard policy

Blizzard's published UI Add-On Development Policy says add-ons must be distributed free of charge and addon code must be completely visible, unobfuscated and publicly viewable.

That policy does not itself use OSI-license terminology. Our stricter project default is:

```text
public GitHub repository;
MIT license for the tool and generated addon templates;
no premium addon features;
no obfuscated addon code;
no in-game advertising/donation solicitation;
third-party notices and source provenance retained.
```

External corpora are cloned on demand and not redistributed in release artifacts.

---

# 16. Rust workspace

## 16.1. Production crates

```text
wow-core
    IDs, profiles, evidence, findings, stable handles.

wow-store
    SQLite schemas, migrations, content-addressed blobs.

wow-reference
    APIDoc lowering, ReferencePack build/read, Ketho parity.

wow-annotations
    Ketho-compatible and Emmy-dialect projections.

wow-emmy
    Emmy adapter, project actor, diagnostic provider registry.

wow-project
    Lua/TOC/XML files and incremental project generations.

wow-graph
    universal nodes/relations, lineage and bounded graph queries.

wow-recognizers
    data-driven SyntaxFact matcher and framework packs.

wow-search
    exact/migration/shape/FTS/graph ranking.

wow-rules
    API/load/event/Secret/overlay/project diagnostics.

wow-cbm
    optional MCP client/handoff bridge.

wow-context
    skeletons, ProjectMap and response budgets.

wow-service
    use cases independent of CLI/MCP/LSP.

wow-app
    CLI, MCP and LSP binaries.
```

Development-only:

```text
wow-testkit
wow-eval
```

## 16.2. Processes

Normal interactive operation:

```text
one wow process or daemon;
optional existing CBM process;
no process per crate;
no database server.
```

Reference build CI may run Ketho/Numy differential jobs separately.

---

# 17. Implementation sequence with blocking gates

## E0 — executable vertical slice

Build:

```text
pinned Emmy adapter;
minimal ReferencePack fixture;
Ketho-compatible annotation library fixture;
one generic Emmy diagnostic;
one WoW API-not-found diagnostic;
one Secret-local diagnostic;
CLI `wow check`.
```

Gate:

```text
same file receives merged generic + WoW findings;
known valid WoW API resolves through annotations;
unknown current API is found without grep;
no user editor config mutation;
1/2/N repeated runs byte-identical after sorting.
```

## E1 — full API reference and annotations

```text
restricted APIDoc evaluator;
all generated systems/tables/events/widgets;
raw unknown fields;
Ketho parity;
profiles 120001/120005/current as configured;
Spartan query parity.
```

Gate:

```text
all source files ingested or explicitly diagnosed;
negative authority only for complete partitions;
annotation pack accepted by Emmy probe;
Secret metadata retained raw.
```

## E2 — project model

```text
TOC parser;
XML parser;
load graph;
API use index;
state/event/hook facts;
Secret-local rules;
ProjectMap.
```

Gate:

```text
unreachable/use-before-load fixtures caught;
project generation updates only affected files/relations;
ProjectMap ≤2 KB;
zero blocking false positives in launch corpus.
```

## E3 — basic Blizzard UiGraph and skeletons

```text
package shards;
TOC variants;
functions/methods/templates/frames/mixins;
L0/L1 skeletons;
owner/load/object/call trees.
```

Gate:

```text
current and 10.0.0 ActionButton lineage fixture resolves;
package query touches bounded data;
agent retrieves target with ≤3 source reads.
```

## E4 — search and lineage

```text
aliases;
deprecations;
replacement journal;
AST/signature/graph lineage;
FTS5;
ranker and explanations.
```

Gate:

```text
explicit replacements outrank fuzzy hits;
unknown replacements remain candidates;
search benchmark top-3 recall ≥0.9 on labeled WoW tasks.
```

## E5 — framework recognizers and external corpus

```text
core recognizer DSL;
Ace3/oUF/ElvUI/WA/BigWigs/Details/Plater packs;
official GitHub workflow;
external repo index manifests.
```

Gate:

```text
named pack removal changes coverage only;
no production code branches on repository name;
universal role precision measured per pack.
```

## E6 — CBM bridge

```text
MCP client/handoff;
stable source handle joins;
merged search results;
coverage separation;
DerivedFacts upstream proposal.
```

Gate:

```text
CBM unavailable does not break exact search;
CBM Candidate is never presented as Proven;
no direct CBM DB access.
```

## E7 — production LSP/MCP/packaging

```text
thin LSP methods;
MCP tools;
installer;
prebuilt ReferencePacks;
GitHub Actions release;
cargo-deny/audit/SBOM;
rollback/last-known-good.
```

---

# 18. Evaluation corpus and metrics

## 18.1. Corpora

```text
Blizzard UI current + selected historical snapshot;
Ketho output;
Numy output;
synthetic generated projects;
user portfolio;
Ace3/oUF/ElvUI/WeakAuras/BigWigs/Details/Plater manifests;
mutated negative fixtures;
Secret/local restriction fixtures.
```

## 18.2. Agent tasks

At least 30 real tasks:

```text
find current API;
replace removed API;
locate Blizzard package/function;
find safe attachment surface;
trace event to UI update;
identify state owner;
assess change impact;
compare community implementations;
repair Secret misuse;
explain load failure.
```

## 18.3. Metrics

```text
files read per task;
bytes/tokens delivered;
search top-1/top-3 recall;
first-patch acceptance;
false blocking rate;
unknown reported honestly;
index/update latency;
project/reference DB size;
Emmy update compatibility;
CBM-on vs CBM-off benefit.
```

Promotion rule:

```text
no component enters default path without measured task benefit or unique correctness responsibility.
```

---

# 19. Recovery of decisions from old documentation

## 19.1. Preserved

```text
ArchitectureSnapshot/ProjectMap as generated project state;
evidence/coverage/generation on facts;
UiIndex and package shards;
ExtensionPoint and overlay assessment;
L0/L1/L2 skeletons;
open RestrictionFacet registry;
Ketho/Numy differential oracles;
no full Blizzard tree in Emmy library;
exact/migration before text similarity;
SQLite FTS5 before custom search engine;
CBM for broad discovery;
Secret-local before deep flow;
patch impact;
agent eval before complexity;
watch/post-write verification compatibility;
generation instead of repeated manual conformance where possible.
```

## 19.2. Corrected

```text
Gethe-first
    → Blizzard UI first; Gethe is one acquisition provider.

external Emmy process only
    → public Emmy Rust library embedded behind one adapter.

ast-grep recognizers
    → data-driven recognizers over canonical Emmy SyntaxFacts to avoid second parser.

CBM-only graph
    → CBM broad graph + small WoW domain graph.

current-only reference
    → one active target profile + compact configured historical lineage.

personal KB cards
    → generated ProjectMap and versioned public rule packs.
```

## 19.3. Rejected

```text
44+ microcrates as an implementation target;
custom graph/search servers in v1;
vector DB inside WoW backend;
hardcoded addon-specific production logic;
full deep Secret flow before local rules work;
editor setting mutation as correctness mechanism;
direct writes into CBM database;
full Blizzard UI source in model context;
claim that a single `parent` captures WoW ownership.
```

---

# 20. Normative ADRs

```text
ADR-01. EmmyLua analyzer is an upstream Rust library dependency, not a fork.
ADR-02. `wow-emmy-ls` merges built-in Emmy and WoW diagnostics in one generation.
ADR-03. Current lack of public external checker registration is handled by our host; upstream PR is optional.
ADR-04. Ketho defines compatibility behavior and parity fixtures, not editor runtime policy.
ADR-05. Numy is a differential oracle; FrameXML truth comes from structural source parsing.
ADR-06. Raw Blizzard metadata and generated annotations are separate projections.
ADR-07. Blizzard UI content is canonical; acquisition provider is provenance.
ADR-08. A project selects one active reference profile; profiles never mix in diagnostics.
ADR-09. WoW graph stores typed parent axes, not one generic parent.
ADR-10. Recognizers emit universal roles and are data, not repository-specific branches.
ADR-11. Emmy parser is the sole correctness-path Lua parser.
ADR-12. CBM remains unchanged broad code intelligence; no direct DB mutation.
ADR-13. WoW domain graph is a small sidecar until a supported CBM import ABI exists.
ADR-14. Exact/replacement/lineage/shape rank before fuzzy/text/semantic similarity.
ADR-15. SQLite is the first storage/search/graph substrate.
ADR-16. Secret facets are open and unknown facets degrade affected rules to NotEvaluated.
ADR-17. Community addons are example evidence, never platform authority.
ADR-18. External repository source is cloned on demand and not vendored.
ADR-19. Agent reads L0/L1 skeletons before full source.
ADR-20. Project architecture memory is generated and portable.
ADR-21. Default project/release license is MIT and repository is public.
ADR-22. No default component without an agent-eval or correctness justification.
```

---

# 21. Definition of Done

## Reference

```text
all configured Blizzard APIDoc files ingested or explicitly rejected;
Ketho canonical parity report green;
Numy differential report available;
raw restrictions preserved;
profiles packaged and atomic;
exact/migration search deterministic.
```

## Emmy

```text
no fork;
no editor mutation;
built-in + WoW diagnostics merged;
annotation pack loaded as library;
compat probe generated per Emmy artifact;
update rollback tested.
```

## Graph/search

```text
TOC/XML/package/function/template/event/state relations indexed;
all inferred edges carry evidence/coverage;
multiple parent axes exposed;
L0/L1 skeletons available;
removed/moved symbols resolve through lineage or remain explicit candidates.
```

## CBM

```text
optional bridge works through documented transport;
no direct database access;
Candidate vs Proven separation enforced;
CBM-off workflow remains functional.
```

## Agent package

```text
AGENTS.md installed;
official gh/GitHub MCP workflow documented;
external repo commit/license captured;
current Blizzard source studied before community examples;
check required before task completion.
```

---

# 22. Source snapshot register

## Core tooling

```text
Ketho/vscode-wow-api
    d0b5b51fac4c52c493371b9b18e66ce604ea4326

EmmyLuaLs/emmylua-analyzer-rust
    aaaca68425d9362876228649b0b8d92f07654daa

LuaLS/lua-language-server
    7a73c7889c1ec981dfd76fba38f5096379f62f99

Tencent/LuaHelper
    15c9fac58a73c6f780257c81664a03b52d941f9d

NumyAddon/FramexmlAnnotations
    b38f5ec0f9fbf493e31c4f060b6f3db2ef743c78

spartanui-wow/wow-api-mcp
    81e96a304315bca2ac0ac1c064feaf97c323803e

DeusData/codebase-memory-mcp
    e513beb487bea21105a031da328872c1b6f58eac
```

## Blizzard UI samples

```text
current checked mirror revision
    31c7f7b9cc79e56c986b365c06a6afbcf3c9177b
    12.1.0 (69299), 2026-08-13

historical comparison
    f0084386950fe3dc31a1d61de33b364e268cf66b
    10.0.0 (46293), 2022-10-25
```

## Representative addon/framework corpus

```text
WoWUIDev/Ace3              d295b12f8b889a30e86e0e901c5494df4b149c49
oUF-wow/oUF                b6d1005ea6b6e4cdf1e2d7729ada8d3a08986074
tukui-org/ElvUI            bfc542d437f7e5e58234c0180c56a02502a6b758
WeakAuras/WeakAuras2       7bb9d239987921e41166dc2395f6e31ec78c94d7
BigWigsMods/BigWigs        2b8efea7b77801ed8b93e9a1720feb5b96eeb4d5
Tercioo/Details-Damage-Meter c139bf364e92455a69933b242a1b743e7a696d6b
Tercioo/Plater-Nameplates  0b764e35bb3c6248097866a615c3a874c97ad290
```

## Comparative ecosystems

```text
MaximumADHD/Roblox-Client-Tracker adfa070936d98e0f233b01b88f8f2c2b7f908ed0
JohnnyMorganz/luau-lsp             d5df9af2c703ffec5577b88b1b376c558b17348f
FabricMC/yarn                      ee98590897c919286b96b7c7f2c89ae4ac737762
FabricMC/fabric-loom               b04ca661baa0a22580286995bed86c059c9e89af
aers/FFXIVClientStructs            2db639850bd30a259f28b70f2bd963698a1ce1e5
```

---

# 23. Immediate engineering target

The next implementation milestone is not another architecture rewrite. It is E0:

```text
compile pinned Emmy;
load one generated WoW annotation fixture;
index one APIDoc fixture;
run generic + API + Secret-local diagnostics;
return one normalized `wow check` result;
lock it with golden tests.
```

Everything after E0 expands a proven vertical slice rather than restarting the design.
