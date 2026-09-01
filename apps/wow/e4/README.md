# `apps/wow` E4-C search, lineage, migration, and impact CLI contract

**Status:** implementation-ready documentation; no Rust code yet.

**Contract ID:** `apps/wow/e4-c/search-lineage-impact-cli`

`apps/wow` is a thin CLI adapter over `wow-service`. Its only framework dependency is `wow-service`.

## Command families

```text
wow search index status
wow search index build
wow search index validate
wow search query
wow search continue
wow search explain
wow search select
wow search context

wow lineage status
wow lineage build
wow lineage validate
wow lineage review validate
wow lineage review apply
wow lineage compare
wow lineage trace
wow lineage explain

wow migration candidates
wow migration validate

wow impact plan
wow impact run
wow impact continue
wow impact explain
```

No command applies migration steps, edits source, runs the game, invokes models, resolves a candidate automatically, or bypasses `wow-service`.

## Application responsibilities

- parse strict bounded command/config/artifact input;
- construct exactly one typed E4-C service request;
- pass symbolic current selectors unchanged to service;
- pass exact shard/snapshot/result/candidate/review/continuation IDs mechanically;
- project OS signals into typed cancellation;
- invoke `wow-service` exactly once per valid command;
- emit canonical envelope JSON, faithful text, or one exact validated artifact;
- map service outcome/validation state to frozen exit codes;
- enforce stdout/stderr/broken-pipe/output-file behavior.

## Forbidden responsibilities

- current or catalog resolution;
- search shard/query/ranking/miss algorithms;
- search candidate auto-selection;
- lineage proposal/proof/review/migration/impact algorithms;
- reviewer authorization from local/GitHub identity;
- project/reference/graph/search/context/store direct calls;
- source/repository/editor/client discovery;
- source edits, migration application or tool authorization;
- raw SQL/FTS/regex/script/plugin/model execution;
- implicit network/config/environment discovery;
- retry on a different generation;
- background work or double output.

## Selectors

The CLI accepts exact IDs and a limited symbolic `current` token only where the service contract allows it. It never resolves or validates current locally.

Candidate selection requires both exact result and candidate identifiers. Rank number, display name, query text, or “first result” is not accepted as a selection selector.

## Review input

Review commands accept one explicit bounded strict JSON `LineageReviewDecisionEnvelope` file or stdin. The app transports bytes only. It does not infer reviewer identity, validate signatures, choose proof class, rewrite notes or accept plain prose.

## Output modes

```text
envelope-json
text
artifact
```

`envelope-json` is the automation default. `artifact` is allowed only when the service returns exactly one eligible validated artifact for that command/profile.

## Exit codes

```text
0    complete/no-change or validation Valid
1    completed validation Invalid or review decision Invalid/Rejected where command profile declares
2    partial/candidate-only/conflict-blocked/truncated/not-evaluated
3    structured selector/generation/shard/snapshot/candidate/review/retention/capability failure
4    internal owner/service/closure/serialization/output failure
64   CLI/config/input/output transport failure before service invocation
130  cancelled
```

Exact command mapping is in [`OUTPUT_EXIT_AND_STREAMS.md`](OUTPUT_EXIT_AND_STREAMS.md).

## Current state

```text
documentation frontier: E4-C
implementation frontier: not-started
Cargo.toml: absent
Rust source: absent
CI/workflows: absent
```
