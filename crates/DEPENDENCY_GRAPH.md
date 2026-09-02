# Crate dependency graph

**Status:** normative boundary through documentation frontier E5-C.

Dependencies point toward narrower foundations. Maximum edges do not require activation.

| Crate | Maximum permitted direct dependencies |
|---|---|
| `wow-core` | none |
| `wow-store` | `wow-core` |
| `wow-reference` | `wow-core`, `wow-store` |
| `wow-annotations` | `wow-core`, `wow-reference` |
| `wow-emmy` | `wow-core` |
| `wow-graph` | `wow-core`, `wow-store` |
| `wow-recognizers` | `wow-core`, `wow-emmy`, `wow-graph` |
| `wow-project` | `wow-core`, `wow-store`, `wow-emmy`, `wow-graph`, `wow-recognizers` |
| `wow-rules` | `wow-core`, `wow-reference`, `wow-emmy`, `wow-project`, `wow-graph` |
| `wow-search` | `wow-core`, `wow-store`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-context` | `wow-core`, `wow-reference`, `wow-project`, `wow-graph` |
| `wow-cbm` | `wow-core` |
| `wow-service` | reviewed production crates through narrow public contracts |
| applications | `wow-service` only |

## Active E5-C slice

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-graph
        └── wow-recognizers
```

Other crates are inactive direct dependencies in E5-C. Their relevant evidence may be transitively referenced by exact E5-A/B artifacts but E5-C does not invoke them.

## Owner/effect flow

```text
E5-B PromotionSubmission catalog
-> wow-service independent revalidation
-> wow-recognizers core semantic/producer validation
-> wow-graph output and partition validation
-> wow-store immutable artifact/attestation/signature/catalog publication
-> signing authorization/signing/verification ports
-> canary authorization/assignment/observation ports
-> rollout/activation/rollback/revocation authorization ports
-> wow-project exact reindex
-> wow-graph new snapshot/partition closure
-> wow-store current/LKG/retention/audit
-> apps/wow transport
```

No owner imports service or applications. `wow-store` does not interpret recognizer/project/graph semantics. `wow-recognizers` does not publish catalogs/current pointers or authorize rollout. `wow-graph` does not run recognizers or resolve current. `wow-project` does not select packs. Service coordinates exact public receipts only.

## Distinct authorities

```text
E5-B review authorization
E5-C artifact validation
signing authorization
signature verification
publication authorization
canary authorization and typed evidence
rollout authorization
activation/current CAS
LKG designation
rollback/revocation/deactivation authorization
future E7 distribution authorization
runtime correctness
```

No edge collapses these states. GitHub/OS/CLI/file/commit identity is not any authorization.

## Historical immutability

Activation or rollback creates new project/recognizer/graph generations. Historical generations retain original partitions. Stale partition closure applies only to new targets and preserves foreign/core-independent/calibration partitions.

## Next E6-A boundary

```text
wow-cbm -> wow-core
```

E6-A external provider candidates remain optional `Candidate` evidence. `wow-cbm` cannot depend on search/context/service/apps or create reverse graph/project dependencies. E6-B later maps unverified locators through owner ports at service level.

## Forbidden patterns

- `wow-core` depending on framework crates.
- `wow-store` interpreting domain semantics.
- `wow-graph` parsing source, running recognizers, authorizing, or calling service/apps.
- `wow-recognizers` storing durable service/publication effects or activating packs.
- `wow-project` depending on service/apps.
- `wow-service` reimplementing owner algorithms or exposing raw storage/signing/vault/observation handles.
- Application importing any framework crate except `wow-service`.
- Private credentials, raw SQL, parser/session objects, filesystem roots, arbitrary executable callbacks, or provider databases crossing public seams.
- Production crate depending on application.

Changing an edge requires exact crossing data/operation, insufficiency of current seam, cycle/identity/security/privacy/license/evidence analysis, tests/mutations, migration notes, and manifest/workstream updates.