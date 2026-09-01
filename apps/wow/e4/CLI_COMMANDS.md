# E4-C CLI commands and option contracts

**Status:** normative command grammar. Concrete parser implementation may differ only with same-change fixture/contract updates.

## Common options

```text
--config <PATH>
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--consumer-profile <ID>
--budget-profile <ID>
--request-id <ID>
```

Only command-declared options are accepted. Config precedence:

```text
explicit CLI values
-> explicit strict JSON config
-> service-owned named defaults
```

The app does not read environment variables, cwd/home config, editor settings or Git metadata.

## Publication selectors

```text
--project current
--project store-generation:<ID>
--project publication-set:<ID>

--before-project store-generation:<ID>|publication-set:<ID>|current
--after-project store-generation:<ID>|publication-set:<ID>|current

--platform omitted|current|store-generation:<ID>|publication-set:<ID>
--before-platform omitted|store-generation:<ID>|publication-set:<ID>|current
--after-platform omitted|store-generation:<ID>|publication-set:<ID>|current

--reference reference-view:<ID>|reference-generation:<ID>|current-compatible
```

The app transports these tokens; service resolves symbolic aliases.

## Search shard selectors

```text
--search-shard search-shard:<ID>
--search-shard for-owner-generation:<SearchProfileSetId>
```

`for-owner-generation` is a deterministic service catalog lookup after owner generation acquisition. It is not latest.

## Lineage selectors

```text
--lineage-snapshot lineage-snapshot:<ID>
--lineage-snapshot for-comparison:<LineageProfileSetId>
```

Zero or multiple catalog matches is a service error; app does not choose.

## `wow search index status`

Constructs `search_index_status` with exact/current owner selectors and optional profile filter. No shard build.

## `wow search index build`

Requires:

```text
owner selector(s)
--search-profile <ID>
--operation-id <ID>
```

Optional exact expected-current guard/configured output/retention profile. No implicit follow-up query.

## `wow search index validate`

Requires exact `--search-shard search-shard:<ID>`. Reads one explicit artifact only through service owner ports; no local database open.

## `wow search query`

Accepts one strict query source:

```text
--query-json <PATH|->
```

or a bounded closed CLI subset:

```text
--text <UTF-8 literal>
--exact-id <TYPE@BASE64URL>
--exact-name <UTF-8 literal>
--kind <ID> (repeatable within limit)
--universe <ID> (repeatable within limit)
--lane <ID> (repeatable within limit)
--require-lane <ID>
--search-profile <ID>
```

The app does not construct raw SQL/FTS syntax or infer query intent beyond the frozen mechanical option-to-field map.

## `wow search continue`

Requires exactly:

```text
--continuation <PATH|->
```

No owner selector, query, lane, profile or budget override.

## `wow search explain`

Requires exact:

```text
--search-result <ID>
--candidate <ID>
```

and either exact retained shard selectors encoded in the service result guard or an explicit result artifact input. Rank alone is invalid.

## `wow search select`

Requires:

```text
--search-result <ID>
--result-set <ID>
--candidate <ID>
--selection-origin <ID>
```

Optional exact guard artifact. No `--first`, `--top`, `--best`, `--yes`, display-name or rank selector.

## `wow search context`

Requires all `search select` fields plus one exact E3-C context request/profile source:

```text
--context-request <PATH|->
```

The candidate's exact entity becomes the context root after service validation. Query text is never reused as a root.

## `wow lineage status`

Reports exact comparison/profile/catalog state. Selectors may be exact/current under service rules.

## `wow lineage build`

Requires:

```text
one exact comparison class
before and after selectors
--lineage-profile <ID>
--operation-id <ID>
```

Optional:

```text
--search-candidates <enabled|disabled>
--review-input <PATH|->
```

The app cannot accept/reject a candidate itself.

## `wow lineage validate`

Requires exact `--lineage-snapshot lineage-snapshot:<ID>`.

## `wow lineage review validate`

Requires exact base/comparison guards and `--review-input <PATH|->`. No mutation occurs.

## `wow lineage review apply`

Requires:

```text
--lineage-snapshot lineage-snapshot:<ID>
--review-input <PATH|->
--operation-id <ID>
```

Service revalidates authorization and graph semantics and creates a new snapshot. App cannot force acceptance.

## `wow lineage compare`

Requires exact before/after entity IDs or one exact accepted lineage assertion ID. Display names are insufficient.

## `wow lineage trace`

Requires exact root entity/assertion and exact lineage snapshot. Candidate/Possible inclusion flags are explicit and profile-bound.

## `wow lineage explain`

Requires exact proposal/assertion/component/review/change ID under one exact snapshot.

## `wow migration candidates`

Requires exact lineage snapshot and source entity/change root. Search enrichment is service/profile-controlled and remains Candidate.

## `wow migration validate`

Requires one explicit bounded candidate/recipe artifact:

```text
--input <PATH|->
--lineage-snapshot lineage-snapshot:<ID>
```

No apply/edit command exists in E4-C.

## `wow impact plan`

Requires exact lineage snapshot, exact change/assertion roots and exact target graph selector/profile.

## `wow impact run`

Accepts either a strict plan artifact or the exact plan fields. It cannot infer roots from a query string.

## `wow impact continue`

Requires exactly one continuation artifact. No generation/profile/budget override.

## `wow impact explain`

Requires exact impact result/path ID and governing snapshot/plan guards.

## Input tokens

Entity/root tokens reuse the mechanical exact grammar:

```text
<RootKind>@<base64url-no-pad(canonical UTF-8 ID bytes)>
```

Search candidate token, when compact form is used:

```text
<SearchResultId>@<SearchCandidateId>
```

The parser treats both components as IDs and does not split names/paths heuristically.

## One-call rule

After successful CLI/config/input parsing, exactly one `wow-service` operation is invoked. Validation, explain and continue commands are not composed from multiple public service calls inside the app.
