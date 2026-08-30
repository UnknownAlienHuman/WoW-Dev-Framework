# Agent workflow

**Status: operational**

The framework is designed to make the correct research order easy and the expensive context order difficult.

## 1. Establish state

Start with `wow_status` or the equivalent service call.

Confirm:

```text
selected profile, Interface, build, and source digest
reference generation and capability report
project generation and indexed roots
TOC variants and active flavor
optional Codebase Memory status/generation
known failed or partial partitions
```

Do not interpret a search miss until the relevant coverage is known.

## 2. Read project structure and current KB routing first

For every World of Warcraft engineering task:

1. read the current [`AGENTS.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/AGENTS.md) and [`INDEX_MINI.md`](https://github.com/UnknownAlienHuman/wow-addon-engineering-kb/blob/main/INDEX_MINI.md) in the separate WoW Addon Engineering Knowledge Base;
2. select the current task-specific route from that index;
3. for a concrete addon, resolve the actual repository under [UnknownAlienHuman repositories](https://github.com/UnknownAlienHuman?tab=repositories);
4. read that repository's instructions and TOC files;
5. identify first-party code, declared libraries, XML, SavedVariables, and load-on-demand units;
6. obtain the generated Project Map;
7. locate the owner/load chain for the target subsystem;
8. search existing abstractions before designing a new one.

The knowledge base provides living WoW research and routing. Project behavior still comes from the actual repository and selected source profile. This framework links to the KB and does not duplicate its changing patch, security, field-note, or upstream-bug content.

## 3. Resolve current platform facts

Recommended order:

```text
exact current lookup
→ deprecation/replacement/lineage lookup
→ owner/load/object/event/state trees
→ L0/L1 skeleton
→ exact source span when required
```

A missing exact symbol is not a replacement request until negative authority is established.

## 4. Form a plan

`wow_plan` should return:

- target entities and chains;
- current API/restriction contracts;
- smallest required source handles;
- expected files and responsibilities to change;
- known dynamic or coverage gaps;
- diagnostics and tests required;
- runtime scenarios that static analysis cannot settle.

A plan distinguishes facts from candidates and does not invent a workaround before locating the correct extension point.

## 5. Use external implementations late

Only after current Blizzard/project facts are understood:

1. use exact GitHub search or optional Codebase Memory to select candidates;
2. inspect repository license, commit, TOC/bootstrap, and target subsystem;
3. compare universal structural patterns;
4. convert relevant paths/spans into stable source handles;
5. revalidate every patch-sensitive API, hook, template, and restriction against the selected Reference Pack;
6. copy code only when license and independent implementation policy permit it.

External code is implementation evidence, not platform authority.

When repository writes are required, follow the GitHub connector capability-verification procedure in [`AGENTS.md`](../AGENTS.md). Lack of local network access, Git credentials, or an authenticated `gh` session does not establish that connector write actions are unavailable.

## 6. Implement the smallest coherent change

- preserve current architecture and project conventions;
- avoid new abstractions when an existing owner is correct;
- keep lifecycle, cleanup, and profile behavior explicit;
- add a fixture or test that proves the changed path executed;
- avoid speculative compatibility branches without a selected target profile;
- do not suppress a diagnostic to hide an unresolved contract.

## 7. Check

Run `wow_check` and relevant project commands against the fresh tree.

The report should separate:

```text
generic Emmy diagnostics
WoW diagnostics
root-cause groups
NotEvaluated rules and missing capabilities
profile/generation identity
new or changed coverage gaps
```

Then run format, lint, tests, schema validation, deterministic output comparison, and task-specific evaluations as available.

## 8. Runtime review

Static analysis may produce required runtime scenarios. A runtime review record includes:

- exact client build/profile;
- addon revision;
- restriction/combat/group state;
- reproduction steps;
- raw logs/probe output;
- observed result;
- whether the static conclusion was confirmed, contradicted, or remained inconclusive.

Never fabricate runtime confirmation.

## 9. Context budget

Default task context should include no more than:

```text
Project Map (approximately 2 KB)
target owner/load chain
relevant L0/L1 skeletons
current API/restriction contracts
known dynamic/coverage gaps
required checks
external source handles only when requested
```

Read discipline:

- search and outline before source;
- L0/L1 before L2;
- no full Blizzard file unless exact spans are insufficient;
- no whole external repository dump;
- every material conclusion carries a source/evidence handle.

## 10. Completion report

A concise completion report states:

```text
what changed
which contract and profile were used
which source handles supported the change
commands run and pass/fail/skipped status
fixtures and evaluations added
runtime verification performed or still required
remaining NotEvaluated capabilities or coverage gaps
```

A completed implementation task must not leave the user with an unlabelled candidate or an implicit profile assumption.
