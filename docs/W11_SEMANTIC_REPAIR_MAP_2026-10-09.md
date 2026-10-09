# W11 semantic repair map — 2026-10-09

## Точка старта

Продолжать работу от проверенного product commit:

```text
5348b9f2b7b345b5e27af9624f96d89d1c86ed96
```

Исходный дефектный baseline, по которому проводился аудит:

```text
a2bf0af3675203c07078c03f5641cef649b6c0ed
```

Для проверки WoW API в этой операции `Gethe/wow-ui-source:live` был разрешён в точную ревизию:

```text
09b9db7948abc9b9648dedaab51eb0cf3ee67b31
```

Это evidence одной операции, а не навечно зафиксированная «актуальная версия». При следующем исследовании moving selector надо разрешить заново и использовать одну точную ревизию для всех сравниваемых данных.

## Что уже исправлено

Исправлен общий путь `wow-emmy -> wow-project -> wow-recognizers -> wow-service`, а не отдельные симптомы:

1. Добавлена консервативная проекция exact colon receiver из существующих generation-bound Emmy facts. Receiver больше не подменяется первым позиционным аргументом.
2. `RegisterUnitEvent` приведён к реальной форме `event, unit1, ...`; выдуманный handler argument удалён.
3. Исправлены текущие формы EventRegistry frame-event bridge и отдельные custom producer/subscription semantics.
4. CVar callback использует exact `CVarCallbackRegistry` receiver и отдельные аргументы key/callback.
5. `SetScript`/`HookScript` переведены на reviewed callable keys `Frame.SetScript` и `Frame.HookScript`; формы `hooksecurefunc` разделены.
6. Library matcher теперь возвращается от recognizer fact к Emmy `call_id`, переносит обязательные source/evidence support и различает require/GetLibrary/NewLibrary/embed.
7. Entity и relation proposals обрабатываются независимо от канонического порядка proposal ID.
8. `native_event` допускает только соответствующий recognizer ceiling: `Derived`/`Possible`, без runtime authority.
9. Актуализированы версии затронутых fact/producer/evaluation/source-graph profiles, чтобы старые partitions не выглядели результатом новой семантики.

Проверено на Linux, Rust 1.99.0:

```text
cargo fmt --all
cargo check --locked \
  -p wow-emmy -p wow-project -p wow-recognizers -p wow-service -p wow-cli \
  --all-targets --all-features
cargo clippy --locked \
  -p wow-emmy -p wow-project -p wow-recognizers -p wow-service -p wow-cli \
  --all-targets --all-features -- -D warnings
cargo xtask check
```

Все команды прошли. Тесты в этом checkpoint намеренно не запускались и не должны считаться выполненными.

## Что НЕ закрыто

Этот commit устраняет детерминированные блокеры, но не доказывает полную корректность W11/E2-B. Не закрыты:

- оставшиеся 12 rule ID: `core.toc.*` (5), `core.xml.*` (4), `core.state.*` (3);
- полный `apps/wow graph build` acceptance на реальном аддоне;
- effective XML receiver/inheritance/lifecycle/runtime-dispatch semantics;
- complete ProjectView/GraphView publication, invalidation, retention и recovery;
- настоящие rule-specific positive/near-negative/partial/mutation fixtures;
- fixture/checksum freeze и full E2 package acceptance;
- Windows/runtime acceptance;
- сравнительный gate с WoW API Ketho MCP.

## Порядок работы для следующего агента

### Этап 1 — заново прочитать authority и проверить HEAD

Обязательно прочитать:

```text
AGENTS.md
.agents/skills/wow-dev/SKILL.md
docs/IMPLEMENTATION_STATUS.md
docs/PROJECT_COMPLETION_MATRIX.md
crates/wow-recognizers/e2/README.md
crates/wow-recognizers/e2/RULE_FAMILIES.md
crates/wow-recognizers/e2/FACT_INPUT_MODEL.md
crates/wow-recognizers/e2/OUTPUT_AND_GRAPH_HANDOFF.md
crates/wow-recognizers/e2/TEST_MATRIX.md
```

Проверить, что `main` содержит product commit выше и что temporary `.ci/w11-*`/checkpoint workflow отсутствуют. Не начинать новый worktree и не возвращаться к baseline `a2bf0af`.

### Этап 2 — закончить функциональный код W11, без расширения тестовой матрицы

Сначала реализовать недостающие typed owner facts в `wow-project`. В `wow-recognizers` запрещено добавлять второй TOC/XML/Lua parser или source-text fallback.

Порядок:

1. TOC facts и правила:
   - `core.toc.package@1`;
   - `core.toc.file_order@1`;
   - `core.toc.dependencies@1`;
   - `core.toc.load_on_demand@1`;
   - `core.toc.saved_variables@1`.
2. XML facts и правила:
   - `core.xml.template@1`;
   - `core.xml.object@1`;
   - `core.xml.inherits@1`;
   - `core.xml.script@1`.
3. State rules:
   - `core.state.saved_variable_root@1`;
   - `core.state.literal_path_read@1`;
   - `core.state.literal_path_write@1`.

Каждый owner fact обязан сохранять exact generation, package/variant identity, source order, exact span, content identity, confidence, ambiguity, omissions и coverage. Частичная область никогда не доказывает отсутствие.

После каждого небольшого функционального slice запускать только минимальные проверки:

```text
cargo fmt --all --check
cargo check --locked -p <affected crates> --all-targets --all-features
cargo clippy --locked -p <affected crates> --all-targets --all-features -- -D warnings
cargo xtask check
```

Не раздувать тесты до завершения функционального product path.

### Этап 3 — довести application/service path

После появления всех 26 active rule ID:

1. Подключить partitions в существующую service composition в однозначном owner order.
2. Вывести полный результат через реальный `apps/wow graph build` lane.
3. Проверить, что graph receipt содержит node/edge crosswalks и exact support для каждой новой relation.
4. Не строить final graph IDs внутри recognizer: recognizer выдаёт только proposals.
5. Не превращать structural evidence в runtime frame existence, event delivery, loaded library revision, combat/taint safety или Secret authority.
6. Затем продолжить W12 conflict/derivation и W13–W16 coherent publication/invalidation/retention/recovery.

### Этап 4 — только после функциональной готовности выполнить полные тесты

Когда весь выбранный функциональный W11/E2-B scope реализован, добавить и выполнить полный pipeline:

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

Минимальные обязательные cases:

- CreateFrame, CreateFromMixins, Mixin assignment;
- RegisterEvent и multi-unit RegisterUnitEvent;
- EventRegistry bridge с callback и без callback;
- custom producer+subscription, no producer и ambiguous producers;
- exact/dynamic CVar keys;
- SetScript, HookScript, обе формы hooksecurefunc, dynamic target;
- direct LibStub, GetLibrary, NewLibrary, reviewed embed и `Libs/`-only negative;
- все TOC/XML/state rules;
- shuffle/duplicate facts, budget/truncation, cancellation;
- producer replacement и disablement без повреждения чужих partitions.

Sidecar-only проверка callable key не считается rule test. Положительный case обязан подтвердить принятую graph entity/relation и exact evidence.

### Этап 5 — исправить фиктивные fixture associations и заморозить contract

Каждый active rule должен ссылаться на собственные реальные fixtures, а не на любой существующий `RECOG-*` ID. Для каждого правила нужны:

- positive;
- structurally similar near-negative;
- partial/incomplete case;
- dynamic/ambiguous case, где применимо;
- rename/path/local-identifier mutation;
- deterministic shuffled/duplicate-fact case;
- budget/truncation case;
- producer replacement/disable case.

После выполнения зафиксировать expected match/proposal/partition IDs, profile IDs и canonical SHA-256. Никогда не auto-bless fixtures после изменения поведения.

## Запрещённые сокращения

- Второй Lua/XML/TOC parser в recognizers.
- Regex, имя репозитория, путь `Libs/` или популярность как semantic condition.
- Receiver из `arguments[0]`.
- Graph proposal без source/evidence support.
- `Derived` из dynamic target или ambiguous producer.
- Clean negative из Partial/NotEvaluated coverage.
- Runtime claims из static structure.
- Подмена отсутствующего fixture любым существующим fixture ID.
- Возврат Python/interpreter-based producer path.

## Обязательный финальный сравнительный gate: WoW API Ketho MCP

После того как собственный полный pipeline и fixtures проходят, необходимо выполнить отдельное сравнительное тестирование через **WoW API Ketho MCP**. Без этого W11/E2-B нельзя объявлять semantic-complete.

Обе стороны должны использовать один и тот же:

```text
WoW flavor
moving selector, разрешённый один раз в exact revision
Gethe source revision
admitted addon corpus
normalization profile
```

Сравнивать нормализованные данные, а не текстовые описания:

1. resolved callable и colon/non-colon form;
2. argument positions/varargs `RegisterUnitEvent`;
3. EventRegistry native bridge против custom producer/subscription;
4. CVar callback registration;
5. SetScript, HookScript и обе формы hooksecurefunc;
6. LibStub require/new/embed и version handling;
7. source spans, owner/caller identity, confidence, omissions и blockers;
8. итоговые entity/relation sets на одном corpus.

Сформировать discrepancy matrix:

```text
case
framework result
Ketho MCP result
exact Gethe evidence
runtime evidence, если требуется
classification: framework bug | Ketho difference | unsupported | unresolved
chosen action
```

Ketho MCP — обязательный comparative oracle и implementation donor, но не право молча перезаписать факты. Current exact Gethe source остаётся source authority; runtime-sensitive расхождения остаются unresolved до named-client runtime probe. Все расхождения сохранять, fixtures автоматически не переписывать.
