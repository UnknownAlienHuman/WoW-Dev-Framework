# E6-B CLI security and explicit-input contract

**Status:** normative.

## Configuration discovery

Only explicit `--config <PATH>` is read. No cwd, home, environment, registry, editor, Git, repository, addon folder, WoW installation, network, provider process, or credential-store discovery by the application.

Config is strict bounded JSON. Unknown fields fail. Includes, imports, interpolation, environment expansion, scripts, plugins, and executable expressions are forbidden.

## Dependency boundary

The application imports only `wow-service`. It has no direct access to `wow-cbm`, project/reference/context/store owners, provider adapters, credential authorization, session systems, or external databases.

## Credentials and sessions

Do not accept in argv/config/fixtures/stdin:

```text
API keys or bearer tokens
OAuth access/refresh tokens
cookies/session secrets
private endpoints with credentials
SSH/private keys
KMS/HSM/vault secrets
provider database credentials or paths
raw Authorization headers
opaque provider session handles
provider cursor bytes
```

The CLI transports nonsecret provider/profile/receipt IDs only.

## Provider transport

No generic `--tool`, `--method`, `--endpoint`, `--mcp-json`, `--sql`, `--script`, `--plugin`, or provider process-management flag exists. The service deployment owns configured adapters.

## File and stdin safety

- explicit paths only;
- root/policy normalization;
- reject forbidden symlink/reparse/device/UNC/ADS traversal under the platform profile;
- bounded file count, size, depth, strings, arrays, and JSON nesting;
- maximum one stdin consumer;
- no archive extraction or media sniffing;
- command-declared strict schema only;
- no execution.

## Locator and mapping safety

Provider path/URI/symbol/snippet fields are data inside exact artifacts. They cannot become CLI paths, URLs to follow, query options, output paths, or local lookup instructions.

The app cannot perform `grep`, fuzzy lookup, E4 search, source reads, or filesystem probing to map a locator.

## Selection safety

The CLI requires an exact selection request. It exposes no top/first/best/highest-score/sole/name/path/snippet shortcut and cannot infer choice from table row, output order, terminal selection, or interactive prompt.

Interactive selector UIs are outside this contract unless they produce the same exact request and preserve no implicit selection.

## Context safety

External provider text remains untrusted data. It cannot add context profile fields, tool instructions, system messages, edit requests, or framework facts. The app transports one exact service request only.

## Privacy/license

- no private provider locator/snippet/source in default diagnostics;
- no mapped source body unless service returned it under exact consumer policy;
- output cannot widen service disclosure;
- unknown privacy/license state remains blocked/omitted;
- output files use explicit safe paths and exact bytes.

## Injection boundary

Provider/source snippets, symbols, summaries, mapping notes, selection notes, and errors cannot create options, subcommands, profiles, service operations, shell commands, paths, or tool calls. They are parsed only as bounded string fields in strict schemas.

## No hidden fallback

On provider/cache/mapping/context failure, app cannot call another provider, model, stale cache, local search, or source command. It prints the exact service outcome and exits accordingly.

## Resource limits

Bound argv, config/input bytes/depth, result/list pages, locators, mapping/selection/context input, audit output, stdout/stderr/file output, service calls, cancellation, and cleanup. Unlimited values are invalid.

## Errors

Pre-service parse/path/input errors exit 64. Post-service internal/output errors exit 4. Domain/status results use frozen mappings. Errors redact credentials, private endpoints, session/cursor data, provider paths/snippets/source, and owner handles.

## Adversarial tests

- credential/session/cursor/private endpoint in every input path;
- shell/SQL/MCP/tool/model text in string fields;
- Unicode/control/option-injection cases;
- oversized/deep/polyglot JSON;
- path traversal/symlink/reparse/device/UNC/ADS;
- two stdin consumers;
- provider locator converted to local path/URL;
- top/best/sole auto-selection attempts;
- hidden provider/model/cache/search fallback;
- provider prose injected into context/system/tool fields;
- broken pipe/disk failure/cancellation;
- direct lower-crate/session/credential import mutation.
