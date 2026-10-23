# Implementation order

The [manifesto](ARCHITECTURE.md) defines the product. This is the only ordered
queue; [IMPLEMENTATION.md](IMPLEMENTATION.md) supplies code routes and
[WOW_LINTER.md](WOW_LINTER.md) supplies behavior. Existing Issue numbers identify
related work, not permission to restore historical contracts or test campaigns.

## 1. A useful ordinary addon check

Related: [#200](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/200),
[#201](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/201),
[#197](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/197).

Connect root/TOC/flavor/Library input to the existing shared Emmy session and
public CLI. Resolve one analyzer configuration, initialize Lua51/STD, and
preserve upstream message/code/severity/range
and remove the four-code allowlist. Select checks before doing their work.
Return known findings separately from input/analysis limitations. Remove
compatibility-report and Project publication requirements from this path.
Derive completeness from selected-check dependencies; the current global
component reducer makes native checks partial independently of their findings.

The first usable result is a typed addon check with an existing Library. It
requires neither a persistent source index nor completion of all old cleanup.

## 2. Correct Blizzard declarations for that check

Related: #200/#201; reuse the delivered builder and native Ketho port.

Separate client flavor from API environment; handle member-level `SecureOnly`.
Model `MayReturnNothing`, `NilableContents`, ordered repeated stride groups,
mixin relations and relevant restriction metadata.
Keep unresolved types/omissions visible. Use one source-to-Library pipeline,
consistent reader limits, batched cancellable Git reads and a simple metadata/map
handoff. Do not recreate the removed example producer or `/1` verifier.

The concrete current-source cases are `C_ArtifactUI.GetMetaPowerInfo` (repeated
triples) and `C_EquipmentSet.GetItemIDs` (zero results versus nullable table
contents). Connect the small metadata handoff to the Library reader; leaving it
as an ignored raw sidecar does not finish this step.

## 3. Correct project context and changes

Related: [#109](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/109),
[#117](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/117).

Connect TOC/XML/dependency selection, addon chunk namespace, supported handler
contexts and precise virtual-source maps to the same owner. Distinguish cosmetic
metadata from unknown load conditions. Handle changed/deleted files and changed
load/Library selections without stale results. Retain conservative semantic
reindexing until a narrower invalidation scheme is demonstrably correct.
Allow XML-only input through the physical-inventory boundary. Reuse the existing
single-TOC/package inline analysis and maps; implement the separate per-addon
chunk context rather than mistaking package path prefixes for that context.

Remove the stale `InlineLuaNotAnalyzed` file-closure blocker for supported
extracted bodies without claiming implicit handler context is already complete.
Connect the TOC/package-selected update path, which currently refuses live
updates, to selection/context rebuilding.

## 4. Small WoW checks and editor/agent reuse

Related: [#103](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/103),
[#195](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/195),
[#107](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/107),
[#198](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/198),
[#199](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/199).

Use direct typed checks for supported API context, widget/template, event/callback
and restricted-use cases. API absence and runtime safety must not be guessed.
Bind API uses through Emmy lexical/source identity before exact absence lookup;
the current typed-local shadowing path can misattribute a Lua member error.
Separate unresolved Secret producer semantics from malformed fact identities;
the former limits the affected check, while the latter remains an input error.
Replace the one-file LSP path with the configured project; expose only useful
read-only MCP operations. Keep standard framing, empty results and increasing
LSP document versions, including valid gaps. Remove generic pack/matcher/proof
and frontend profile wrappers as
these consumers move. #103 owns useful recognizer/check selection, not a new
engine; early execution selection belongs to the check route above.

## 5. Direct source lookup and remaining platform removal

Related: [#131](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/131),
[#169](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/169),
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122),
[#106](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/106).

Store current files/body text/symbols/useful relations in one SQLite database.
Read after reopen without rebuilding analysis; return readable matching excerpts.
Remove nested databases, replay recapture, self-sizing pages and Context packs.
Keep the useful outline/direct-neighbor query, exact scope selection and
empty/partial/truncated result distinctions in the direct implementation.
The known remaining fingerprint code is removal work with its consumers under
[#118](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/118), not an
already-deleted subsystem or a reason to repair its historical product.

Attach remaining Store lifecycle/DTO/format cleanup
([#111](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/111),
[#194](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/194),
[#112](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/112)) to the
reader it replaces. Measure the actual index/query path when it is usable
([#113](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/113)); do not
invent performance targets from line counts.

## Working policy

One manager/worktree, bounded delegation, code first, formatting and affected
strict Clippy. Tests are deferred until functional completion by owner direction;
then reconcile relevant known failures and exercise the advertised workflows.
The supplied audit's test failures remain evidence, not instructions to restore
retired scope. A component or compilation pass alone never completes
[#125](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/125).
