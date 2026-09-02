# AGENTS.md — `apps/wow` E6-B

- Only framework dependency: `wow-service`.
- Parse strict explicit bounded commands/options/JSON inputs.
- Unknown commands, fields, selector forms, and output modes fail before service invocation.
- Pass exact configuration/descriptor/state/query/result/candidate/artifact/mapping/selection/context IDs and guards mechanically.
- Never resolve current/latest/default providers or owner generations locally.
- Never discover provider tools/processes/endpoints or read credential stores/environment secrets.
- Never accept secret material, arbitrary MCP/RPC/SQL/script/model fields, provider database paths, source paths to open, or generic tool names.
- Exactly one service call per valid command.
- Never choose top, sole, highest-score, nearest, same-name, first, newest, or most frequent candidates/mappings.
- Never map locators, inspect source, build context, or combine semantic evidence locally.
- JSON output is exact service bytes plus one LF; eligible artifact output is exact approved bytes; text preserves all Candidate/mapping/selection/context/partial/conflict/`OutcomeUnknown`/nonclaim state.
- Broken pipe/output failure never causes a second service call.
- No Cargo/Rust/workflow/placeholder implementation during documentation phase.