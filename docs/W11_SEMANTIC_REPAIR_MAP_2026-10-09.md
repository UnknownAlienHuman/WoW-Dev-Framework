# W11 semantic repair map — 2026-10-09

## 1. Точная точка старта

Продолжать работу от финального проверенного product commit:

```text
eee77f4125b46bb08826856ade5ff57cc699780c
checked tree: 42c2ab87f4a6cba458f7e148e2b85613d2e00a4a
```

Не возвращаться к промежуточным checkpoint и не начинать от исходного дефектного baseline:

```text
a2bf0af3675203c07078c03f5641cef649b6c0ed
```

История bounded repair checkpoints:

```text
5348b9f2b7b345b5e27af9624f96d89d1c86ed96  callable/API-shape repair
4fff530c5629d91a07a81a16a2cceea918d6f6f0  receiver/evidence closure
c6451d88b6642cd48d705b4e2b8f3b6ac32a9965  CVar matcher predicate closure
eee77f4125b46bb08826856ade5ff57cc699780c  final support/coverage closure
```

Для проверки WoW API в этой операции moving selector `Gethe/wow-ui-source:live` был разрешён один раз в точную ревизию:

```text
09b9db7948abc9b9648dedaab51eb0cf3ee67b31
```

Это evidence этой операции, а не навечно зафиксированная актуальная версия. При следующей операции selector нужно разрешить заново и использовать одну и ту же exact revision для всех сравниваемых данных.

## 2. Что исправлено

Исправлен общий путь `wow-emmy -> wow-project -> wow-recognizers -> wow-service`, без второго парсера и без source-text heuristics.

1. Exact colon receiver отделён от positional arguments и выводится только из существующих generation-bound Emmy facts.
2. `RegisterUnitEvent` приведён к реальной форме `event, unit1, ...`; выдуманный handler argument удалён.
3. Исправлены актуальные формы EventRegistry frame-event bridge.
4. Custom producer/subscription связываются по exact receiver и event key; missing/ambiguous producer не превращается в `Derived` или clean negative.
5. CVar adapter:
   - требует exact `CVarCallbackRegistry` receiver;
   - публикует реальные matcher predicates `has_cvar_key` и `exact_cvar_key`;
   - сохраняет exact callback declaration в support closure и recognition receipt;
   - dynamic callback не создаёт endpoint.
6. `SetScript`/`HookScript` используют reviewed callable keys `Frame.SetScript`/`Frame.HookScript`; exact receiver не определяется только наличием `:`.
7. Исправлены двух- и трёхаргументная формы `hooksecurefunc`; unresolved/literal-only targets не получают выдуманную source declaration.
8. Library family:
   - возвращается от recognizer fact к Emmy `call_id` через typed field;
   - различает direct `LibStub`, `GetLibrary`, `NewLibrary` и reviewed embed;
   - сохраняет source/evidence/coverage support;
   - не теряет отношения разных callers к одной library entity.
9. Entity и relation proposals обрабатываются независимо от канонического порядка proposal IDs.
10. `native_event` допускает только recognizer confidence ceiling `Derived`/`Possible` и не выражает runtime delivery authority.
11. Native-event support теперь проверяет exact path/span/digest, generation binding и evidence provenance.
12. Custom-signal и script/hook graph proposals сохраняют matcher support/coverage; exact producer, receiver, target, handler и callback evidence включается в claim closure.
13. Затронутые fact/producer/evaluation/source-graph profiles версионированы, чтобы старые partitions не выглядели результатом новой семантики.

## 3. Проверочная ведомость

Последние три product checkpoints:

```text
run 37925701680
artifact 11613673014
sha256:b647d05de0d68a21c287674276738587f2224a95879be8c4024279d23d3f4e24
product 4fff530c5629d91a07a81a16a2cceea918d6f6f0

run 37926465849
artifact 11613929865
sha256:184d7da6cde7fbd0fdbaf5669e2bbe3df2b22cae9434120577a57eb3fdb97da4
product c6451d88b6642cd48d705b4e2b8f3b6ac32a9965

run 37928095898
artifact 11615330940
sha256:ce8894e1408d251c2341ff41797f695d37445f78fc649ec631fa390665c2740a
product eee77f4125b46bb08826856ade5ff57cc699780c
checked tree 42c2ab87f4a6cba458f7e148e2b85613d2e00a4a
```

Финальный checkpoint прошёл:

```text
cargo fmt --all
cargo check --locked \
  -p wow-emmy -p wow-project -p wow-recognizers -p wow-service -p wow-cli \
  --all-targets --all-features
cargo clippy --locked \
  -p wow-emmy -p wow-project -p wow-recognizers -p wow-service -p wow-cli \
  --all-targets --all-features -- -D warnings
cargo xtask check
git diff --cached --check
exact changed-file boundary
exact checked-tree non-force fast-forward
```

Среда последнего checkpoint:

```text
Ubuntu 24.04
rustc 1.99.0
EmmyLua pin aaaca68425d9362876228649b0b8d92f07654daa
```

Внутренние SHA-256 артефакта проверены; `CHECKED_TREE` совпадает с опубликованным product tree. В финальном дереве отсутствуют temporary `.ci/w11-*` и checkpoint workflows.

Тесты в этих functional checkpoints намеренно не запускались согласно текущему execution order. Это не test acceptance, не Windows acceptance и не WoW-runtime evidence.

## 4. Что остаётся незакрытым

W11/E2-B остаётся partial. Frozen contract объявляет 26 active rule IDs; service-публикованы 14. Не реализованы 12 правил:

```text
core.toc.*   — 5
core.xml.*   — 4
core.state.* — 3
```

Также не закрыты:

- full `apps/wow graph build` acceptance на реальном аддоне;
- effective XML receiver/inheritance/lifecycle/runtime-dispatch semantics;
- coherent ProjectView/GraphView publication;
- incremental invalidation;
- retention/GC, backup и recovery;
- настоящие rule-specific positive/near-negative/partial/mutation fixtures;
- expected match/proposal/partition IDs и checksum freeze;
- full E2 package acceptance;
- Windows и named-client WoW runtime evidence;
- сравнительный gate с WoW API Ketho MCP.

## 5. Порядок работы следующего агента

### Этап A — authority и current tree

Прочитать:

```text
AGENTS.md
.agents/skills/wow-dev/SKILL.md
docs/IMPLEMENTATION_STATUS.md
docs/PROJECT_COMPLETION_MATRIX.md
this document
crates/wow-recognizers/e2/README.md
crates/wow-recognizers/e2/RULE_FAMILIES.md
crates/wow-recognizers/e2/FACT_INPUT_MODEL.md
crates/wow-recognizers/e2/OUTPUT_AND_GRAPH_HANDOFF.md
crates/wow-recognizers/e2/TEST_MATRIX.md
```

Проверить exact HEAD/tree выше. Не создавать worktree или task branch. Не восстанавливать удалённые transport/checkpoint files.

### Этап B — закончить функциональные owner facts и оставшиеся правила

TOC first:

```text
core.toc.package@1
core.toc.file_order@1
core.toc.dependencies@1
core.toc.load_on_demand@1
core.toc.saved_variables@1
```

XML second:

```text
core.xml.template@1
core.xml.object@1
core.xml.inherits@1
core.xml.script@1
```

State third:

```text
core.state.saved_variable_root@1
core.state.literal_path_read@1
core.state.literal_path_write@1
```

TOC/XML parsing остаётся у существующих `wow-project` owners. `wow-recognizers` получает только typed, generation-bound facts. Каждый fact сохраняет package/variant identity, source order, exact span/content identity, confidence, ambiguity, omissions и coverage. Partial scope не доказывает отсутствие.

После каждого bounded functional slice:

```text
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
cargo xtask check
```

### Этап C — application/service closure

После реализации всех 26 active IDs:

1. Подключить partitions в однозначном owner order.
2. Вывести полный результат через реальный `apps/wow graph build` lane.
3. Подтвердить final node/edge crosswalk и exact source/evidence/coverage для каждой relation.
4. Не строить final graph IDs внутри recognizer.
5. Не превращать static structure в runtime frame existence, event delivery, loaded library revision, combat/taint safety или Secret authority.
6. Затем продолжить W12 conflict/derivation и W13–W16 publication/invalidation/retention/recovery.

### Этап D — полный тестовый этап после функциональной готовности

Каждый positive case обязан пройти полный тракт:

```text
exact Lua/TOC/XML input
-> одна Emmy/project session
-> typed owner facts
-> recognizer adapter
-> declarative pack/compiler/matcher
-> GraphProposalBatch validation
-> service partition replacement
-> apps/wow graph-build receipt
-> node/edge/source-evidence read-back
```

Sidecar-only проверка callable key не является rule test.

Обязательные группы:

- CreateFrame, CreateFromMixins, Mixin assignment;
- RegisterEvent и multi-unit RegisterUnitEvent;
- EventRegistry bridge с callback и без callback;
- custom producer+subscription, no producer, ambiguous producers;
- exact/dynamic CVar keys и exact/dynamic callback;
- SetScript, HookScript, обе формы hooksecurefunc, dynamic target;
- direct LibStub, GetLibrary, NewLibrary, reviewed embed, `Libs/`-only negative;
- все TOC/XML/state rules;
- shuffle/duplicate facts, budgets, truncation, cancellation;
- producer replacement и disablement без повреждения чужих partitions.

### Этап E — fixtures и freeze

Для каждого active rule нужны собственные:

- positive;
- structurally similar near-negative;
- partial/incomplete;
- dynamic/ambiguous, где применимо;
- rename/path/local-identifier mutation;
- shuffled/duplicate-fact determinism;
- budget/truncation;
- producer replacement/disable.

Затем фиксируются expected match/proposal/partition IDs, profile IDs и canonical SHA-256. Нельзя подставлять любой существующий `RECOG-*` ID или auto-bless изменённые fixtures.

## 6. Запрещённые сокращения

- Второй Lua/XML/TOC parser внутри recognizers.
- Regex, имя repository, путь `Libs/` или популярность как semantic condition.
- Receiver из `arguments[0]`.
- Graph proposal без exact source/evidence support.
- `Derived` из dynamic target или ambiguous producer.
- Clean negative из Partial/NotEvaluated coverage.
- Runtime claims из static structure.
- Fixture-ID substitution только ради прохождения existence check.
- Python/interpreter-based correctness path.

## 7. Обязательный финальный comparative gate: WoW API Ketho MCP

После прохождения собственного полного pipeline и frozen fixtures выполнить отдельный сравнительный тест через **WoW API Ketho MCP**. Без этого W11/E2-B нельзя объявлять semantic-complete.

Обе стороны используют одинаковые:

```text
WoW flavor
moving selector, один раз разрешённый в exact revision
Gethe source revision
admitted addon corpus
normalization profile
```

Сравнить нормализованные результаты:

1. callable resolution и colon/non-colon form;
2. argument positions/varargs `RegisterUnitEvent`;
3. EventRegistry native bridge и custom producer/subscription;
4. CVar callback registration;
5. SetScript, HookScript и обе формы hooksecurefunc;
6. LibStub require/new/embed и version handling;
7. source spans, owner/caller identity, confidence, omissions и blockers;
8. final entity/relation sets на одном corpus.

Обязательная discrepancy matrix:

```text
case
framework result
Ketho MCP result
exact Gethe evidence
runtime evidence, если требуется
classification: framework bug | Ketho difference | unsupported | unresolved
chosen action
```

Ketho MCP — comparative oracle и implementation donor, но не право молча переписывать факты или fixtures. Exact Gethe source остаётся source authority; runtime-sensitive расхождения остаются unresolved до named-client runtime probe.
