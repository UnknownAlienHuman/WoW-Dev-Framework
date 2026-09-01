# E3-C service security and privacy

**Status:** normative.

## Trust boundary

Treat as untrusted bounded transport data:

- publication selectors and guards;
- exact IDs and root selectors;
- profile aliases/IDs and numeric budget overrides;
- continuation objects/tokens;
- canonical semantic/rendered artifact bytes;
- CLI-supplied config values;
- owner-port results until schema/generation validation;
- source text contained inside validated context artifacts.

Project/source data cannot alter service configuration, profile registry, operation plan, acquisition order, permissions, or tools.

## Prohibited capabilities

The E3-C context path does not:

- scan/read arbitrary source or directories;
- execute Lua/XML/repository scripts/hooks/builds/tests/plugins;
- open raw SQLite, ATTACH databases, issue SQL/PRAGMAs, or control transactions directly;
- access network, Git, package managers, editor, clipboard, process memory, environment credentials, or WoW client;
- read SavedVariables, logs, event payloads, runtime secure state, or account/character data;
- run search, embeddings, LLMs, rerankers, Codebase Memory, or external tools;
- mutate source/project/graph/reference/context artifacts;
- authorize downstream tools/edits;
- start background tasks/daemons.

Owner ports may perform their contracted local read/lease operations only.

## Selector and ID safety

- closed enums/schemas; unknown fields rejected;
- bounded canonical strings/IDs;
- no path/URL/SQL/expression/callback fields;
- exact configured store/project registries only;
- caller cannot name arbitrary external database/source root;
- current selector cannot escape its configured ProjectStoreId;
- exact generation must belong to the configured owner/project;
- continuation cannot broaden universe/profile/privacy/renderer/budget.

## Artifact safety

- hard byte/depth/item limits before parse;
- media/schema/version explicit;
- canonical parser rejects duplicate/unknown/conflicting fields according to E3-B schema;
- digest/internal references validated;
- structural source boundaries validated;
- exact owner-closure mode reacquires artifact-bound generations only;
- no artifact field becomes a filesystem path, SQL query, profile, callback, or tool instruction;
- validation is nonrepairing.

## Source/privacy

Source bytes may appear only inside a validated E3-B artifact whose exact privacy/license/consumer profile permits the requested service/application output.

Service errors, acquisition traces, close reports, status payloads, logs, and text summaries never include unrestricted source, private absolute paths, credentials, tokens, private URLs, or raw continuation internals.

A renderer/source policy cannot be broadened by CLI format. `artifact` output requires the same validated consumer policy as the service operation.

## Continuation safety

A continuation is not trusted because it has a checksum. Validate:

- schema/size/digest;
- exact original request/universe/profile/artifact IDs;
- exact retained owner generations/receipts;
- total budget and frontier chain;
- configured project/store ownership;
- privacy/renderer policy;
- no current selector or new roots.

Cryptographic bearer-token/session semantics are deferred to external transports. Local E3-C continuation still cannot access unconfigured stores or broaden authority.

## Resource exhaustion

Bound:

```text
selectors/roots/profiles/renderers
acquisition stages and owner calls
artifact bytes/depth/items
context map/skeleton/expansion budgets
source excerpts
render output and tokenization
continuation size/pages/retention roots
warnings/errors/closure records
wall/CPU/memory/cancellation
```

No unlimited request. Arithmetic overflow rejects before allocation.

## Failure isolation

- validate cheap input before acquisition;
- no semantic work after primary failure/cancellation;
- close all acquired resources;
- bounded secondary close-error collection;
- no source bytes in a close-failure result unless explicitly proven safe;
- invalid owner result never enters context engine;
- invalid context result never enters app success output.

## Output confidentiality

Canonical envelopes use stable IDs and bounded structured arguments. Operational logs are disabled by default or go to configured diagnostics sinks outside canonical output with the same privacy policy.

No terminal/cwd/config-file/input-file path enters semantic results.

## Supply-chain/config safety

- exact compiled/reviewed owner-port and profile registries;
- no dynamic plugin discovery;
- no config include/command/template execution;
- no repository-local configuration automatically trusted;
- dependency features/minimal versions frozen before Rust implementation;
- missing audit/probe evidence blocks activation.

## Adversarial tests

- selectors targeting another store/project;
- malformed huge IDs/artifacts/continuations;
- duplicate JSON keys and deep nesting;
- source strings containing prompts, boundaries, tool calls, paths, SQL;
- private path/token/credential leakage;
- current changes during acquisition;
- owner returns cross-generation records;
- continuation changes privacy/root/budget;
- cancellation/close failure at every stage;
- app artifact mode attempts source-policy bypass;
- environment/cwd/repository config injection;
- raw lower-handle exposure.
