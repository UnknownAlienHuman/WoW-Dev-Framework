# Layout and rendering contract

**Status:** normative E1-C deterministic artifact topology, inert stub, identifier, documentation, file, and byte rendering contract.

Rendering converts a validated semantic model into static analysis source files. It cannot change semantic identities, infer missing facts, or execute source-provided content.

## 1. Rendering inputs

```text
validated AnnotationSemanticModel
LayoutRenderingProfile
consumer capability profile(s)
DocumentationSanitizationProfile
budgets/cancellation
```

Exact model/profile/reference generation and profile IDs must match the build request.

## 2. Versioned layout profile

A layout profile defines:

```text
profile ID/version/compatibility target
logical module -> file partition mapping
artifact-relative path rules
file header/footer/templates
module/import/meta markers
semantic declaration kind -> inert stub template
member/type/doc/restriction rendering rules
identifier and generated-name rules
ordering/spacing/blank-line/line-ending/encoding rules
consumer applicability
budgets
canonical digest
```

Ketho-compatible layout is one explicit profile. A future native/minimal profile may differ without changing semantic model.

## 3. Planned familiar tree

The Ketho-compatible profile may produce logical paths such as:

```text
Annotations/Core/Blizzard_APIDocumentationGenerated/
Annotations/Core/Data/Enum.lua
Annotations/Core/Data/Event.lua
Annotations/Core/Data/CVar.lua
Annotations/Core/Widget/
Annotations/Core/ScriptObject/
Annotations/Core/Type/
Annotations/Core/FrameXML/
Annotations/Core/WowDialect/
```

Exact active directories/files freeze from current source/consumer/parity probes. E1-C must not emit placeholder directories or claim complete FrameXML projection when the selected ReferenceView lacks that capability.

## 4. Artifact path safety

Generated paths derive only from:

```text
layout-owned fixed directory names
validated semantic module/system/namespace IDs through explicit safe-name mapping
fixed extension `.lua`
```

Rules:

- root-relative forward-slash paths;
- reject absolute/traversal/device/reserved/NUL/overlong/case-collision paths;
- source file names/paths/prose never concatenate directly into paths;
- distinct semantic identities cannot map to same path without deterministic partition/collision resolution;
- collision resolution is profile-defined and source-mapped/loss-recorded;
- no output writes inside crate; application validates configured root.

## 5. File headers

Renderer-owned fixed headers may include:

```text
---@meta
artifact/profile/reference generation comments
nonruntime warning
module-level annotations required by consumer profile
```

Canonical semantic identity excludes human volatile comments. No timestamp/host/temp/provider checkout path.

Header must not cause consumer to suppress diagnostics or mutate settings unless an explicit reviewed annotation tag is part of the consumer contract and its effects are tested.

## 6. Inert function/method stubs

Use fixed nonfunctional templates, for example conceptually:

```lua
---@param value SomeType
---@return ReturnType result
function C_System.Function(value) end
```

or consumer/profile-specific table assignment forms.

Rules:

- no source-provided function body;
- no calls/side effects/returns that imply runtime implementation;
- body shape fixed by renderer profile;
- namespace/table setup, if syntactically required, uses inert fixed declarations and cannot overwrite runtime when loaded because artifact is analysis-only; runtime loading remains explicitly unsupported;
- source docs cannot close function/comment/string or insert code;
- method receiver semantics explicit.

## 7. Classes/structures/fields

Render from semantic declarations using consumer-supported constructs such as:

```text
---@class
---@field
---@alias
```

Exact tag/syntax profile is frozen. Field/member order is semantic ordinal, then canonical ID. Invalid names use explicit safe forms or loss/unsupported.

Do not invent runtime constructors/metatables or implementation tables.

## 8. Enums/events/CVars

Each kind uses one frozen semantic/render strategy per profile:

- enum tag/class/literal alias/value table as consumer-tested;
- event payload alias/callback/table strategy;
- CVar metadata representation;
- source/deprecation/restriction docs/sidecar linkage.

No strategy is chosen merely because Ketho currently uses a similar file. Consumer semantics and exact reference input govern.

## 9. Named types and aliases

Rendered names come from semantic logical names and safe-name profile. Generated helper/anonymous names derive from domain-separated semantic IDs, not sequence/thread/path.

Alias cycles/forward references follow consumer profile and deterministic file/declaration ordering. If consumer cannot resolve required forward/recursive structure, split/order/generate helper declarations with explicit profile/loss, not ad hoc retries.

## 10. Documentation rendering

Source docs are untrusted. The profile defines:

```text
accepted documentation fields
max per fragment/declaration/file/artifact bytes
normalization of line endings/control characters
escaping or replacement for comment terminators/directive prefixes
whether raw URLs/markup/code fences are retained/simplified/omitted
line wrapping policy
truncation marker policy
```

Rules:

- source line beginning `---@` cannot create a directive;
- `--`, long-comment/string delimiters, control bytes, malformed UTF-8 handled safely;
- docs cannot create/delete blank declaration boundaries or file markers;
- docs omission/sanitization/truncation gets a loss record/source-map relation;
- human prose excluded from semantic declaration identity unless doc semantic manifest explicitly includes normalized content.

## 11. Identifier rendering

Classify source/logical names:

```text
valid simple Lua identifier
reserved keyword
qualified namespace/member
arbitrary string key
invalid/unrepresentable
```

Profile options:

- direct identifier/member form;
- safe bracket-string form where consumer supports declarations/types;
- deterministic generated alias with exact mapping/loss;
- Unsupported.

No locale case folding, character stripping, whitespace replacement, or collision-prone slugging by default.

## 12. String/literal rendering

Renderer owns canonical quoting/escape rules:

- deterministic short/long quoted form selected by profile;
- escape control characters, delimiters, invalid bytes according to contract;
- numeric/literal canonical text from type/number policy;
- no source literal copied verbatim if it can change syntax;
- round-trip tests from semantic value to parser consumer.

## 13. Declaration and file ordering

```text
layout partition/module priority
module semantic order
kind priority
owner/namespace/canonical name
signature/member ordinal
semantic declaration/member ID
```

Files ordered by profile partition then normalized path. No filesystem/store/query/thread order.

## 14. File partitioning

Criteria can include:

```text
system/namespace
entity kind
size budget
consumer forward-reference constraints
compatibility layout profile
```

Partitioning algorithm/version enters profile/artifact identity. Size-based split uses deterministic semantic boundaries and ID-derived suffixes, not worker completion.

## 15. Formatting

- UTF-8 without BOM unless a consumer profile proves another requirement;
- LF endings;
- one final newline;
- fixed indentation/spacing/blank-line rules;
- no external formatter dependency in correctness path unless pinned/probed and profile-bound;
- renderer output directly canonical and idempotent;
- trailing whitespace forbidden.

## 16. Generated fragments and spans

Renderer records fragments while writing final bytes:

```text
semantic element ID
renderer template/rule
file ID
byte start/end
line start/end
sanitized doc/name/string inputs
```

Spans use final UTF-8 byte and line contract. Validate nonoverlap/nesting rules and digest after file completion.

## 17. File validation

Before artifact manifest:

- path uniqueness/safety/case-collision;
- UTF-8/LF/final newline/trailing whitespace;
- syntax parse under frozen consumer-compatible parser;
- no disallowed executable statement/template outside fixed forms;
- declaration/file counts/bytes within budgets;
- semantic declaration/member coverage closure;
- fragment/source-map spans valid against bytes;
- file digest/length/line count exact;
- no source-provided directive/code/file injection.

## 18. Runtime-loading boundary

Artifact manifest/file headers/docs state:

```text
analysis-only generated declarations
not WoW addon runtime code
must not be packaged in addon TOC/release
```

The crate cannot enforce a consumer loading choice at runtime, but tests ensure stubs are inert and no source-provided behavior exists. Application packaging excludes artifact from addon payloads.

## 19. Budgets and cancellation

Bound:

```text
files/path length/declarations/members
rendered type/docs/literal bytes
line/file/artifact bytes
fragments/source-map entries
split count
sanitization/loss records
```

Cancel checkpoints between modules/files/declarations and bounded documentation/type rendering. No final file/artifact manifest on cancel; no background output.

## 20. Determinism

Equivalent semantic model/profile yields byte-identical files/manifests, independent of:

```text
worker count/order
input/store row order
hash maps
filesystem/temp root
consumer probe execution order
wall clock/host/provider path
```

Renderer test repeats 1/2/N workers and shuffles semantic input serialization order while preserving model IDs.

## 21. Required operations

```text
build_layout_rendering_profile
validate_layout_rendering_profile
partition_semantic_model_into_files
build_safe_artifact_relative_path
render_file_header_and_footer
render_declaration
render_member_and_type
sanitize_and_render_documentation
render_safe_identifier_and_literal
record_rendered_fragment_spans
canonicalize_rendered_files
validate_rendered_annotation_file
build_annotation_file_manifest
```

## 22. Required tests

- every declaration/file strategy;
- path/identifier/reserved/invalid/collision cases;
- docs/comment/directive/code/file injection corpus;
- string/numeric/literal escaping;
- function/method/class/alias/enum/event/CVar/widget/dialect stubs inert;
- forward/recursive type ordering;
- deterministic splitting/budget boundaries;
- syntax parse under each consumer profile;
- source-map spans/digests;
- 1/2/N byte equality;
- no timestamp/private/source path/trailing whitespace/BOM/CRLF;
- no full source implementation/runtime packaging.

## 23. Hard stops

- no direct source text interpolation into syntax/paths;
- no source-provided bodies/directives/code;
- no invalid/colliding path/name silent rewrite;
- no external formatter drift;
- no nondeterministic splitting/order;
- no runtime implementation semantics;
- no editor config/diagnostic mutation;
- no complete FrameXML/layout claim without reference capability;
- no artifact manifest before syntax/span/loss validation.
