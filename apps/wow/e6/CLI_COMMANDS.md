# E6-B CLI command grammar

**Status:** normative.

## Common options

```text
--config <PATH>
--request <PATH|->
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--operation-id <ID>
--expect-digest <SHA256>
--consumer-profile <ID>
--privacy-profile <ID>
--license-profile <ID>
--budget-profile <ID>
```

## Exact selectors

```text
--provider-configuration <ID>
--provider-descriptor <ID>
--external-state <ID>
--query <ID|PATH|->
--query-operation <ID>
--result <ID>
--candidate <ID>
--artifact <ID>
--continuation <ID>
--mapping <ID>
--selection <ID>
--project-publication <ID>
--reference-generation <ID>
--graph-generation <ID>
--expect-current-digest <SHA256>
--context-profile <ID>
--context-continuation <ID>
```

A permitted outer current selector, where the service contract explicitly supports it, is passed as typed request data with an expected-current guard. The CLI never resolves it.

## Command mapping

```text
status -> external_candidate_status
provider validate -> external_candidate_provider_validate
query submit -> external_candidate_query_submit
query get -> external_candidate_query_get
query list -> external_candidate_query_list
query cancel -> external_candidate_query_cancel
query continue -> external_candidate_query_continue
operation reconcile -> external_candidate_operation_reconcile
result validate -> external_candidate_result_validate
result explain -> external_candidate_result_explain
result compare -> external_candidate_result_compare
artifact build -> external_candidate_artifact_build
artifact get -> external_candidate_artifact_get
mapping resolve -> external_candidate_mapping_resolve
mapping get -> external_candidate_mapping_get
selection record -> external_candidate_selection_record
selection get -> external_candidate_selection_get
context build -> external_candidate_context_build
context continue -> external_candidate_context_continue
cache validate -> external_candidate_cache_validate
```

## Forbidden options and inputs

```text
--latest
--best
--first
--sole
--nearest
--default-provider
--auto-select
--force-map
--retry-unknown
--tool
--mcp-method
--sql
--command
--provider-database
--credential
--token
--password
--private-endpoint
--open-provider-path
```

No include, interpolation, environment expansion, shell substitution, plugin, script, archive extraction, arbitrary RPC object, or cwd/home/Git/editor/WoW discovery is supported. At most one stdin consumer is allowed.