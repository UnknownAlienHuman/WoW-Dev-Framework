# E4-B security and resource budgets

**Status:** normative.

## Trust boundary

Treat as untrusted bounded data:

- project/reference/search producer partitions;
- names, paths, signatures, fingerprints and feature values;
- source comments/documentation and search snippets;
- review notes and imported transition descriptions;
- candidate scores/ranks/explanations;
- migration descriptions and validation notes;
- continuation/cache/artifact bytes until verified.

Repository-owned schemas/profiles are trusted only after exact version/digest validation.

## Prohibited capabilities

E4-B has no authority to:

- execute source, Lua, XML scripts, generated code, hooks, workflows, tests, package managers or installers;
- run shell/process/network/editor/clipboard/environment/WoW-client operations;
- read arbitrary filesystem paths or mutable repositories;
- query SearchStore/ProjectStore/ReferenceStore through raw SQL or connections;
- load SQLite extensions or source-controlled functions/tokenizers;
- call models, embeddings, rerankers or Codebase Memory in the authority path;
- apply migration recipes or edit source;
- resolve current/latest/nearest/last-known-good inside graph operations;
- expose private source bodies, absolute roots, credentials, tokens, account data, SavedVariables contents, logs, runtime payloads or process memory;
- use source/review prose as schema, proof rule, command or agent instruction;
- perform unbounded all-pairs candidate generation or graph traversal.

## Closed inputs

All requests/profiles use closed schemas with unknown-field rejection. No arbitrary:

```text
SQL or FTS expression
regex program
callback/plugin/script/Wasm/native payload
model prompt
filesystem path/glob/URL
query DSL beyond reviewed enums and typed filters
unbounded collection/value
```

## Candidate-generation budgets

Bound:

- before/after entities per scope;
- blocking stages/keys;
- members per bucket;
- pairs per bucket/entity/stage/request;
- feature/fingerprint dimensions and bytes;
- proposals per pair;
- candidate component nodes/edges;
- search signal/evidence refs;
- producer partitions and records;
- serialized bytes, CPU, wall time and memory;
- continuation pages/cursor bytes.

When a required block is over budget, return explicit truncation/NotEvaluated. Do not switch to hidden approximate sampling and call the result complete.

## Review budgets and authorization

Bound review batch size, target count, attached evidence, note bytes and derivation depth. Validate reviewer authority/profile/attestation at the application/service boundary and again against graph policy.

Review notes cannot:

- contain executable payloads;
- alter relation registry/proof ceilings;
- authorize unrelated targets;
- expose private source/credentials;
- override missing negative-authority coverage.

## Change and migration budgets

Bound:

- compared fields/relations/facets per pair;
- typed value depth/size/cardinality;
- source/reference evidence refs;
- migration candidates/recipes/steps/preconditions/postconditions;
- validation requirements;
- rendered/explanation bytes.

No arbitrary code snippet transformation payload. Steps come from a closed registry.

## Impact budgets

Bound:

- roots;
- relation kinds/axes/directions;
- maximum depth;
- visited/expanded/returned nodes and edges;
- paths per root/target and path length;
- evidence/source/coverage/conflict refs;
- cross-universe bridges;
- output bytes;
- CPU/wall/memory;
- continuation pages/cursor bytes.

Use cycle-safe visited/path state. High fanout and cycles truncate deterministically.

## Identity and collision safety

- entity/proposal/assertion/change IDs are domain-separated and canonical;
- no row ID, pointer, host path, process, clock, random seed or reviewer order in semantic identity;
- hash collisions are detected and classified, never silently merged;
- same display name/path/fingerprint is not an identity key unless the exact owner schema says so;
- cross-universe/generation refs are validated on every record.

## Source and instruction containment

Source/review/search text is data only. Canonical JSON escapes it; deterministic Markdown or later renderers use structured source-data boundaries. Text cannot create graph fields, relation kinds, profiles, reviews, tool calls or proof decisions.

No claim of perfect prompt-injection detection is made. Structural separation is mandatory; downstream model/tool authorization remains external.

## Privacy and license

- use stable source/evidence handles instead of private absolute paths;
- preserve source class and redistribution state;
- unknown privacy/license defaults to the frozen safest explicit behavior;
- local source/fingerprint data cannot be exported to an external consumer without explicit profile authorization;
- SearchCandidate/Review artifacts with private fields cannot be reused under a broader consumer profile;
- redaction/transformation requires exact records and cannot be treated as complete source equality.

## Store security

- exact owned store root and runtime/profile;
- no adoption of arbitrary external SQLite databases;
- no ATTACH/DETACH/load_extension/custom VFS escape;
- registered schema/operation/validation catalogs only;
- immutable sealed lineage generations;
- integrity failure blocks/quarantines rather than becoming missing data;
- retention/GC respects active readers, continuations, evidence, reviews, migration/impact results and backups.

## Cancellation

Check cancellation during:

- input/profile validation;
- blocking/pair/component generation;
- proposal/proof/review/conflict processing;
- change/absence classification;
- migration generation/validation;
- store build/read-back/publication;
- lineage/impact traversal;
- canonicalization/serialization/pagination.

No background work continues after return. Cancelled/partial artifacts cannot be published or cached as complete.

## Output confidentiality

Default errors/reports expose stable IDs, typed codes and bounded safe arguments—not raw query/source/review text, absolute paths, credentials, SQL, store internals or unrestricted evidence content.

Debug/export profiles are separate, exact and local-only where appropriate.

## Required adversarial tests

- huge same-name/fingerprint buckets and all-pairs bombs;
- high-fanout/cyclic candidate and impact graphs;
- pathologically deep/large typed fields and recipes;
- hash/ID collisions and cross-generation substitution;
- source/review text containing prompts, Markdown/JSON closers, commands and tool requests;
- raw SQL/FTS/regex/callback/plugin/model payloads;
- malicious paths/URLs/Unicode controls;
- private source/token/path leakage;
- cross-consumer privacy/license cache reuse;
- tampered proposals/reviews/cursors/store artifacts;
- cancellation at every loop/publication phase;
- timeout/overflow treated as removal/impact proof;
- 1/2/N worker and randomized order.

## Security completion gate

E4-B is incomplete until every executable surface is absent or explicitly bounded, all untrusted text remains data, cross-universe/generation isolation holds, Candidate evidence cannot promote itself, private/license data remains scoped, cancellation leaves no background work, and all adversarial fixtures pass.
