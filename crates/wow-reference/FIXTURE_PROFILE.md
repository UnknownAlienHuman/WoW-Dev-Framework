# E0-B fixture profile

**Status:** normative closed fixture declaration. This is not a release Reference Pack.

## 1. Identity

```text
profile_id: fixture-retail-120100-e0-v1
profile_kind: fixture
release_eligible: false
flavor: retail
interface: 120100
client_build: 12.1.0.69497
source_repository: Gethe/wow-ui-source
source_revision: 027d26c3406d3de2cbd2b1f67d468fe033a1bcd4
source_verification_date: 2026-08-27
fixture_catalog_id: C_E0Fixture
fixture_schema_version: 1
```

The build/revision provide explicit context consistent with the current engineering KB baseline used when this fixture was authored. The fixture records themselves are synthetic, minimized, project-owned data.

The implementation must not:

- label this profile as a full or current Reference Pack;
- infer that every API/facet in build `12.1.0.69497` is represented;
- reuse its Secret facet as a real-game classification;
- silently upgrade it to release-grade status.

## 2. Purpose

The fixture proves four E0 seams:

1. exact known-symbol lookup;
2. exact absence with and without negative authority;
3. first-class restriction-facet retrieval;
4. conflict isolation independent of source-ingestion completeness.

It is deliberately small enough that every source record, lowered fact, coverage record, and expected query result can be reviewed manually.

## 3. Synthetic catalog

### System

```text
C_E0Fixture
```

### `KnownApi`

```text
qualified name: C_E0Fixture.KnownApi
kind: function
arguments:
  1. value: string, required, non-null
returns:
  1. accepted: boolean, non-null
restriction facets: none in E0
```

Use: exact positive lookup and ordinary-value control fixture.

### `SecretText`

```text
qualified name: C_E0Fixture.SecretText
kind: function
arguments: none
returns:
  1. text: string, non-null
restriction facets:
  secret.return on return position 1
  applicability: unconditional_fixture
```

Use: reference producer contract for one direct local Secret operation rule.

### `RemovedApi`

```text
query key only: C_E0Fixture.RemovedApi
stored entity: none
```

Use: exact miss. It is not assigned a replacement or alias.

## 4. Registration-source model

The fixture source conceptually represents an APIDocumentation registration table, but the contract bundle stores canonical data rather than executable Lua.

Logical registration order:

```text
0. system C_E0Fixture
1. function KnownApi
2. function SecretText
3. restriction metadata for SecretText return 1
```

A future Emmy/CST-backed evaluator must reproduce the same raw canonical records from an allow-listed fixture source without executing it.

## 5. Input inventory

Normative logical inputs in [`examples/fixture-bundle.json`](examples/fixture-bundle.json):

```text
fixture-profile
fixture-registration-source
fixture-variant-declarations
lookup-cases
```

Each input declares:

- stable input ID;
- normalized repository-relative logical path;
- content digest;
- byte length or canonical payload length;
- order index;
- license/provenance;
- affected coverage partitions.

Undeclared source material cannot enter the model.

## 6. Evaluator policy

E0 fixture policy ID:

```text
wow-reference-e0-declarative/1
```

Required limits:

```text
max input bytes: 65,536
max records: 64
max table depth: 16
max table entries: 1,024
max expression steps: 4,096
```

Allowed forms are listed in [`OPERATIONS.md`](OPERATIONS.md). The fixture does not require loops, function execution, metatables, module loading, IO, or environment access.

## 7. Canonical entity keys

```text
system:C_E0Fixture
function:C_E0Fixture.KnownApi
function:C_E0Fixture.SecretText
```

Canonical exact query key for absence:

```text
function:C_E0Fixture.RemovedApi
```

Keys are case-sensitive and kind-sensitive.

## 8. Coverage partitions

### Profile/inventory

```text
reference.fixture.profile
reference.fixture.inventory
```

### Symbol domain

```text
reference.fixture.apidoc.system:C_E0Fixture
```

### Restriction domain

```text
reference.fixture.restriction:C_E0Fixture.SecretText
```

The restriction partition is independent from broad symbol presence so a facet conflict does not erase the known function entity.

## 9. Capability IDs

```text
reference.fixture.profile.valid
reference.fixture.inputs.complete
reference.symbol.exact_lookup
reference.restriction.facets
reference.source_handle.resolve
```

## 10. Variants

### Complete

```text
variant_id: complete
symbol partition: Complete
SecretText restriction partition: Complete
conflicts: none
```

Expected:

- `KnownApi` -> found;
- `SecretText` -> found;
- `RemovedApi` -> authoritative absent;
- `SecretText` facets -> `secret.return` found.

### Partial

```text
variant_id: partial
symbol partition: Partial
restriction partition: Complete unless query case states otherwise
reason: declared unsupported/omitted record affecting exact negative authority
```

Expected:

- intact known entities may still return found with partial surrounding coverage;
- `RemovedApi` -> absent without authority;
- no replacement candidate;
- exact blockers returned.

### Conflict

```text
variant_id: conflict
input ingestion: Complete
symbol presence for SecretText: established
restriction partition source reading: Complete
restriction contract: conflict
```

The overlay introduces a second incompatible observation for the SecretText return facet. Expected:

- `SecretText` symbol -> found;
- `SecretText` facet lookup -> conflict;
- dependent Secret rule -> NotEvaluated;
- no first/last registration winner.

## 11. Unknown-field fixture

The closed bundle contains one unknown field path used to prove preservation and capability classification.

Rules:

- raw value round-trips exactly;
- field is associated with source/evidence;
- classification states whether it blocks a capability;
- unrelated facts remain usable;
- no field is silently dropped.

## 12. Source handles

All fixture source handles use a registered fixture/reference origin and normalized logical paths under:

```text
fixtures/e0/reference/
```

They never contain:

- local drive letters;
- user home directories;
- absolute paths;
- network shares;
- addon project paths;
- access tokens or temporary URLs.

## 13. Reference generation

The fixture `ReferenceGenerationId` is derived from canonical:

```text
profile declaration
input inventory
input content digests
evaluator policy version
variant identity
fixture schema version
```

It excludes wall-clock time, filesystem location, worker count, and discovery order.

Each variant may have a distinct generation/model digest. Consumers cannot mix them in one result.

## 14. Normative files

- [`examples/fixture-bundle.json`](examples/fixture-bundle.json)
- [`examples/lookup-cases.json`](examples/lookup-cases.json)
- [`examples/CHECKSUMS.json`](examples/CHECKSUMS.json)
- [`examples/README.md`](examples/README.md)

If any fixture semantics change, update:

1. this document;
2. affected decisions/data model/operations;
3. `CONTRACT.json`;
4. lookup cases;
5. checksums;
6. test expectations.

Do not regenerate expected outputs from a broken implementation and call the result a contract update.
