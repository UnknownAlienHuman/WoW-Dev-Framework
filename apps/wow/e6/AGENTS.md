# AGENTS.md — `apps/wow` E6-B

## Scope

Implement strict E6-B CLI parsing, explicit transport input, exactly one service call, output framing, cancellation, and exit mapping only.

## Dependency

```text
apps/wow -> wow-service
```

Any direct import of `wow-cbm`, project, reference, context, store, provider adapters, credential systems, or another framework crate is an architecture failure.

## Command discipline

- Use only the frozen command/option grammar.
- Unknown commands/options/config fields fail before service invocation.
- Construct exactly one typed E6-B request.
- Invoke exactly one service operation.
- Never sort/select by rank, score, top, first, best, last, sole, name, path, snippet, or provider label.
- Never refresh external state or current on continuation/reconcile.
- Never retry automatically after response loss, broken pipe, output failure, or cancellation.

## Input discipline

- Explicit bounded JSON file or stdin only where declared.
- Maximum one stdin consumer.
- No cwd/home/environment/Git/editor/WoW/repository/network discovery.
- No include, interpolation, shell expansion, plugin, script, archive extraction, or media sniffing.
- Paths are transport paths, never semantic mapping selectors.
- Raw credentials, tokens, cookies, private endpoints, session handles, provider cursors, and database paths are forbidden.

## Provider discipline

- Transport provider descriptor/profile references only.
- No raw MCP JSON-RPC, tool name, endpoint, or connection flag.
- No provider install/start/stop/update/configure/index/import/delete commands.
- Do not infer provider authorization from OS/GitHub/CLI identity.

## Mapping and selection discipline

- Transport exact result/candidate/locator/owner-publication/mapping IDs.
- Never open provider paths/URLs.
- Never perform local source lookup or fuzzy mapping.
- Selection requires exact mapping receipt/root and explicit origin.
- There is no `--top`, `--best`, `--first`, `--sole`, or score-threshold auto-select option.

## Context discipline

- `external context` transports exact selection/root/context profiles to one service operation.
- It does not read source or invoke context/project/reference directly.
- Text output must keep provider evidence separate from framework context facts.

## Output discipline

- `envelope-json`: exact canonical service bytes plus one LF.
- `artifact`: exact one eligible artifact without wrapper/newline changes.
- `text`: faithful projection preserving Candidate/nonclaims, external-state, mapping, selection, partial/conflict/truncation/`OutcomeUnknown`, privacy/license, and closure state.
- stdout contains requested output only; stderr is bounded diagnostics.
- Broken pipe/output failure never triggers a second service call.

## Completion report

```text
command and exact service operation
explicit input sources and limits
one request/operation/result identity
provider/external-state/mapping/selection/context refs
stdout mode/bytes, stderr and exit code
cancellation/broken-pipe/output behavior
credential/privacy/license handling
tests and skipped/NotEvaluated implementation gates
```
