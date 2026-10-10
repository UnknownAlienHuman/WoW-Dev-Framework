# W17 deep process and product audit — 2026-10-10

Status: active audit record. This document supplements, but does not replace,
`docs/EXECUTION_MODEL.md` and canonical GitHub Issues.

## Exact inspected heads

```text
WoW-Dev-Framework process head: ba3ccfa83d9dfc80fe97255f64692464f6f11b14
latest W17 product predecessor: 747ec584964ee812ea256d0ddf6186094c56937d
Gethe/wow-ui-source live: 09b9db7948abc9b9648dedaab51eb0cf3ee67b31
Gethe version evidence: 12.1.0 (69933)
```

Moving selectors are operation evidence and must be resolved again for a later
operation.

## Confirmed process corrections

The repository now has:

- one canonical Issue per active/acceptance work package;
- one-manager/one-worktree/one-write-lease policy with finite expiry/heartbeat and
  crash/takeover rules;
- Tier S functional checks, Tier M exact-head checks, Tier A acceptance and Tier X
  external/runtime qualification;
- a push/PR `Quick exact-head gate` without the full test/rustdoc/release matrix;
- no-product-stacking rule while the current quick gate is pending or failed;
- a two-failed-attempt audit/change-route rule;
- Gethe/Ketho routing limited to tasks whose correctness actually depends on it;
- safe manual branch classification with exact-main input and dry-run default;
- obsolete W11-W17 tracking/specification PRs closed without merging stale heads.

The exact process head above passed Quick run `38059947722`; artifact
`11673330491`, digest
`sha256:0374c12337f136f32c5955617ce495842a4c074f5c64ab352f80df1e91565633`,
retained for 14 days. Remaining process risks are tracked in Issues #104, #105
and #112.

## Hard product blocker: current corpus exceeds the XML site profile

`crates/wow-project/src/graph/xml_source_maps.rs` defines:

```text
MAX_SITES = 4096
```

The owner refuses when the script-fact map or analysis/unresolved set exceeds
that limit. At exact Gethe revision `09b9db...`, GitHub code search reports at
least the following number of distinct XML files containing each distinct handler
tag:

```text
OnLoad       788
OnEvent      582
OnShow       552
OnHide       487
OnClick      398
OnUpdate     182
OnEnter      491
OnLeave      490
OnMouseDown  145
----------------
lower bound 4115
```

These are distinct tag spellings, so their occurrences are disjoint. The search
counts files containing at least one tag, not total occurrences. Therefore 4115
is already a conservative lower bound and proves that the current full-mirror
mapping path cannot fit the reviewed `MAX_SITES=4096` profile. Additional handler
tags and multiple occurrences per file increase the actual total.

This is a hard blocker for claiming a full current-source W17 mapping or exporting
a complete `SkeletonInputView`. The fix must be based on an exact manifest/corpus
census and a reviewed bounded strategy; replacing 4096 with an arbitrary huge
constant is not acceptable. See Issue #113.

The same census must measure all inherited ceilings, including:

```text
MAX_FILES
MAX_LOADS
MAX_NODES
MAX_EDGES
MAX_PIECES
MAX_TEXT_BYTES
per-package/source/profile limits
```

Exact-limit and limit+1 cases, partial/truncation semantics and real-corpus
headroom must be explicit.

## Hard graph blocker: axes reject rich registries

`GraphAxisProfile::bind()` locates relation definitions by
`GraphRelationKind` and rejects a second matching definition. Current registries
intentionally contain several distinct relation IDs mapped to the same broad enum,
including multiple `Owns`, `Loads` and `DependsOn` definitions.

Examples include:

```text
source_package_owns
xml_owns_template
toc_owns_state
xml_occurrence_source_span
xml_script_site_owns_virtual_lua
xml_map_piece_source_span

source_loads
source_package_loads
toc_loads

source_depends_on
source_package_depends_on
toc_depends_on
```

Consequently `GraphAxis::Ownership` and the Load axis can be structurally
unavailable on the actual rich W17 registry even when all relevant graph data is
present. Synthetic axis tests that use one definition per enum do not cover this.

The implementation must not select the first matching definition. It needs a
reviewed definition-aware axis model, distinct relation kinds, or removal of
redundant semantic edges. In particular, `xml_map_piece_source_span` is currently
classified as `Owns`, although a map piece maps/references an admitted span and
already carries exact source support. See Issue #114.

## Hard architecture blocker: native-only profile ladder

The actual service and replay path reconstructs
`build_platform_graph_proposal_plan()` and therefore selects
`wow-project/platform-direct-producers/1`. Inventory spans `/2`, structural roles
`/3` and XML source maps `/4` are separate native-library-only entries. The W17
documentation explicitly says service/replay remain on `/1`.

Adding object classification, fingerprints and later stages as `/5`, `/6`, and so
on would create an expanding compatibility surface without one application
consumer. Before another aggregate recipe, select either one composed W17 profile
or explicit independently versioned capability composition. Classify `/1`–`/4`
as released/external, persisted/admitted, frozen fixture, or transient unreleased
and name every actual selector/replay/store/fixture consumer. See Issue #115.

## Hard architecture blocker: missing E3-A candidate boundary

Normative E3-A documents require:

```text
validated owner inputs and graph proposals
-> one immutable BlizzardUiIndexCandidate
-> exact GraphPublicationPlan / BlizzardUiPublicationBundle
-> fresh inactive read-back and current activation
-> BlizzardUiProjectView
-> bounded SkeletonInputView
```

Current Rust has no `BlizzardUiIndexCandidate` or complete equivalent owner.
`ProjectPublicationBundle` stores graph records plus replay/header bindings and
validates an existing pair. It does not own the complete E3-A candidate contract:
source/package/load/analyzer/recognizer manifests, proposal validation/rejection,
fingerprints, skeleton-input manifest, and capability/coverage/conflict/truncation
summary. Its direct validator reconstructs only producer `/1`.

Do not expose `SkeletonInputView` directly over raw ProjectView/graph/store
internals. Implement or extend one immutable store-independent candidate
capability, then make publication and Skeleton construction consume it. See Issue
#116.

## Identity audit observation

Native XML source-map `/4` starts from source graph profile
`wow-project/source-load-proposals/22` and then relabels the retained
`ProjectGraphProvenance.profile` as
`wow-project/platform-direct-producers/4` before admitting new piece support.
The current documentation describes this as an augmented `/4` source report, so
this is not yet classified as a separate defect. Before persistence or Skeleton
handoff, verify explicitly that:

- source graph profile and direct-recipe profile are separately recoverable where
  downstream compatibility needs both;
- no consumer compares the relabeled field with `source_graph_profile(config)`;
- replay/migration cannot confuse source `/22` with producer `/4`;
- serialized identity tests cover both meanings and old profiles.

If one field is serving both identities, split it before acceptance rather than
adding more relabeling exceptions. This audit belongs in the #115/#116 design.

## Required operating improvements

Before the next W17 product write:

1. The manager posts an exact writer claim in Issue #106: manager/session label,
   one worktree, exact base, owned paths/contracts, slice, Tier S chain, blockers,
   finite lease and heartbeat.
2. On chat/host loss, the next manager reconciles remote `main`, the Issue log and
   named retained artifacts; unpublished local state is presumed lost.
3. Tier S passes before publication. Do not stack another product commit while the
   current `main` quick gate is pending or failed.
4. After two failed attempts at the same approach, write a short failure audit and
   change route instead of repeating patches.
5. The agent reads only: `AGENTS.md`, `docs/EXECUTION_MODEL.md`, the canonical
   active Issue, target owner contracts and exact source evidence needed for the
   slice. Historical checkpoint diaries are not hot context.
6. Gethe/Ketho resolution is required only when correctness depends on current WoW
   source/API semantics. Process/docs/store-generic work must not perform unrelated
   source/Ketho operations.
7. Native Ketho annotation execution belongs to annotation/source-normalization
   work or Tier X parity, not every task containing the word “source”.
8. Functional completion includes a reachable application positive path,
   conservative incomplete/dynamic/budget/cancellation behavior, Tier S pass,
   exact-head publication/read-back and one concise Issue update.

## Evidence durability and supply-chain follow-up

Quick exact-head evidence is now retained for 14 days and immutable
commit/tree/toolchain/artifact identity is recorded in the relevant Issue. Before
using CI configuration as a hardened supply-chain boundary:

- pin first-party GitHub Actions to reviewed commit SHAs and update them through a
  controlled dependency process;
- keep PR runs read-only and free of credentials/secrets;
- enable manual `main` protection against force-push/deletion and require linear
  history; later require the quick gate before merge.

## W17 dependency order after blockers

```text
1. Resolve #113: exact corpus census and reviewed bounded profile/sharding.
2. Resolve #114: definition-aware axes/relation semantics.
3. Resolve #115: one selected composed profile or explicit capability composition.
4. XML object/region classification over reviewed exact owner facts.
5. Stable fingerprints with collision and compatibility policy.
6. Resolve #116: one validated BlizzardUiIndexCandidate consumed by publication.
7. Bounded SkeletonInputView from the acquired candidate capability.
8. Source service/CLI transport.
9. Tier M exact-head milestone.
10. Tier A package acceptance.
11. Tier X Gethe/Ketho/WoW API Ketho MCP discrepancy matrix and named runtime probes.
```

No downstream context owner should reconstruct W17 internals or work around a
failed budget, axis, selected-profile, or candidate contract.
