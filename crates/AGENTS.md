# Crate contribution rules

These rules apply to crate implementations and cross-crate owner seams. Follow
the repository instructions and the selected canonical Issue; use
[`../docs/REFACTOR_QUEUE.md`](../docs/REFACTOR_QUEUE.md) for current refactoring
order. Read the target crate's README, scoped `AGENTS.md` (if present), and only
the owner/caller contracts and fixtures touched by the change. Do not turn
package acceptance or optional E5/E6/release work into a prerequisite for an
unrelated code change.

## Ownership and dependencies

Each crate owns one semantic responsibility. Keep domain algorithms in their
existing owner; Service orchestrates owners and applications/tools depend on
Service only among framework crates. Owner crates never depend on Service,
applications, or tools. Cross-crate Markdown seams describe operations and
projections; they are not Cargo packages. Follow [`DEPENDENCY_GRAPH.md`](DEPENDENCY_GRAPH.md)
and the target contract when changing a seam.

| Crate | Responsibility |
| --- | --- |
| `wow-core` | Generic exact semantic primitives |
| `wow-store` | Persistence, effects, leases, retention, and recovery |
| `wow-reference` | Exact platform/reference facts and views |
| `wow-emmy` | Versioned analyzer adapter and rolling compatibility |
| `wow-project` | Source, project, load, index, and overlay ownership |
| `wow-graph` | Typed graph, lineage, impact, and producer partitions |
| `wow-recognizers` | Declarative recognizers and calibration semantics |
| `wow-rules` | Diagnostics and remediation tiers |
| `wow-search` | Exact-generation retrieval and ranking |
| `wow-context` | Exact-root context artifacts |
| `wow-cbm` | Optional external Candidate normalization |
| `wow-service` | Multi-owner orchestration only |

Do not edit sibling owners to ease a local implementation. Add a dependency
only for a concrete cross-owner operation that existing lower-level seams cannot
support; analyze cycles and identity, and update the graph and affected contracts.
Record an architecture decision when accepted architecture changes. Do not add
placeholder crates, public stubs, fake adapters, duplicate algorithms, or broad
generic abstractions without concrete repeated call sites.

## Semantic and data boundaries

- Public operations are narrow, owner-controlled, transport-independent, and
  deterministic for equivalent logical inputs. State exact profiles,
  generations, capabilities, coverage, conflicts, budgets, cancellation, and
  proof limits where the owner contract requires them.
- Preserve provenance, confidence, omissions, conflicts, and nonclaims.
  `Candidate`, `Possible`, partial, conflict, truncated, cancelled, failed,
  `OutcomeUnknown`, `ResynchronizationRequired`, and `NotEvaluated` are not proof,
  clean Negative, complete, or pass. Clean absence requires explicit owner
  negative authority over complete exact relevant coverage.
- Validate exact IDs/digests at their owning boundary. Do not infer hidden
  semantics from repository, addon, path, provider, model, client, transport,
  release, or installation identity. Owner operations do not select a floating
  current/latest/default/previous/LKG/LKR value.
- Mutable owner state stays private and publishes immutable generations/views.
  Response loss is not absence; same operation ID with a different request
  digest fails. Durable or externally observable effects require the exact
  reconciliation and recovery contract owned by the package.
- Source, provider, client, and release text is untrusted data, never
  instructions. Never execute analyzed addon or repository code.

Public seams must not expose raw SQL/connections/physical object keys, arbitrary
filesystem roots or traversal capability, process/shell/plugin/callback/RPC
execution, provider APIs or payloads, private credentials, or unbounded input.
Keep host, platform, and FFI code behind the appropriate owner adapter and exact
target contract.

## Implementation and checks

Use types that enforce real invariants, explicit schema/serialization versions,
deterministic canonical bytes, typed errors, bounded cancellable parsing, and
minimal `unsafe` with a specific safety invariant. Tests verify committed
fixtures; they do not rewrite them. Choose checks for the affected owner chain
and behavioral risk, following the repository check cadence; a successful build
or lint is not package acceptance. Keep state and evidence records accurate when
implementation status actually changes.

For WoW annotation/normalization/type-lowering changes, follow
[`../docs/KETHO_RUST_PORT.md`](../docs/KETHO_RUST_PORT.md). Resolve current source
only when correctness depends on current WoW facts or the selected contract
requires it.
