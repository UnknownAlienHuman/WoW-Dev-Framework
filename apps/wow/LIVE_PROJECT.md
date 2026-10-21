# Native live project commands

`apps/wow/src/project.rs` dispatches local commands through
`wow-service::live_project`. Physical Lua, selected standalone TOC/XML and
package inputs support publication/read. This describes existing machinery, not
an obligation to preserve it: #131 replaces expensive reads and #111 removes
unneeded lifecycle administration.

## Commands

```text
wow project publish --config project.json --project <ProjectId> --store-root <new-private-directory> --operation-id <id> --expected-current absent --initialize --allow-partial
wow project read --store-root <directory> --store-generation current
wow project read --store-root <directory> --store-generation <StoreGenerationId>
wow project reconcile --store-root <directory> --operation-id <original-id>
wow project update --config final-project.json --project <ProjectId> --store-root <directory> --operation-id <id> --expected-current <record-id> --library keep --allow-partial
wow project recover --store-root <directory> [--domain-current] --format json
```

All commands support `--format json|text`. Store directories contain retained
source and must be explicitly owned. Source/configuration is not executed or
rewritten. Only one owning process opens a store at a time.

## Publication, read and update

`publish` builds the native Project and graph, retains Main/Library bytes,
checks the inactive readback and activates against the supplied expected Current
record. Later publication omits `--initialize`; initialization does not adopt an
existing store. Use the returned Current record for subsequent changes.

Every current input uses one strict `ProjectReplay` DTO and one
`wow-project.live-replay` record. Configuration retains Library selection,
function-call facts and TOC/package/platform choices. There is no numbered replay
or feature-specific catalog. Incompatible derived stores are rebuilt, not migrated;
user source/configuration remains intact.

`read` resolves Current or Exact once and holds the original transaction/read
lease. It still reconstructs Emmy/Project and graph owners and recaptures replay
before returning IDs and file/Library/node/edge counts. This is not a cheap index
query; #131 removes that cost. It never substitutes another generation. An older
leased reader retains its acquired pair after Current advances.

Ordinary `update` requires a physical-Lua publication and explicit final input.
It derives Add/Update/Remove, recomposes the graph and publishes against the
expected Current record. Standalone TOC/XML and package inputs can publish/read
but are not accepted by this physical-only update route (#109).

`--library keep|replace|clear` is required. Keep checks that the final Library
inventory matches the retained one; Replace selects the supplied set. Clear and
empty Replace are refused by the current mandatory-Library policy, not silently
converted to Keep. Update does not initialize a store.

NoChange retains Current without consuming an operation ID. A changed request
uses the durable publication receipt: retrying its original ID/base/target returns
the same outcome; a different target conflicts. Request reconstruction may still
repeat analysis. `reconcile` observes the original operation without publishing
again; use it after an uncertain acknowledgment rather than inventing a new ID.

## Selected platform commands

`--selection <selection.json>` supplies `wow-service/platform-project-selection/1`,
`owner_project_id`, `source_profile_id` and selected configuration. The optional
`composed_candidate=true` adds the candidate receipt/check, not another replay
format. Publication and read verify the actual configuration and rebuilt owners.

Selected publish/update currently accept `--function-fingerprints <query.json>`
and `--xml-fingerprints <query.json>`. Their existence does not restore the removed
fingerprint-platform scope; #118 audits remaining useful consumers.

```text
wow project skeleton --selection <selection.json> --store-root <directory> --store-generation current|<generation-id> --request <skeleton.json> [--format json|text]
```

Skeleton requires the composed candidate and a
`wow-service/live-skeleton-read-request/1` containing `candidate_id`,
`candidate_digest` and `input`. Its current one-hop profile exposes a root,
support/handles and bounded direct neighbors, not source excerpts or complete
context. Selected update uses the retained platform owner and rebuilds the
candidate; query-only changes can publish a new candidate. Selected read/Skeleton
keep result bytes private until the held read and Store close successfully.

## Recovery and output boundaries

`recover` observes physical store state; it performs no repair or activation.
`--domain-current` additionally reconstructs the exact Current Project/Graph pair
and reports its outcome beside the physical report. Lower Store recovery code
still exists, but Service backup, restore/replacement and quarantine APIs have
been removed. Do not document or rebuild those removed wrappers.

JSON is a bounded service result plus LF. Publication/read return 2 for partial
source coverage; reconciliation returns 0 for an observed operation or 3 if absent.
Service/output failures return 4, invalid arguments 64 and precommit cancellation
130. Recovery uses its own coverage/outcome mapping in `project.rs`; successful
observation does not authorize repair or prove complete semantic analysis.

Current replay behavior is described in [PLATFORM_SOURCE_REPLAY.md](../../docs/PLATFORM_SOURCE_REPLAY.md).
Build/Clippy results are not end-to-end acceptance. Use existing consumer cases
for the changed path; actual addon/search completion remains #125/#201.
