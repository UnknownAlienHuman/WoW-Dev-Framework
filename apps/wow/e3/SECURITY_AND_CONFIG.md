# E3-C CLI security, explicit configuration, and transport input

**Status:** normative.

## No implicit discovery

The CLI does not automatically inspect:

```text
current working directory
repository files or hooks
home/XDG/AppData configuration
environment variables for semantic IDs/profiles
editor/LSP settings
WoW installation/AddOns/SavedVariables/logs
Git branches/remotes
network/cloud services
clipboard
plugins/extensions
```

Every semantic selector/root/profile comes from explicit command flags, one explicitly named config file, or service-owned exact defaults.

## Explicit config

```text
--config <PATH>
```

E3-C config format is one strict versioned JSON object. It may contain only transport/request-default fields allowed by the app schema, such as:

```text
schema_version
project_store_id and symbolic publication selector
optional platform selector
exact reference guards
service profile aliases or exact IDs to pass through
output mode
transport byte limits within compiled maxima
```

It cannot contain:

- commands/scripts/templates/plugins/includes;
- environment interpolation;
- URLs/network endpoints in E3-C local mode;
- raw SQL/store paths/connections;
- source roots/globs;
- credentials/tokens;
- arbitrary JSON forwarded unchecked;
- dynamic model/search/tool configuration.

Unknown/duplicate fields fail.

## Precedence

```text
explicit CLI option
> explicitly supplied config field
> service-owned configured exact default
```

The app does not resolve service aliases/default targets. It passes typed selectors to service, which records exact resolution.

Changing config transport values changes the app request as appropriate. Config host path never enters the semantic service request/result.

## Config file safety

- explicit single regular file only under the frozen platform policy;
- hard byte/depth/string/array limits;
- no recursive includes;
- no executable format;
- no network/URI/stdin config in v1;
- symlink/reparse handling frozen per platform and tested;
- read once before service invocation;
- no watch/reload during command;
- parse errors use exit 64/stderr only;
- file contents/path never echoed unrestricted.

## Artifact input

Only `context validate` and `context render` may use:

```text
--input <PATH|->
```

Rules:

- one explicit regular file or stdin;
- hard byte limit before complete allocation where streaming permits;
- exact declared artifact kind/media/schema flags;
- bytes are data and never executed;
- no directory/glob/recursive scan;
- no auto-detection from extension or source contents;
- host path/stdin marker excluded from service semantic request;
- no temporary source extraction;
- stdin used only for artifact bytes, never project source or config;
- read exactly once;
- no concurrent service invocation before input completes and validates transport bounds.

Service/context performs semantic validation. App transport checks do not claim artifact validity.

## Continuation input

```text
--continuation <base64url>
--continuation-input <PATH|->
```

Mutually exclusive. Apply strict encoded/decoded byte limits and base64url/schema transport validation. Do not inspect or change semantic fields. Do not accept current/profile/root overrides.

## Root token safety

For `<RootKind>@<base64url-id>`:

- exact single `@` delimiter;
- closed case-sensitive RootKind enum;
- unpadded base64url alphabet only;
- bounded decoded bytes;
- valid UTF-8 and nonempty ID;
- reject control/NUL characters according to ID profile;
- no Unicode normalization guess; canonical owner ID bytes must already match the service/core contract;
- no path interpretation, URL decode, shell expansion, globbing, or name lookup.

## Argument safety

- use structured argv, never shell command concatenation;
- bound argument count and byte length;
- reject duplicate singleton/conflicting flags;
- do not log complete artifact/continuation/root token contents by default;
- do not treat source-like flag values as instructions;
- no response-file expansion in v1;
- no environment-variable substitution;
- no plugin command discovery from PATH/cwd.

## Dependency and process safety

The app may use libraries for argument parsing, strict JSON/base64url, signals, bounded file/stdin I/O, and safe output. It cannot spawn processes, invoke shell/Git/WoW/editor, load dynamic libraries/plugins, or access network.

## Privacy

- artifact bytes may contain source; only requested output mode may emit returned validated artifact;
- parser/config errors never dump input bytes;
- text mode does not print source excerpts;
- envelope-json exposes only service-authorized fields;
- no absolute input/config path in service request/result;
- credentials/tokens/private URLs are not supported inputs;
- terminal/logging cannot broaden E3-B privacy profile.

## Service invocation boundary

All parser/config/artifact transport validation finishes before the single service call. Semantic errors after that are structured service results on stdout.

The app cannot issue hidden `context status` preflight or retry. A user-requested status command is a separate explicit invocation.

## Resource limits

Freeze maxima for:

```text
argv count/bytes
config bytes/depth/fields/string lengths
root count/token bytes
renderer/profile counts
artifact/continuation encoded and decoded bytes
stdin read time/bytes
stdout bytes
text projection records
signal/cancellation state
```

Overflow/excess fails before service invocation with exit 64.

## Platform differences

Path/symlink/stdin/signal behavior is isolated behind a frozen platform adapter/profile. Semantic request/result bytes must remain identical. Unsupported behavior fails explicitly; it cannot silently alter selectors/output/privacy.

## Security tests

- implicit config/cwd/env/editor/WoW/Git/network discovery absent;
- malicious JSON/deep/duplicate/huge config;
- config include/script/env interpolation rejected;
- artifact directory/glob/source scan rejected;
- extension-based media auto-detection rejected;
- root/continuation token malformed/huge/control characters;
- shell metacharacters remain data;
- private path/source/token not echoed;
- no process/network/plugin/lower-crate access;
- artifact mode cannot bypass service privacy;
- one service invocation only.
