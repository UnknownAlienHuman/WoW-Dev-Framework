# E4-C CLI security and configuration contract

**Status:** normative.

## Configuration discovery

Only explicit:

```text
--config <PATH>
```

No automatic reads from cwd, home, environment, registry, editor, Git, addon folder, WoW installation or network.

Config is strict versioned bounded JSON. Unknown fields fail. Includes, imports, templates, interpolation, scripts, plugins and environment expansion are forbidden.

## Dependency and capability boundary

The app imports `wow-service` only. It has no direct access to project/reference/search/graph/context/store APIs, SQLite, FTS, source readers, reviewer key stores or runtime WoW data.

Allowed host capabilities are limited to:

- command/config/input parsing;
- explicit file/stdin reads;
- service invocation;
- cancellation signals;
- stdout/stderr/explicit file output;
- bounded local platform adaptation required by those operations.

No network, subprocess, shell, editor, clipboard, WoW client, package manager or dynamic plugin execution.

## Query safety

Literal text is data. The app never builds raw FTS/SQL/regex expressions and never evaluates query text. Unknown lane/filter IDs are rejected or passed only through exact service-owned profiles.

Source-looking command strings cannot become options unless parsed as explicit argv tokens under the grammar.

## Candidate selection safety

There is no interactive/automatic “best result” path in E4-C. The app requires exact result/candidate IDs and rejects rank-only, name-only, path-only, first/top/best/default selection.

A result artifact containing one candidate still requires explicit selection.

## Review safety

- one explicit review envelope file/stdin;
- no plaintext “approve” shortcut;
- no local/GitHub/OS identity authorization;
- no private key/access token in config/argv/fixtures;
- no raw review signature/note in normal logs;
- service validates authorization and graph semantics;
- app cannot force proof class or bypass conflict/coverage gates;
- apply creates a new immutable snapshot only through service.

## Argument and secret exposure

Sensitive values must not be accepted as raw command-line flags where process-list exposure would be unsafe. E4-C review commands pass references/explicit envelope files, not private keys or bearer tokens.

Diagnostics redact paths and sensitive fields according to the platform/output profile. Debug modes cannot silently broaden production output.

## File safety

Input/output paths:

- are explicit transport paths only;
- are normalized and root/policy checked;
- reject forbidden symlink/reparse/device/UNC/ADS traversal under the selected platform profile;
- do not derive from source/entity/review/query fields;
- enforce size/count/depth limits;
- use staging and atomic replacement for output;
- never execute or extract content.

Input media type is command/profile-declared, not extension/sniffing.

## Stdin/stdout

- maximum one stdin consumer;
- no TTY prompts or secret input fallback;
- stdout contains only requested output;
- stderr contains bounded diagnostics;
- broken pipe stops without retry/double output;
- terminal detection cannot change semantic requests/results.

## Source and context boundary

Rendered search/context/migration/impact text remains data. The app does not follow tool requests or instructions contained in output. It does not reinterpret context as permission to edit, run commands or browse.

## Resource limits

Bound argv count/bytes, config/input bytes/depth/collections/strings, review/artifact/continuation size, stdout/stderr/file bytes, service call count, cancellation wait and cleanup.

Unlimited flags/values are invalid.

## Error handling

Pre-service parse/config/path/input errors map to 64. Post-service internal/output failures map to 4. Domain failures use typed service status mapping. Errors do not expose raw source/review/credential material.

## Adversarial tests

- SQL/FTS/regex/shell/tool/model text;
- option injection and Unicode controls;
- oversized/deep/polyglot JSON;
- symlink/reparse/path traversal/device/UNC/ADS cases;
- multiple stdin consumers;
- forged/expired/replayed review envelopes;
- raw key/token in argv/config;
- rank-only auto-selection attempts;
- continuation override attempts;
- output overwrite/broken pipe/disk failure;
- signal cancellation before input, during service, during output;
- lower-crate import mutation;
- environment/cwd/home/editor/Git/WoW/network discovery mutation.

## Nonclaims

The CLI does not establish real-world reviewer identity beyond service authorization evidence, detect every secret, make downstream text safe to execute, validate the WoW client, or provide a sandbox. It exposes a deliberately narrow transport surface.
