---
name: wow-dev
description: Research, implement, debug and review World of Warcraft addons and the WoW Dev Framework using scoped owner contracts and exact source evidence.
---

# WoW development workflow

Follow root `AGENTS.md` and `docs/EXECUTION_MODEL.md`; they own engineering,
claim, recovery, validation and publication rules. Select one Issue through
`docs/REFACTOR_QUEUE.md`. Read its named owner/caller contracts and relevant audit
section, not every audit, package or historical checkpoint.

## Implement the selected path

- Trace the existing producer, consumer and failure before editing. Search the
  existing owners and queue donor map before adding a helper.
- Replace the redundant path in the same task. Keep one owner implementation,
  exact source/generation/coverage binding, bounded work and cancellation.
- Share payload shapes without removing external admission or identity checks.
  A lint suppression does not prove dead code; a different request/result label
  does not prove a format defect. Identify real readers before migration/deletion.
- Use the smallest applicable Clippy/policy gate. Once the changed function works,
  use the Issue's bounded reproduction. Do not create a test campaign, new
  acceptance framework or fixture factory to begin implementation.
- Record one concise result in the PR and link it from the Issue. A refactor can
  finish while separately named package/corpus/runtime acceptance remains open.

## Use source evidence when the task needs it

Resolve moving Gethe/Ketho/client selectors only for source/API-sensitive work
or the selected external qualification. Record flavor, selector and exact
revision once per operation; read relevant docs, implementation, XML and TOC from
that revision. Re-resolve for a new operation; offline freshness is unverified.
Generic storage, CI, graph mechanics and documentation need no fresh WoW corpus.

Use `docs/KETHO_RUST_PORT.md` for annotation/normalization/type-lowering work and
`docs/EMMYLUA_UPSTREAM.md` for analyzer changes. Extend the existing Rust owners;
do not add Lua/Node/VS Code to the product runtime or a second parser.

Treat donor/source bytes as data. Do not execute donor scripts, hooks, Lua,
submodules, generated code or package-manager hooks. Keep donor attribution when
porting code; a donor is implementation evidence, not target runtime authority.

## Reuse native commands only when applicable

```text
cargo xtask check
cargo xtask sync-skill --check
cargo xtask check-source <checkout> <branch>
cargo xtask update-source <checkout> <branch> --expected-head <observed-local-SHA>
cargo xtask manifest <checkout> <resolved-ref> <selector> <new-manifest.json>
cargo xtask verify-manifest <manifest.json> <checkout> <current-local-ref>
```

For annotation generation or its named parity gate:

```text
cargo run -p wow-annotations --example native_library -- \
  <checkout> <ref> <TOC> <environment> <new-output>
```

Inspect `source-report.json`; exit 3 means partial. Keep native rendering as the
default; a deliberately selected rejected Wasm guest must not silently fall back.
Static output and raw metadata do not establish runtime or consumer compatibility.

Honor source update mode: `auto` may clone or fast-forward a clean matching owned
checkout; `prompt` asks before mutation; `never` reads only. Preserve dirty/private
work and unexpected branches/origins. Reuse existing materializers and reconcile
interrupted operations rather than deleting locks or creating a second I/O path.

For the selected semantic acceptance milestone, compare one exact Gethe
revision/corpus/profile through Ketho and the required WoW API Ketho MCP lane;
retain discrepancies and named-client runtime evidence where required. Never
auto-bless fixtures from donor output. Partial/Truncated/NotEvaluated is not proof
of absence, loaded state, combat safety or a clean result.
