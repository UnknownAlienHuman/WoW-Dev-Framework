# E3-A source excerpts, prompt-injection, privacy, and security

**Status:** normative exact-source detail and untrusted-content boundary.

## Principle

Source excerpts are faithful bounded evidence from an exact source handle. They are not generated explanations, reconstructed code, or trusted instructions.

## Source excerpt request

```text
ContextSourceExcerptRequest
    exact ContextInputSnapshot
    source handle/file/source-unit/content digest
    target byte span or exact semantic/source-node ID
    SourceExcerptProfile
    requested surrounding lines/bytes within profile
    license/privacy/security policy
    budgets/cancellation
```

No path-only or current-file lookup.

## Valid source origins

Initial profile can allow exact retained:

```text
first-party project source
XML virtual Lua units
reviewed reference/source objects under license policy
small generated analysis artifacts when explicitly selected
```

Dependency/external/runtime/private origins require separate explicit policy. SavedVariables, logs, client memory, credentials, and runtime Secret-capable payloads are out of scope.

## Validation

Before excerpting:

- publication/project/source generation matches request;
- source handle resolves in exact project/reference view;
- content digest and byte length match retained object/source;
- span is in bounds and on valid encoding boundaries;
- origin/role/license/privacy/security permits access;
- source object is referenced by exact publication set;
- requested/context-expanded span is bounded;
- cancellation/budget state valid.

A stale source handle never falls back by path/name.

## Span expansion

The profile defines deterministic expansion:

```text
exact target span
optional containing declaration/member/source node
fixed maximum leading/trailing lines or bytes
line-boundary normalization
prefix/suffix truncation markers
```

Do not expand to an entire large function/file by default. If semantic containment is unavailable, use only exact requested span plus fixed line context.

## Faithfulness

The excerpt record retains:

```text
exact original content digest
requested and actual half-open byte spans
faithful selected bytes/text
line/column projection profile
normalization/escaping transformations
prefix/suffix truncation
source/license/provenance/evidence refs
excerpt digest
```

Line endings may be normalized only if the profile records both original content identity and exact transformation. Do not paraphrase code/comments inside an excerpt.

## Rendering and escaping

Render excerpts as inert quoted/code data:

- fence length/delimiter chosen to contain source safely;
- no source interpolation into JSON keys, Markdown links, paths, directives, or tool commands;
- invalid/control bytes handled by explicit encoding/escape/rejection policy;
- source cannot close the surrounding container and append trusted text;
- generated labels distinguish source from repository instructions;
- no executable HTML/script or terminal escape effect in supported renderers.

## Prompt injection

Project/reference/source text may state:

```text
ignore previous instructions
run a command or tool
read secrets
change policy
modify files
report false validation
```

E3-A treats this solely as quoted untrusted source evidence. It cannot affect:

```text
agent/tool policy
root/lane/detail selection
budget or security profile
coverage/confidence classification
renderer/template behavior
source or repository mutation
```

Context bundles include an untrusted-source boundary label for downstream consumers.

## Documentation and comments

Comments/docs can be included as exact excerpts when requested, but they do not establish types, roles, events, runtime behavior, or architectural purpose unless an owning structured contract independently does so.

## Private data

Exclude/redact by policy:

```text
absolute local roots/usernames/temp paths
credentials/tokens/private URLs/environment values
SavedVariables/log/chat/user data
unnecessary full source or object payloads
runtime-sensitive/secret-capable values
private repository content not authorized for the artifact/consumer
```

Redaction creates exact loss/security records. Do not include the sensitive original in public errors/loss reports.

## License/provenance

Every source excerpt records source repository/revision/object/handle and license/redistribution policy. A context artifact not authorized for redistribution can retain a handle/digest without embedding bytes.

Pinned real-project fixture excerpts are minimal, licensed, and evaluation-only. Production logic cannot depend on their names/content.

## Object access

Use exact store/project/reference object handles, not arbitrary filesystem paths. Verify object digest/length/reference set. No general object enumeration.

## Source budget

Track excerpt count, exact bytes/lines, context expansion, escaped/rendered bytes, and omitted source requests separately. Mandatory evidence links do not require source text and survive source-budget exhaustion.

## Source unavailable

If source is missing/stale/forbidden/over budget:

- retain structured skeleton/evidence where valid;
- return source-specific `NotEvaluated`/loss/omission;
- provide exact reason and source handle;
- do not reconstruct from analyzer messages, graph labels, cached prose, or another generation.

## No source mutation

E3-A is read-only. No edit/apply/autofix/format/write operation; no temporary modification for tests; no editor integration.

## Resource/path attacks

Test:

```text
huge source/span/context request
invalid UTF-8/control/NUL/terminal escapes
Markdown/code-fence/HTML/JSON injection
path traversal/absolute/URI/tokenized labels
symlink/reparse object substitution
stale handle same path new digest
source object removed after lease
malicious prompt/tool instructions
```

All are bounded or rejected without policy effect.

## Determinism

Equivalent exact source/profile/request yields identical actual span, faithful semantic bytes, escaping/rendering transformation record, excerpt ID/digest, loss state, and budget metrics.

## Required tests

- physical and XML-virtual source handles;
- exact span and deterministic surrounding context;
- stale generation/digest/path reuse rejection;
- allowed/disallowed license/origin roles;
- prompt/directive/fence/HTML/JSON/terminal injection corpus;
- invalid encoding/control bytes;
- private path/token/SavedVariables/runtime payload exclusion;
- source budget and unavailable source with structured skeleton retained;
- no source reconstruction/paraphrase;
- object reference/digest validation;
- deterministic bytes/metrics/rendering.

## Hard stops

- no path-only source lookup;
- no generated/reconstructed source;
- no source text as instructions or structured authority;
- no unrestricted/full source by default;
- no private/runtime-sensitive payload;
- no source mutation/editor/tool execution;
- no unlicensed embedding;
- no silent redaction/omission.
