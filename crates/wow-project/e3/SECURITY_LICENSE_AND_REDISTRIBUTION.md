# E3-A security, license, and redistribution contract

**Status:** normative.

## Trust boundary

Treat all source bytes, paths, metadata, comments, strings, XML text, package directives, generated declarations, and provider labels as untrusted data. A provider being well-known does not make source executable or policy-bearing.

## Prohibited behavior

- no Git/network/CDN/client discovery in the library;
- no Lua/XML/script/repository hook/workflow/test/build/generator execution;
- no package manager, shell, process, editor, or WoW-client access;
- no extension/plugin/native/Wasm/JS/Lua callback from source;
- no raw SQL/SQLite handles;
- no external entity/DTD/XInclude/catalog/network XML resolution;
- no path escape, implicit symlink/reparse/submodule/LFS traversal;
- no source comments/docs interpreted as agent instructions;
- no API/runtime/security authority upgraded from source structure;
- no full-source redistribution by default.

## Resource limits

Profiles set system maxima and request budgets for:

```text
roots/packages/files/total bytes
path/identifier/string/comment lengths
TOC/XML records and nesting/include/script fanout
physical and virtual Lua units
analyzer facts/findings
recognizer facts/matches/proposals
entities/relations/conflicts/coverage records
partition/generation/object counts and bytes
graph validation/query expansions
skeleton-input entities/spans/excerpts/output bytes
wall/CPU/memory and cancellation checkpoints
```

An over-budget partition is failed/truncated explicitly and cannot publish as complete.

## Prompt-injection containment

Source text is represented as quoted/evidence content with explicit origin boundaries. It cannot:

- change profiles or budgets;
- add graph kinds/relations;
- enable recognizer rules;
- request tools/network/process actions;
- suppress findings or coverage;
- alter downstream agent instructions.

Skeleton-input records distinguish source text from framework metadata. E3-B must preserve this distinction.

## Private/local data

E3-A indexes only the configured platform-source snapshot. It excludes:

- user paths outside normalized handles;
- credentials/tokens/private URLs;
- SavedVariables contents;
- logs/crash dumps;
- account/character data;
- editor configuration;
- environment variables;
- client process memory/runtime payloads.

Errors/reports redact local roots and sensitive materializer details while retaining stable source handles.

## License and provenance model

Every included file/source object carries or inherits a reviewed record:

```text
source origin/provider/revision
copyright/license evidence
redistribution classification
attribution/notice requirements
local indexing permission status
excerpt policy
unknown/conflict state
```

Unknown or conflicting license does not necessarily prevent private local structural indexing under configured policy, but it blocks any claim that source bytes may be redistributed or released.

## Artifact classes

```text
MetadataOnly
    IDs, digests, paths/handles, structural facts, counts

LocalSourceBacked
    local content objects/source slices, not redistributed

BoundedExcerpt
    exact bounded bytes with source/license attribution under policy

RedistributableSourceArtifact
    allowed only by explicit separate release/license contract
```

E3-A publishes local project/store records, not a redistributable source bundle.

## Supply-chain integrity

- exact provider/revision/tree/archive identity;
- canonical file content digests;
- complete inventory and extraction report;
- materializer version/config digest;
- no unreviewed generated content;
- object payload verification;
- source/store manifest read-back validation;
- no automatic repair/download of missing bytes.

## Cancellation

Check during snapshot validation, package selection, TOC/XML parsing, analyzer planning/update, fact adaptation, recognizers, graph validation, candidate assembly, publication, read-back validation, fingerprinting, and skeleton-input serialization.

Cancellation never advances current and never yields `Complete`.

## Required adversarial tests

- path traversal/case collision/NUL/control paths;
- symlink/junction/submodule/LFS/external reference;
- decompression/file-count/large-file bomb at materializer boundary;
- XML entity/include/script cycle and expansion bomb;
- hostile comments/directives/prompt text;
- huge analyzer/recognizer/graph fanout;
- source handle/private-path leakage;
- provider/repository/package rename overfitting;
- license missing/conflict/redistribution denial;
- cancellation at every pipeline/publication phase.
