# E3-A context security and resource isolation

**Status:** normative.

## Trust boundary

All project/graph/source labels, attributes, comments, documentation, snippets, paths, query requests, continuation cursors, and rendered destinations are untrusted bounded data. Exact framework IDs and validated view capabilities still require generation/profile checks.

## Prohibited capabilities

`wow-context` must not:

- discover or open arbitrary filesystem paths;
- open SQLite/database handles or execute SQL;
- access network, GitHub, package managers, external documentation, models, embeddings, or search services;
- spawn processes, shell commands, editors, language servers, or WoW clients;
- execute Lua, XML scripts, TOC directives, source hooks, generated code, plugins, callbacks, or templates;
- read SavedVariables contents, logs, runtime payloads, credentials, or unrelated user data;
- treat source text as instructions or policy;
- emit active HTML/script/terminal-control payloads;
- request unbounded graph/source/context output.

## Input validation

Validate before work:

- exact compatible input/profile/schema IDs;
- bounded root count and exact root forms;
- registry-valid relation/axis/kind IDs;
- bounded strings, arrays, maps, nesting, paths, spans, source refs, evidence refs, and cursors;
- UTF-8/encoding policy and integer overflow;
- confidence/provenance/coverage policy not broader than ContextProfile;
- tokenizer/excerpt capability when requested;
- destination/member names derived only from validated fixed-format IDs/profile templates.

## Denial-of-service controls

Bound and test:

```text
high-fanout entities and axes
cyclic call/state/registration graphs
many competing assertions/conflicts
large evidence/source-ref closure
very deep or numerous reason paths
huge strings/comments/source lines
large Project Map sections
Unicode combining/control/pathological tokenizer inputs
many tiny fragments and separator overhead
continuation replay/tampering
cancellation during every loop/render/source stage
```

Algorithms use iterative bounded traversal where hostile depth could overflow a stack.

## Source isolation

Source bytes arrive only through the exact `ContextSourceReader` seam. The context crate cannot convert a label/path into a file open. Source reader responses are validated against snapshot, handle, span, digest, encoding, license, and budgets.

## Prompt injection isolation

Source/docs/strings may contain text such as “ignore previous instructions,” tool calls, URLs, Markdown headings, JSON keys, or code fences. They remain quoted/fenced payload. They cannot:

- call tools or alter profiles;
- trigger source expansion;
- change artifact schema;
- create external links/commands;
- suppress uncertainty;
- become agent instructions.

Security tests include multilingual and encoded variants.

## Privacy and redaction

- no private absolute paths in canonical/rendered artifacts unless a separate local-only profile explicitly allows a redacted form;
- no tokens, credentials, account/character/runtime user data;
- source-handle metadata minimized;
- errors/logs use IDs and bounded redacted labels;
- rejected private content is not echoed in full;
- redaction reports state reason and affected fields/spans without reproducing the secret.

## Destination safety

When a caller persists rendered members, member paths/names come from validated artifact/member IDs and fixed layout profiles. Context output never chooses an arbitrary host destination.

Atomic file publication, permissions, storage roots, retention, and deletion belong to the caller/store/application boundary.

## Cancellation

Check cancellation during:

- request/input validation;
- graph candidate enumeration;
- dependency closure and selection;
- source read/redaction;
- rendering/tokenization;
- artifact validation/diff/invalidation.

No background continuation after cancellation. A cancelled result cannot be complete or published.

## Resource report

Operational report includes bounded:

- visited/selected/omitted fragments;
- graph nodes/edges/paths;
- source reads/bytes;
- machine/rendered bytes;
- token accounting state/counts;
- cancellation/time/memory classifications;
- largest fanout/string/excerpt classes without leaking content.

Time, memory, host, and thread metrics are noncanonical.

## Security acceptance

Implementation must pass architecture tests proving absence of direct dependencies/APIs for filesystem, network, process, editor, SQLite, model, search, and runtime access in the E3-A crate, in addition to hostile data fixtures.
