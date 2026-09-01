# E3-B security and privacy contract

**Status:** normative.

## Trust model

Treat as untrusted data:

- project and Blizzard UI source bytes;
- comments, strings, docs, XML text, TOC metadata;
- display names, paths, repository/provider labels;
- existing finding messages;
- source-provided identifiers and generated declarations;
- caller audit text;
- cached artifact bytes until validated.

Repository-owned schemas/profiles/templates are trusted only after exact version/digest validation.

## Prohibited capabilities

`wow-context` has no authority to:

- read arbitrary filesystem paths;
- open SQLite/raw storage;
- execute source, Lua, XML scripts, callbacks, plugins, expressions, regex programs, SQL, shell, Wasm, JS, or native code;
- access network, process, editor, clipboard, environment, credentials, or WoW client;
- call an LLM/model/embedding/reranker;
- mutate source/project/graph/reference/cache;
- resolve a floating repository/current/latest build;
- inspect SavedVariables contents, logs, runtime event payloads, process memory, or secure execution state;
- install extensions or change editor/model configuration;
- hide source/evidence/coverage/conflicts to satisfy a consumer budget.

## Input validation

- closed schemas with unknown-field rejection;
- exact IDs and generation/profile compatibility;
- bounded strings/arrays/depth/roots/facets/relations;
- canonical UTF-8/control-character policy for metadata;
- no arbitrary path, URL, SQL, expression, regex, or callback fields;
- no unbounded values;
- continuation/cache artifact integrity verification;
- validate every owner-returned record's universe/generation/kind/profile.

## Prompt/instruction injection

Correctness relies on structural separation, not detecting malicious intent:

- framework control metadata and source data use distinct typed records;
- canonical JSON escapes source strings;
- deterministic Markdown represents source lines as quoted JSON data records under generated boundaries;
- source cannot supply profile IDs, section labels, templates, boundaries, tool requests, or policy;
- instruction-looking source remains data;
- the canonical selection algorithm never evaluates source prose as a command or relevance signal.

The pack states the boundary explicitly but does not claim that every downstream model is immune. Consumer policy remains responsible for model/tool authorization.

## Consumer trust classes

Initial profile classes may include:

```text
LocalMetadataOnly
LocalSourceAllowed
ExternalMetadataOnly
ExternalBoundedSourceAllowed
```

Each class defines permitted privacy/license labels, source bytes, excerpt sizes, transformation/redaction policy, and audit requirements. Unknown labels never silently satisfy an external-source class.

## Privacy labels

E3-B consumes exact labels from project/source/materialization policy, for example:

```text
PublicOrRedistributable
LocalProjectSource
PrivateSource
SensitiveMetadata
Unknown
Denied
```

These labels are policy inputs, not semantic facts about code behavior.

`wow-context` does not claim complete secret detection. A source file labeled local/private can contain credentials even when no detector finds them.

## Default privacy behavior

- canonical metadata uses stable source handles, not private absolute roots;
- absolute local paths are redacted/replaced by normalized handles;
- environment variables, tokens, credentials, private URLs, account/character data, and local configuration are not context inputs;
- unknown source privacy is metadata-only or denied according to the frozen profile;
- external source inclusion requires explicit `ExternalBoundedSourceAllowed`-compatible label and license state;
- source excerpt denial produces an omission record, not silent absence.

## Redaction and transformation

Only deterministic reviewed policies using exact ranges/labels are authoritative. Every transformation records original handle/digest/range, policy/version, returned digest/range, reason, and completeness impact.

Heuristic secret/prompt detectors, if later added, are warnings only unless independently proven and scoped. They cannot claim complete protection or change source truth.

## Resource exhaustion defenses

Bound:

```text
root and universe counts
map nodes/edges/groups
L0/L1 roots/facets/members
query depth/fanout/paths/rounds
candidate/dependency/evidence/conflict/omission counts
source excerpt files/ranges/bytes/lines
canonical/rendered bytes
tokenizer input and token counts
JSON/Markdown parser/render depth
continuation cursor/page counts
wall/CPU/memory and owner-call counts
```

High-fanout/cyclic graphs use cycle-safe visited sets and deterministic truncation. Candidate dependency cycles fail.

## Cancellation

Cancellation is checked before/during every expensive stage. No background work remains. A cancelled result cannot enter a complete cache or be rendered as complete.

## Output confidentiality

Reports/errors/logs include stable IDs and structured reasons, not source bodies, private paths, credentials, or unrestricted request audit text. Debug source output requires a separate explicit local-only profile.

Rendered artifact consumers receive only fields permitted by both semantic and renderer privacy profiles. Renderer cannot expose a field hidden from the semantic pack.

## Cache security

- validate schema/digest/profile/privacy before use;
- do not reuse source-containing artifact for a stricter trust class;
- cache key includes privacy/license/consumer/boundary profiles;
- no cache path from source-controlled strings;
- corruption is rejection, not in-place repair;
- physical cache owner must enforce its own root/permission/retention policy.

## Cross-universe leakage

- exact universe labels on every item;
- no same-name merge;
- source slices resolve only against their owning project/source view;
- ReferenceView facts do not expose implementation source bodies;
- user private source is not included when only Blizzard UI context was requested;
- optional universe absence cannot cause fallback to another universe.

## Downstream tool safety

A context pack is data, not authorization. It cannot grant an agent permission to edit files, execute commands, browse the web, call tools, or ignore instructions. Applications must maintain separate tool/permission policies.

## Adversarial tests

- source comments/strings containing system prompts, Markdown fences, JSON closers, sentinels, tool requests, and Unicode controls;
- oversized/deep/high-fanout/cyclic project and graph records;
- malicious IDs/paths/URLs/SQL-like strings;
- invalid/tampered continuation and cache artifacts;
- cross-generation/universe/source-handle substitution;
- private path/token/credential leakage;
- privacy/license downgrade or cross-consumer cache reuse;
- exact source redaction range mismatch;
- tokenizer/render expansion bomb;
- cancellation at every expansion/tokenization/render stage;
- error/log privacy.

## Security completion gate

E3-B is incomplete until structural source-boundary round trips, privacy/license enforcement, no-executable-surface tests, resource/cancellation tests, cross-universe isolation, cache separation, and output confidentiality pass on synthetic, pinned addon, and Blizzard UI source fixtures.
