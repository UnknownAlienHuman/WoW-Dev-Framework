# E4-A security, privacy, and resource budgets

**Status:** normative.

## Trust boundary

Treat as untrusted:

- literal query text;
- owner identifiers, aliases, labels, comments, and documentation;
- source spans and snippets;
- repository/provider/package/path names;
- cached or persisted SearchShard bytes until validated;
- continuation/result-set objects;
- external application audit text;
- physical SQLite files.

Reviewed schemas/profiles are trusted only after exact version and digest validation.

## Prohibited capabilities

`wow-search` cannot:

- execute Lua, XML scripts, repository hooks, workflows, generators, plugins, callbacks, expressions, SQL, shell, Wasm, JS, or native extensions;
- load SQLite extensions;
- accept raw FTS `MATCH`, SQL, regex, tokenizer, rank function, table, column, PRAGMA, or path input;
- access arbitrary filesystem, network, process, editor, clipboard, environment, credentials, or WoW client;
- read SavedVariables contents, logs, runtime payloads, process memory, or secure execution state;
- call LLMs, embeddings, rerankers, or Codebase Memory;
- mutate project/reference/graph/source truth;
- emit automatic edits or remediation;
- resolve a floating current/latest generation.

## Query input limits

Profiles bound:

```text
request bytes and nesting
literal text bytes/code points
exact identifiers/aliases/members/prefixes
terms/phrases/prefix terms
structured filters/features
roots and graph seeds
enabled/required lanes
result/detail/snippet fields
continuation bytes
```

Reject negative, overflowing, zero-where-required, or unlimited values.

## Index-build limits

Bound:

```text
owner partitions and records
documents per partition/shard
fields/values/origins per document
text bytes and tokens per field/document/shard
aliases/prefixes/trigrams/shape features
FTS rows/terms/prefix expansion
object/database/page size
validation and golden queries
memory/CPU/time/cancellation
```

A truncated build cannot publish as complete.

## Lane limits

### Exact/alias/member/prefix

- input lengths;
- ambiguous exact-match count;
- aliases per entity;
- prefix minimum/expansion/candidates;
- origin/evidence bytes.

### Text

- AST depth/nodes;
- terms/phrases/prefixes;
- FTS result rows;
- per-field matches;
- snippets/highlights;
- query time/memory.

### Similarity

- candidate prefilter size;
- identifier/trigram count;
- edit-distance matrix/operations;
- distance threshold.

### Shape

- feature count;
- type recursion/depth;
- candidate count;
- relation/detail lookups.

### Graph

- seeds;
- relation kinds;
- depth/fanout;
- visited nodes/edges;
- paths per target;
- total path bytes.

## Result and explanation limits

Bound:

- candidate enumeration cap;
- result-set manifest size;
- page size;
- explanation signals/paths/origins;
- coverage/conflict/omission records;
- snippets/detail handles;
- serialized bytes;
- cumulative continuation budget.

Mandatory candidate identity, rank, authority, coverage/conflict, omissions, and continuation cannot be pruned. If the minimum does not fit, fail.

## Safe query compilation

- parse only closed JSON/domain types;
- validate field/relation/profile IDs against reviewed registries;
- encode literal terms as data;
- use prepared operations;
- never concatenate user text into SQL/FTS syntax;
- no caller-specified column/table names;
- reject invalid Unicode/NUL/control characters according to profile;
- preserve original/normalized digests without echoing private text in errors.

## FTS denial-of-service defenses

- bound prefix queries and term expansion;
- forbid leading/unbounded wildcards;
- bound phrase length and AST complexity;
- no caller NEAR distance or open NOT scans in v1;
- use SQLite limits and progress/cancellation hooks from the frozen profile;
- cap result rows before snippet generation;
- no optimize/rebuild commands from query input.

## Source and snippet safety

- index only approved bounded fields;
- source text cannot register aliases/profiles/operators;
- snippets are escaped typed data, not raw HTML/Markdown;
- no full source body by default;
- exact source detail uses owner handles and separate privacy/license checks;
- local/private text cannot enter an external-consumer result/profile;
- no absolute private paths, credentials, tokens, or disallowed source in errors/logs/explanations.

## Physical shard safety

- open only owned validated SearchShard artifacts;
- no adoption of arbitrary SQLite files;
- read queries use read-only/query-only mode;
- no ATTACH/DETACH;
- no extension loading;
- verify schema/profile/digests/integrity before query;
- corruption is failure/quarantine, not empty results;
- physical cache/store paths never come from source/query strings.

## Cancellation

Check during:

- owner enumeration/document projection;
- normalization/query planning;
- every lane loop;
- FTS execution and snippet generation;
- similarity/shape calculations;
- graph traversal;
- fusion/explanation;
- result materialization/pagination;
- validation/serialization.

No background work after return. Cancelled output cannot be complete or enter a complete cache/result manifest.

## Output privacy

Default logs/errors contain stable IDs, counts, codes, and bounded redacted query metadata. They do not contain full query text, source docs, private paths, credentials, or snippets.

Evaluation corpora with source text require an explicit local profile and license/privacy decision.

## Adversarial corpus

Include:

- FTS/SQL/regex-like query strings;
- quotes, operators, wildcard and column injection;
- invalid/ambiguous Unicode and confusables;
- extremely long/repeated terms;
- prefix explosion;
- high-frequency document terms;
- huge alias and field lists;
- high-fanout/cyclic graph;
- malicious comments/documentation;
- private-path/token-like fields;
- corrupt/tampered SQLite, manifest, cursor, result set;
- cross-generation/universe substitution;
- cancellation at each stage.

## Security completion gate

E4-A is incomplete until all executable surfaces remain absent, query compilation is structural, owner/shard privacy is enforced, resource/cancellation tests pass, physical shard validation rejects hostile artifacts, and no source/query content can alter authority, profiles, ranking, tools, or agent policy.
