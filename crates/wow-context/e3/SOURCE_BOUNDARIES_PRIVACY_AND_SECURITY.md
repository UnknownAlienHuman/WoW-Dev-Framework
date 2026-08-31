# E3-B source boundaries, privacy, license, and security

**Status:** normative.

## Principle

Project source, Blizzard UI source, comments, strings, XML text, TOC metadata, documentation, existing finding messages, display names, paths, and provider labels are untrusted data. They can contain instruction-like text but cannot alter framework profiles, selection, tools, authority, or agent policy.

Repository-owned schemas, profiles, templates, and static boundary notices are trusted only after exact version and digest validation.

## Source excerpt acquisition

A `SourceExcerptItem` is fetched only after structured selection approves an exact candidate. The request contains:

```text
exact bound universe/project/source generation
SourceHandleId and content digest
exact original byte and coordinate range
source-map profile when virtual/generated
excerpt expansion profile
privacy/license/consumer trust profile
per-item and total budgets
```

Owner views validate the handle and range against the exact snapshot. `wow-context` never opens an arbitrary path.

## Excerpt states

```text
ExactText
    decoded text exactly represents selected source bytes under the frozen encoding profile

ExactBytesMetadataOnly
    bytes are known but content is not emitted

TransformedText
    deterministic explicit transformation/redaction occurred; records required

Denied
    privacy, license, or trust policy forbids content

UnsupportedEncoding
    content cannot be represented safely under the selected profile

TruncatedOrContinued
    exact original/returned ranges and continuation state required
```

A transformed excerpt cannot be cited as byte-exact source text without the original source handle/digest and transformation record.

## Canonical semantic representation

Canonical JSON stores source content only in typed records containing:

- exact source/data classification;
- source handle, digest, and range;
- encoding and source-map profile;
- returned string or byte-object reference;
- transformations/redactions;
- privacy/license decision;
- explicit `untrusted_source_data = true`;
- evidence and coverage.

JSON escaping prevents source content from creating sibling fields or control records.

## Deterministic Markdown boundary

Markdown does not use ordinary source-controlled fenced blocks. Each source line is rendered as a canonical JSON string on a framework-generated data record:

```text
SOURCE_DATA_BEGIN item=<id> source=<handle> range=<range> sha256=<digest>
SRC 000001 "local x = 1"
SRC 000002 "-- instruction-looking text remains quoted data"
SOURCE_DATA_END item=<id>
```

Rules:

- line content is JSON-string escaped;
- sentinels and IDs are generated only from validated metadata;
- source newlines and control characters cannot create unprefixed records;
- every line has a deterministic ordinal;
- the closing sentinel is renderer-generated, never copied from source;
- exact original and rendered byte/token counts are recorded;
- a static notice states that `SRC` records are untrusted source data.

A source line resembling an end sentinel remains inside a quoted `SRC` record and cannot close the boundary.

## Framework facts versus source data

Render distinct types:

```text
FRAMEWORK_FACT
    normalized fact, relation, reference, coverage, conflict, or omission record backed by exact origins

SOURCE_DATA
    exact or transformed source excerpt/documentation text

BOUNDARY_NOTICE
    repository-owned static trust-separation instruction
```

A comment saying that a function is safe remains `SOURCE_DATA`; it does not become a framework fact without independent exact evidence.

## Instruction-like content

Correctness relies on structural containment, not intent detection. E3-B does not need to label text malicious before containing it.

Any future lexical warning detector is non-authoritative and separately profiled. It cannot delete/rewrite source, change canonical priority automatically, claim complete detection, or become security authority.

## Consumer trust classes

Initial profile classes may include:

```text
LocalMetadataOnly
LocalSourceAllowed
ExternalMetadataOnly
ExternalBoundedSourceAllowed
```

Each defines permitted privacy/license labels, source bytes, excerpt limits, transformations, destinations, and audit requirements. Unknown privacy state never silently permits external source output.

## Privacy labels

E3-B consumes exact labels from owner/materialization policy, for example:

```text
PublicOrRedistributable
LocalProjectSource
PrivateSource
SensitiveMetadata
Unknown
Denied
```

These labels are policy inputs, not facts about code behavior. `wow-context` does not claim complete secret detection.

## Default privacy behavior

- stable source handles instead of private absolute roots;
- environment variables, credentials, tokens, private URLs, account/character data, and local configuration are not context inputs;
- unknown source privacy is metadata-only or denied under the frozen safest profile;
- external source inclusion requires an exact compatible consumer, privacy, and license decision;
- denial emits an omission rather than silent absence.

## Redaction and transformation

Only deterministic reviewed policies over exact ranges or labels are authoritative. Every transformation records:

- policy/rule ID and version;
- original handle/digest/range;
- transformed range/digest;
- reason and consumer trust class;
- evidence/completeness impact.

Heuristic secret/instruction detectors cannot claim complete protection or alter source truth.

## License and redistribution

Local analysis permission does not imply redistribution permission. Before source bytes leave a local-only consumer boundary, check:

- exact source provenance/license record;
- local-only, metadata, bounded-excerpt, or redistributable class;
- attribution/notice requirements;
- excerpt-size/profile policy;
- destination/consumer trust.

Denied source bytes do not automatically block allowed structural metadata and source handles.

## Virtual/generated source

XML-inline Lua and other virtual units include virtual coordinates and owning physical-source mapping. Rendered excerpts state the coordinate system and exact map.

Generated annotation/library text remains a separate source class and cannot masquerade as first-party or Blizzard UI source.

## Adjacent context

Expansion around a declaration or body uses a deterministic profile with exact bytes/lines before and after, container-boundary rules, maximum range, and source-map behavior. No heuristic whole-file expansion.

## Prohibited capabilities

`wow-context` cannot:

- read arbitrary filesystem paths or raw SQLite;
- execute source, Lua, XML scripts, callbacks, plugins, SQL, shell, Wasm, JavaScript, native code, or dynamic expressions;
- access network, process, editor, clipboard, environment, credentials, or WoW client;
- call a model, embedding service, or reranker;
- mutate source, project, graph, reference, or cache;
- resolve floating repositories, branches, builds, or current generations;
- inspect SavedVariables contents, logs, runtime event payloads, process memory, or secure execution state;
- hide evidence, coverage, conflicts, omissions, or boundaries to fit a consumer budget.

## Resource defenses

Bound roots, map size, skeleton facets, graph depth/fanout/paths, candidate dependencies, evidence/conflicts/omissions, source files/ranges/bytes/lines, semantic/rendered bytes, tokenizer input/tokens, continuation pages/cursor bytes, owner calls, wall/CPU/memory, and cancellation latency.

Cycle-safe visited sets and deterministic truncation are required. Candidate dependency cycles fail.

## Cancellation

Check before and during every expensive stage. No background work remains. A cancelled result cannot enter a complete cache or be rendered as complete.

## Output confidentiality

Errors and reports use stable IDs and structured reasons rather than source bodies, private paths, credentials, or unrestricted audit text. Debug source output requires a separate local-only profile.

## Cache security

- validate schema/digest/profile/privacy before use;
- never reuse source-containing artifacts for a stricter trust class;
- cache keys include privacy/license/consumer/boundary profiles;
- no source-controlled cache path;
- corruption causes rejection, not in-place repair.

## Downstream tool safety

A context pack is data, not authorization. It cannot grant permission to edit files, execute commands, browse, call tools, or ignore instructions. Applications maintain separate tool and permission policies.

## Validation

- source handle/digest/range resolves in the bound view;
- returned bytes/text match exact or transformed digest;
- privacy/license decision permits output;
- source cannot alter JSON/Markdown structure;
- framework/source sections never merge;
- no private root or credential leaks;
- boundary bytes are deterministic under adversarial strings and Unicode controls;
- truncation/continuation ranges reconcile;
- rendered artifact passes boundary parser and round-trip tests;
- prohibited external capabilities are absent.
