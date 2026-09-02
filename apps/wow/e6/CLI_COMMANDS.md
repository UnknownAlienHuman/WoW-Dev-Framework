# E6-B CLI command and option contracts

**Status:** normative grammar.

## Common options

```text
--config <PATH>
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--consumer-profile <ID>
--privacy-profile <ID>
--budget-profile <ID>
--operation-id <ID>
--request-id <ID>
```

Precedence:

```text
explicit CLI fields
-> explicit strict JSON config
-> service-owned named defaults
```

No implicit environment/cwd/home/Git/editor/WoW configuration.

## Exact identity options

```text
--provider <ProviderDescriptorId>
--adapter-profile <ID>
--external-state <ExternalStateBindingId>
--generation <ExternalGenerationId>
--observation <ExternalObservationReceiptId>
--result <ExternalCandidateResultSetId>
--candidate <ExternalCandidateId>
--artifact <ExternalCandidateArtifactId>
--locator <UnverifiedProviderLocatorId>
--mapping <ExternalCandidateMappingReceiptId>
--selection <ExternalCandidateSelectionReceiptId>
--project-publication <ProjectPublicationId>
--reference-generation <ReferenceGenerationId>
--mapped-root <ExactOwnerRootId>
--context-universe <ContextUniverseSetId>
--expect-digest <SHA256>
```

The application does not resolve these identifiers.

## Explicit JSON inputs

```text
--request <PATH|->
--query <PATH|->
--mapping-request <PATH|->
--selection-request <PATH|->
--context-request <PATH|->
--continuation <PATH|->
--cache-entry <PATH|->
--input <PATH|->
```

Each command accepts only declared inputs and at most one stdin consumer.

## Status and validation

```text
wow external status
wow external provider validate --provider <ID> [--expect-digest <SHA256>]
wow external generation validate --external-state <ID> [--expect-digest <SHA256>]
```

Validation is read-only and never repairs/reclassifies provider state.

## Query

```text
wow external query
    --provider <ID>
    --query <PATH|->
    --operation-id <ID>
```

The query contains a closed E6-A request and explicit state/session profiles. There is no endpoint, token, tool-name, raw MCP, SQL, script, model, or provider management flag.

## Continue

```text
wow external continue
    --result <ID>
    --continuation <PATH|->
    --operation-id <ID>
```

Continuation cannot replace provider, state, query, profile, privacy scope, or cumulative budget.

## Result get/list/validate

```text
wow external result get --result <ID> [--expect-digest <SHA256>]

wow external result list
    --catalog-snapshot <ID>
    [--continuation <PATH|->]

wow external result validate --result <ID> [--expect-digest <SHA256>]
```

The app does not sort/select newest, best, highest-score, or sole results.

## Explain and artifact build

```text
wow external explain
    --result <ID>
    --candidate <ID>

wow external artifact build
    --result <ID>
    --operation-id <ID>
    --artifact-profile <ID>
```

Explanation preserves Candidate authority. Artifact build does not map/select/contextualize automatically.

## Mapping

```text
wow external mapping validate
    --mapping <ID>
    [--expect-digest <SHA256>]

wow external map
    --mapping-request <PATH|->
    --operation-id <ID>
```

Mapping request names exact result/candidate/locator, Project or Reference owner, exact publication/generation/profile, and budgets. No path, URL, fuzzy, FTS, embedding, name, or snippet mapping options exist.

## Selection

```text
wow external selection validate
    --selection <ID>
    [--expect-digest <SHA256>]

wow external select
    --selection-request <PATH|->
    --operation-id <ID>
```

Selection request names exact result/candidate/mapping/root and explicit origin. Forbidden flags include:

```text
--top
--first
--last
--best
--highest-score
--sole
--auto-select
--name
--path
--snippet
```

## Context

```text
wow external context
    --context-request <PATH|->
    --operation-id <ID>
```

Request names exact selection receipt, mapped owner root, context universe/profile/budgets, and external-attachment disclosure profile. The app does not invoke context directly or include provider prose/rank/score as framework facts.

## Operation get/reconcile

```text
wow external operation get
    --operation-id <ID>
    --original-request-digest <SHA256>

wow external operation reconcile
    --operation-id <ID>
    --original-request-digest <SHA256>
```

Reconcile preserves original provider/state/query/owner/profile/budget identities. It is not a rerun command and accepts no replacement fields.

## Cache validation

```text
wow external cache validate
    --cache-entry <PATH|->
```

Validation cannot refresh provider state or call the provider implicitly.

## Nonexistent command surface

No command exists for:

```text
provider install/start/stop/update/configure/index/import/delete
provider database open/write
raw MCP/tool call
external auto-select
external map-by-name/path/snippet
external graph/lineage publish
external edit/apply/tool-run
external privacy delete (deferred contract)
```

## One-call rule

After successful transport parsing, every command invokes exactly one public E6-B service operation. The application does not compose authorization, session, E6-A, mapping, selection, context, store, retention, or audit operations itself.
