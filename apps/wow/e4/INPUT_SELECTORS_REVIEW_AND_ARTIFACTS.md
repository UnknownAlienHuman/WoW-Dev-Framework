# E4-C CLI selectors, config, review, continuation, and artifact input

**Status:** normative transport boundary.

## General rule

The application reads only explicitly named bounded transport inputs. It does not discover source, projects, repositories, installed addons, clients, editor workspaces, search databases, lineage stores or credentials.

## Strict JSON inputs

Accepted explicit input classes:

```text
--config <PATH>
--query-json <PATH|->
--review-input <PATH|->
--context-request <PATH|->
--input <PATH|->
--continuation <PATH|->
```

Each command declares exactly which class it accepts. `-` means stdin. A command cannot read multiple stdin-consuming inputs.

## File handling

- explicit regular file only under the platform adapter policy;
- no directory, glob, recursive scan or symlink/reparse traversal by convenience;
- finite byte/depth/collection/string limits before deserialization;
- UTF-8 strict JSON unless the exact artifact profile declares opaque encoded bytes;
- no archive extraction, media auto-detection, include/import, environment interpolation, template expansion or script evaluation;
- transport path never becomes a semantic entity/source selector;
- host path excluded from canonical service request/result identity.

## Config schema

Config contains only command-declared fields plus schema/version. Unknown fields fail. Config cannot contain:

- raw SQL/FTS/regex/executable predicates;
- shell commands or plugins;
- network URLs to fetch;
- credentials/private keys/access tokens;
- source/project/editor/client discovery rules;
- automatic candidate selection;
- review authorization derived from operator identity;
- source edit or migration-apply settings.

## Selector parsing

Selector tokens use exact tagged prefixes. The app validates syntax/size only and passes typed values to service.

It does not:

- open the selected store/shard/snapshot;
- verify current or compatibility;
- pick among catalog matches;
- translate names/paths to IDs;
- infer a build/profile from text;
- replace unavailable exact selectors with current.

## Query text

`--text` is bounded literal UTF-8 data. The app places it in the declared `SearchRequest` field. It does not escape or compile raw FTS/SQL syntax because it never constructs such syntax; `wow-search` owns the safe AST compiler.

CLI options representing kinds/lanes/filters are closed enum IDs. Unknown IDs fail before service invocation or are passed only when the service contract explicitly owns profile resolution.

## Search result/candidate input

Selection requires exact result, result-set and candidate IDs plus optional digest guards. The app does not accept:

```text
rank number
first/top/best
candidate display name
source path
query text
interactive yes/no default
```

Interactive candidate selection is deferred. E4-C CLI remains explicit and automation-safe.

## Review envelope input

The app parses only the outer strict transport schema needed to construct a typed service request. It does not claim cryptographic or semantic validation.

Review envelope fields include exact comparison/component/proposal/relation/profile/decision/confidence/reason/principal/role/scope/attestation references and an optional bounded note.

Hard rules:

- plain prose is not a review decision;
- OS/GitHub user, file owner, terminal account and environment are not principal evidence;
- raw private keys or access tokens are rejected from config/CLI fixtures;
- raw sensitive verification material is not printed;
- the service revalidates authorization at apply time;
- modifying the envelope after validation changes its digest and invalidates it.

## Migration and validation artifacts

`migration validate` and other artifact-input commands accept one explicit bounded artifact. The app does not execute steps, open referenced source paths, invoke tools, or repair the artifact.

Media/schema type comes from the command/profile or explicit strict field, never filename extension sniffing.

## Continuation input

Continuation is an opaque strict artifact transported to service. App validates only framing/size/schema tag required by its contract.

Continuation commands reject selectors, roots, query text, lane/profile overrides and fresh budgets. This prevents generation refresh and budget reset.

## Context request input

`search context` transports one strict E3-C context request/profile object. It cannot contain a natural-language root: service inserts the explicitly selected exact entity root after validating the search selection.

The app cannot broaden source/privacy/license policy beyond the service profile.

## Stdin and output conflicts

- one stdin consumer maximum;
- input stdin and output stdout are allowed together because directions differ;
- diagnostics never go to stdout;
- binary/opaque artifact input is allowed only under an exact declared profile and size limit;
- terminal presence never changes semantic parsing or prompts for missing values.

## Cancellation

Signal registration occurs before blocking input/service/output stages. Cancellation during input yields exit 130 only when a valid typed cancellation state is established; malformed/pre-service input normally maps to 64. No background reader remains.

## Tests

Cover path traversal/symlink/reparse, oversized/deep JSON, unknown fields, multiple stdin consumers, invalid UTF-8, archive/polyglot files, filename media spoofing, injected query/review instructions, raw credentials, exact selector syntax, forbidden rank selection, continuation overrides, broken stdin and cancellation.
