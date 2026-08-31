# Logical schema and registered operations

**Status:** normative store/domain seam.

## Bundle ownership

```text
wow-project
    project source/TOC/XML/load/analyzer/recognizer/candidate records

wow-graph
    graph registry/assertion/conflict/coverage/snapshot records

wow-store
    generation, schema, operation, validation, object, lease, registry,
    recovery, retention, and physical integrity records
```

Store does not invent domain tables or merge policies.

## Schema bundle

```text
RegisteredSchemaBundle
    owner crate and contract ID
    schema ID/version
    logical record families
    physical DDL implementation ID
    required indexes
    foreign/reference closure declarations
    compatibility/migration constraints
    validation catalog dependencies
    canonical digest
```

DDL is implementation-owned reviewed code selected by ID. It is not caller SQL text.

## Operation catalog

```text
RegisteredOperationDefinition
    operation ID/version
    owner
    accepted payload schema IDs
    target logical partitions
    phase constraints
    idempotence/repetition policy
    expected effects/report schema
    budgets
```

E2-D phases:

```text
store-bootstrap
project-records
graph-registry
graph-assertions
conflicts-and-coverage
domain-manifests
object-references
generation-manifests
validation-support
```

Order is explicit and acyclic.

## Required store-owned operations

```text
create_generation_metadata
record_registered_bundle_set
record_operation_manifest
record_object_reference_manifest
record_validation_report
seal_generation_manifest
```

## Project-owned logical operation families

```text
replace_project_source_inventory
replace_toc_xml_load_partitions
replace_lua_unit_and_analyzer_bindings
replace_recognizer_result_partitions
record_project_index_candidate
record_project_snapshot_manifest
```

## Graph-owned logical operation families

```text
insert_or_validate_graph_registry
replace_graph_producer_partitions
replace_graph_conflicts_and_coverage
record_graph_snapshot_manifest
```

Exact operation names/versions freeze in the respective contracts. Store sees typed invocation IDs and expected manifests.

## Read catalog

Registered reads include exact bounded operations for:

```text
generation and bundle manifests
project source/file/package/TOC/XML/load records
analyzer and recognizer bindings
graph entities/relations/assertions/conflicts/coverage
project and graph snapshot manifests
validation and object-reference manifests
```

No generic SQL query API. Complex bounded graph traversal remains graph-owned and composes registered reads or reviewed prepared query implementations.

## Validation catalog

Store-level:

```text
SQLite quick/integrity check according to profile
foreign-key and schema/application/user version
registered bundle/operation manifest closure
row/count/digest manifest reconciliation
object-reference closure
forbidden mutable/staging member absence
```

Domain-level catalogs are invoked through registered validation IDs and return structured records. Store does not reinterpret a domain failure as success.

## Compatibility

A generation pins every schema/operation/read/validation bundle version. Opening with an incompatible runtime is typed unavailable. In-place migration is forbidden; migration/rebuild materializes a new generation.

## Forbidden seam

```text
execute_sql(String)
with_connection(callback)
with_transaction(callback)
table_name_from_caller
pragma_from_caller
attach_database_from_path
load_extension
user_defined_function_from_input
migration_script_from_repository
```
