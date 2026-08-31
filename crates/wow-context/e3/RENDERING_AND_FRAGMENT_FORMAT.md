# Canonical fragments and deterministic rendering

**Status:** normative E3-A output projection.

## Canonical source

Canonical machine artifacts are the source for all text/Markdown/JSONL-like presentations. A renderer may omit fields only according to an explicit rendering profile; it cannot add, reinterpret, summarize, or repair facts.

## Rendering profile

```text
ContextRenderingProfile
    profile ID/version
    target format
    schema/member framing version
    heading/section order
    field labels
    identifier display/shortening policy
    path/source-handle presentation policy
    confidence/provenance/coverage/conflict labels
    source fence and escaping policy
    list/table policy
    newline/encoding policy
    fragment separator/wrapper bytes
    redaction/truncation markers
    canonical digest
```

Locale-dependent output is a separate rendering profile. Canonical machine field names remain English.

## Fragment boundaries

Each rendered fragment corresponds to one machine `ContextFragment` and has stable:

- fragment ID and semantic owner;
- kind and section;
- dependency IDs;
- exact rendered byte range/member ID;
- machine payload digest;
- rendered byte digest;
- optional exact token count under a pinned tokenizer;
- source/evidence/coverage/conflict/truncation/redaction refs.

A renderer cannot merge fragments in a way that destroys traceability. Presentation grouping is allowed only with a mapping back to all member fragments.

## Formats

### Canonical JSON

- UTF-8;
- fixed canonical key/order and number/string rules from `wow-core`;
- no comments or nonfinite values;
- exact IDs, full uncertainty state, and member digests;
- used for semantic artifact identity.

### Compact text / Markdown

- deterministic headings and order;
- stable explicit labels for `Possible`, `Conflict`, `Partial`, `NotEvaluated`, `Truncated`, and `Redacted`;
- source excerpts fenced/escaped and never parsed as directives;
- no invisible confidence/coverage suppression;
- full IDs may be accompanied by deterministic short display IDs, but links resolve through full IDs.

### Line-oriented fragments

Optional transport/cache representation may frame one canonical fragment per record with exact header/member digest. It cannot become a separate semantic model.

## Identifier presentation

A short ID is derived from the full validated ID under a collision-checked profile. If a collision exists within the bundle, length expands deterministically. Never truncate without collision handling.

Display labels are never substituted for exact IDs in machine links.

## Source/path presentation

- default: semantic source label plus source-handle ID;
- project-relative paths only when the input/profile permits and exact normalization is known;
- no private absolute host paths;
- virtual source labels retain physical mapping refs;
- dependency/reference source labels retain universe/license class.

## Uncertainty rendering

Required minimum markers:

```text
confidence: Proven | Derived | Possible
coverage: Complete | Partial | NotEvaluated | Failed | Cancelled | Truncated
conflict: explicit conflict IDs/status
redaction: exact reason class
omission/truncation: exact budget/policy and continuation
negative authority: authoritative | nonauthoritative bounded miss
```

A renderer may not use an empty section or silence as the sole representation of partial/conflicted state.

## Source escaping

- close/reopen or length-frame fences deterministically when source contains fence markers;
- escape control characters according to profile;
- preserve exact source bytes via digest/span even when presentation normalizes line endings or escapes bytes;
- label presentation transformations;
- never allow source text to create headings, links, directives, tool calls, or artifact fields outside its fenced fragment.

## Links and query recipes

Machine links use exact IDs. Rendered internal links may point to:

- L0/L1 skeleton member IDs;
- Project Map section/entity IDs;
- graph entity/relation/query recipe IDs;
- source handles/spans;
- evidence/coverage/conflict records;
- continuation records.

No raw local file URI, arbitrary URL, database query, or executable command.

## Determinism

Equivalent machine artifacts/profile produce identical rendered bytes regardless of:

- worker count/scheduling;
- input/database/hash-map iteration;
- host locale/timezone/path/temp directory;
- line-ending defaults;
- terminal width/color capability;
- current clock;
- model/provider availability.

## Render validation

- parse/validate machine artifact first;
- render every fragment/member;
- verify byte limits and exact token counts when requested;
- verify field-to-byte/source-map closure;
- recount final concatenated bytes/tokens;
- verify uncertainty and truncation markers;
- verify no private path/secret/source directive escape;
- compare committed golden bytes.

## Forbidden rendering behavior

- summarizing arbitrary source/comments with a model;
- hiding repeated conflicts/coverage because they are “noisy”;
- reordering by display width, filesystem order, database row, or completion time;
- dynamically selecting headings/wording from source text;
- embedding executable HTML/script/terminal control sequences;
- claiming token limits from character counts;
- rendering partial output as complete after serialization failure.
