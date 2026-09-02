# Crate dependency graph

**Status:** normative final planned boundary through E7-B.

Dependencies point toward narrower foundations. Maximum permitted edges do not require activation; every implementation package uses the smallest exact active slice.

## Maximum direct framework dependencies

| Component | Maximum permitted direct framework dependencies |
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
| `wow-service` | reviewed production owner crates through narrow public contracts |
| `apps/wow` | `wow-service` only |
| `apps/wow-reference-builder` | `wow-service` only |
| `tools/wow-release` | `wow-service` only |

Build, development, and test dependencies are classified separately and cannot introduce a production reverse edge or let an application or tool reproduce owner semantics.

## Foundation direction

```text
wow-core
└── no framework dependency

wow-store
└── wow-core
```

`wow-core` owns generic semantic primitives only. `wow-store` owns generic physical persistence, effects, leases, retention, backup, restore, and GC only. Neither absorbs project, graph, provider, session, editor, release, or installation semantics to avoid a dependency issue.

## Exact local intelligence

```text
wow-reference
    -> wow-core
    -> wow-store when persistent

wow-annotations
    -> wow-core
    -> wow-reference

wow-emmy
    -> wow-core

wow-graph
    -> wow-core
    -> wow-store

wow-recognizers
    -> wow-core
    -> wow-emmy
    -> wow-graph

wow-project
    -> wow-core
    -> wow-store
    -> wow-emmy
    -> wow-graph
    -> wow-recognizers

wow-rules
    -> wow-core
    -> wow-reference
    -> wow-emmy
    -> wow-project
    -> wow-graph

wow-search
    -> wow-core
    -> wow-store
    -> wow-reference
    -> wow-project
    -> wow-graph

wow-context
    -> wow-core
    -> wow-reference
    -> wow-project
    -> wow-graph
```

Active implementation slices may omit maximum dependencies. No owner above imports service, applications, or tools.

## Optional external Candidate owner

```text
wow-cbm
└── wow-core
```

`wow-cbm` receives an already-acquired typed transport. It does not depend on provider SDK or MCP implementation, store, project, reference, graph, search, context, service, applications, or release tooling.

## Service slices

### E0-F

```text
apps/wow -> wow-service
wow-service -> wow-core + fixture reference, Emmy, project, and rule owners
```

### E1-D

```text
apps/wow-reference-builder -> wow-service
wow-service -> wow-core + wow-store + wow-reference + wow-annotations
```

### E3 and E4

```text
apps/wow -> wow-service
wow-service -> exact project, reference, graph, context, search, and rule owner ports
```

### E5-C

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-graph
        └── wow-recognizers

external authorization, signing, and canary adapters
    -> narrow service ports, not framework semantic dependencies
```

### E6-B

```text
apps/wow
    -> wow-service
        ├── wow-core
        ├── wow-store
        ├── wow-project
        ├── wow-reference
        ├── wow-graph
        ├── wow-context
        └── wow-cbm

host provider, session, and authorization adapters
    -> narrow service and wow-cbm ports
```

### E7-A

```text
apps/wow
    -> wow-service
        -> only implemented owner crates required by the exact registry entry

local daemon, LSP, and MCP
    are transport modes inside apps/wow
    and never import lower owner crates

wow-project
    owns workspace and document overlay identity, bytes, versions, and coordinates

wow-store
    owns generic registry, session, operation ticket, response journal,
    lease, retention, and recovery persistence
```

Canonical E7-A IDs:

```text
wow-service/e7-a/frontend-session-operation-registry
apps/wow/e7-a/frontend-transports
```

### E7-B

```text
apps/wow/e7b        -> wow-service
tools/wow-release   -> wow-service

wow-service release slice
    -> wow-core
    -> wow-store
    -> narrow external owner ports:
         source and dependency materialization
         build authorization and typed executor
         SBOM, provenance, license, and notices
         portable and platform signing and verification
         distribution publisher and reader
         installation, replacement helper, migration, and rollback
         support, revocation, retirement, and incidents
```

Release adapters are not framework semantic crates. They implement exact typed service ports and expose no generic process, provider API, filesystem, installer, or credential handles.

## Owner-neutral seam projections

A projection may cross without a reverse dependency only when an exact contract defines its data, authority, and proof ceiling.

### E6 locator mapping

```text
wow-cbm ExternalCandidate
-> service-owned bounded locator projection
-> wow-project OR wow-reference exact mapping owner
-> immutable mapping record in service/store
```

Project and reference owners do not depend on `wow-cbm` or service.

### E7-A document overlays

```text
LSP, MCP, or daemon document message
-> apps/wow transport data
-> wow-service session and overlay operation
-> wow-project exact document-overlay owner
-> wow-emmy and owner analysis against the exact overlay
-> immutable owner results
```

The app and service do not patch source or convert owner coordinates independently.

### E7-A session persistence

```text
service and owner canonical session, request, response, and stream records
-> wow-store generic prepared object, effect, lease, and retention ports
```

Store does not interpret frontend protocol, workspace, document, diagnostic, or editor semantics.

### E7-B release storage

```text
service and release-owner canonical source, build, evidence, bundle,
channel, installation, update, rollback, and support records
-> wow-store generic object, catalog, CAS, effect, backup, retention,
and reconciliation ports
```

Store does not decide reproducibility, signature trust, support, installation eligibility, update success, or rollback safety.

## Authority boundaries

```text
owner facts and exact owner negatives      exact only under owner coverage
search and provider results                Candidate unless exact owner evidence exists
external locator mapping                   locator-to-owner-record identity only
selection receipt                          caller intent only
context artifact                           exact owner evidence only
session and transport state                coordination and delivery only
review, holdout, and signatures            independent scoped gates
release build, channel, and distribution   supply and publication states only
installation and self-check                exact installed-state evidence only
runtime correctness                        separate exact runtime evidence
```

No dependency edge collapses these states.

## Effect direction

```text
OperationId + CanonicalRequestDigest
-> service durable registration
-> exact owner effect through a narrow port
-> owner and store receipt plus reconciliation
-> fresh read-back and validation when required
-> retention and audit
-> reverse close
-> public service result
-> thin app, tool, or protocol projection
```

Response loss never permits a frontend or service to repeat an effect or choose among effects by newest, first, same name, or apparent equality.

## Release and build boundary

Source, dependency, toolchain, build, signing, distribution, installation, and support owners are external adapters or processes selected by exact E7-B profiles. Service sees typed plans and receipts, never:

```text
raw shell, Cargo, rustc, linker, or process command surface
arbitrary environment block
private signing, distribution, build, or provider credentials
raw GitHub or provider API client and payload
installer, filesystem, process-manager, or replacement-helper handle
```

`tools/wow-release` and `apps/wow` cannot import or call these adapters directly.

## Final workspace activation

The final topology is documented, but root Cargo workspace membership activates incrementally:

```text
I0-A: wow-core only
then I0-B through I0-F as implemented
then I1 through I7 packages in dependency order
```

No empty final-topology crates are activated up front.

## Forbidden patterns

- foundation depending on higher framework crates;
- owner depending on service, application, or tool;
- store interpreting domain semantics;
- graph parsing source, running recognizers, authorizing, or calling service;
- recognizers publishing service or store current state;
- project or reference mapping depending on `wow-cbm` or service;
- context accepting provider fields as semantic facts;
- service reproducing owner algorithms or exposing raw owner handles;
- application or tool importing any framework crate except `wow-service`;
- generic MCP, tool, RPC, shell, script, plugin, model, process, SQL, provider API, or installer path;
- source or provider locator opened by service or app instead of an owner;
- cross-provider score fusion or Candidate promotion;
- implicit latest, best, previous, default, top, sole, nearest, LKG, or LKR selection;
- session, transport, release, channel, or installation state used as semantic authority;
- sensitive adapter material crossing public seams;
- production crate depending on application or tool;
- CI or release workflow defining a second semantic or release pipeline.

## Changing the graph

A new edge requires:

```text
exact crossing data or operation
why the current owner-neutral or public seam is insufficient
cycle and identity analysis
security, privacy, license, and supply-chain impact
proof-ceiling and failure or recovery impact
boundary fixtures and mutations
Cargo metadata, manifest, and workstream changes
compatibility and migration consequences
accepted ADR when architecture changes
```

Implementation convenience alone is not sufficient.
