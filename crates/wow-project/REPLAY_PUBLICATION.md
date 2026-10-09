# Native live project pair publication

The W13 owner/service path accepts an already published physical Lua or selected
standalone TOC/XML project and its exact graph. Native service operations and the
[`wow project` CLI](../../apps/wow/LIVE_PROJECT.md) expose this profile; full E2-D
acceptance remains open. Multi-package loader plans currently reject with
`DeferredCapability` / `OperationNotImplementedForMilestone`.

`ProjectReplay::capture` archives the original Main files, fixture references,
exact Library workspace inputs/universes, function-call option and full physical
configuration. The DTO contains source text and belongs in the explicitly owned
local store. It is distinct from a `ProjectView`, analyzer state or a public DTO.
The archive precharges at most 8,192 files, 64 Library workspaces, 16 MiB per file
and 32 MiB aggregate source bytes before copying. Encoded records also obey the
existing store's 32 MiB record and 64 MiB generation ceilings; JSON escaping may
make an otherwise admissible source set exceed the record budget.

Physical-input archives retain the exact v1 encoding. Standalone TOC/XML archives
use `wow-project/native-project-replay/2` and the `wow-project.live-replay.v2`
storage schema. They add a selected TOC, explicit selection context, raw consumed
TOC/XML documents and expected load-plan digest. These documents count against
the same aggregate archive budget. Loader-specific ceilings remain 1,024 consumed
files, 1 MiB per source and 16 MiB aggregate, plus existing parser/index limits.
The replay runs the same loader using a typed retained source port; it neither
deserializes a load receipt nor creates temporary source files. Exact consumed
sets, source digests, decisions, indexes and plan IDs must reproduce. Unselected
or unavailable files are not discovered during replay.

`hydrate` rebuilds configuration and Library workspaces through their existing
validators, runs `ProjectPublisher::publish_initial_cancellable` over the exact
inputs and compares original project/analyzer snapshot identities. It does not
deserialize an analyzer session or promote compatibility/coverage from labels.
Library sources remain separate from Main. The resulting ProjectView offers the
existing freshly recomputed report capabilities; no additional semantic operation
or target-client runtime capability follows.

`ProjectPublicationBundle::build` validates the live project and graph, recomputes
source19 proposals through the existing source owner and requires the exact source
batch, registry, canonically ordered coverage and empty source foundation. Other
producer partitions remain graph-owner validated data; this does not independently
replay every recognizer or establish complete conflict assessment.

Logical graph records and the replay record obtain stable partition-version IDs
first. Project, analyzer, graph and complete logical member versions then derive
`ProjectPublicationSetId`; its header and bindings enter `StoreGenerationId`
afterward. Epoch, current, store row identifiers and host paths cannot enter the
semantic project/graph/publication-set recipes.

`wow_service::live_project::LiveProjectStore` registers a separate live epoch.
It reuses existing store phases: prepare inactive membership, fresh exact
read-back, graph/native-replay owner validation and current CAS. Failure leaves
current unchanged. An uncertain commit is reconciled once, without repeating an
effect. Original graph-bundle publication remains a distinct registered profile;
the live API cannot adopt or relabel that epoch.

`LiveProjectStore::read` resolves Current or Exact once and returns
`LiveProjectRead`, which owns the read transaction/lease together with the actual
ProjectView and GraphPartitionSnapshot. All membership, bindings, header, native
replay and source-pair checks finish while the lease is held. Existing readers
stay on the old generation after activation; later readers see the new pair.
Missing or inconsistent records return a typed unavailable/failure outcome.

Checked on 2026-10-09: full workspace policy, fmt, check, strict Clippy, tests
(844 passed, 1 ignored, 103 targets), rustdoc and build. Focused tests cover actual
close/reopen, Current/Exact acquisition, old leased readers, CAS rejection,
Library/Main separation, prepublication cancellation, missing/mutated replay and
mixed project/graph rejection. The later CLI/service checkpoint additionally
passes the complete producer chain and public read/reconciliation projections
(847 tests passed, 1 ignored, 103 targets). Loader-plan replay, incremental reuse, crash,
backup/GC, real-addon, source-parity and full W13/E2 acceptance remain open.

Fresh full workspace gates and build passed for standalone replay on 2026-10-09:
849 tests passed, 1 ignored, 103 targets. Tests remove the source directory before
hydrate and full service composition, compare the exact retained plan/semantic IDs,
reject missing/mutated/surplus/context/schema substitutions and exercise store
reopen under Exact. Existing physical epochs retain their original catalog and
epoch IDs through a strictly validated legacy open; v2 writes require a new store.
There is no migration or catalog widening in place. Package replay, generation
Library binding and full acceptance remain open.
