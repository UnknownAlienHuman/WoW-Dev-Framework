# E3-B platform UI source decisions

**Status:** normative.

## PSRC-001 — Acquisition and indexing are separate

Network/Git/CASC/provider access belongs to an explicit materializer tool. `wow-project` consumes a closed snapshot only.

## PSRC-002 — Content identity is primary

Provider/repository/ref metadata is provenance. Canonical source manifest and bytes, corpus profile, and tool profiles determine source/corpus identity.

## PSRC-003 — Branch labels are never pins

`live`, `ptr`, `beta`, and other branch labels are recorded but cannot be used as immutable input.

## PSRC-004 — Provider trust is explicit

Community mirror, local client extract, user-supplied archive, and official/publisher source classes remain distinct. No adapter upgrades trust by convenience.

## PSRC-005 — Build observations may conflict

Requested profile, provider commit message, `version.txt`, TOCs, generated docs, and runtime/client build are independent observations.

## PSRC-006 — One flavor/build snapshot per corpus generation

No cross-branch/flavor/profile merging to fill missing files or metadata.

## PSRC-007 — Complete materialized inventory precedes parsing

Parsers receive a closed root-confined file inventory. They never discover/download files dynamically.

## PSRC-008 — Provider lists are hints

`ui-code-list.txt`, `ui-toc-list.txt`, and similar files are retained and reconciled against actual manifest/TOC parsing. They cannot override bytes.

## PSRC-009 — Source roles remain distinct

Implementation source, generated API documentation, provider inventory, repository automation/metadata, tests/tools, unsupported, and unknown files use separate roles and coverage.

## PSRC-010 — Generated APIDocumentation is not re-evaluated here

`wow-reference` owns API documentation evaluation. E3-B may inventory/source-map those files but cannot create Reference Pack authority from them.

## PSRC-011 — One logical platform workspace

The corpus has one logical source/analyzer universe. Physical shards are implementation partitions with explicit equivalence/coverage, not separate truths.

## PSRC-012 — Package partitions are replaceable

Source, TOC/XML/load, analyzer, recognizer, and graph outputs are versioned by exact package/file/capability partitions for incremental rebuild.

## PSRC-013 — Global load structure is source-order based

TOC/package dependencies, selected variants, XML includes/scripts, and bootstrap/LOD roles define the static load model. It does not prove runtime success/readiness.

## PSRC-014 — Dedicated graph universe

All platform implementation entities use `pinned_platform_ui_source` plus exact corpus generation. User project and reference universes remain separate.

## PSRC-015 — Source/reference links are explicit

An implementation occurrence may link to an exact reference entity through a graph relation with separate project-location and platform-contract evidence.

## PSRC-016 — Corpus publication is separate

Platform source uses a separate ProjectStore identity/publication from each user addon project. No shared current pointer.

## PSRC-017 — Context consumes an auxiliary exact view

E3-A receives a SourceUniverseManifest and exact project/graph/source-detail views. It never refreshes or chooses a corpus.

## PSRC-018 — Missing license evidence blocks redistribution claims

No root license file or explicit rights record means redistribution state is unknown/restricted, not permitted by default.

## PSRC-019 — Repository automation never executes

Workflows, hooks, filters, scripts, generators, and tools are source data/ignored metadata only.

## PSRC-020 — No source-level runtime proof

Source structure cannot establish hotfix/data/state behavior, event payload accessibility, Secret state, taint, combat/protected legality, or performance.

## PSRC-021 — Incremental reuse is exact

Same path/name/size/mtime is insufficient. Reuse requires exact content, profile, tool, dependency, coverage, and producer identity.

## PSRC-022 — Similarity is not lineage

Moves/renames/replacements remain E4 lineage/impact work. E3-B records add/remove/change and optional nonauthoritative candidates only.

## PSRC-023 — Source pointers over dumps

Normal artifacts retain exact repo/provider/content/source-handle/path/span pointers. They do not duplicate the corpus into framework documentation.

## PSRC-024 — Partial corpus publication is honest

A partial package/analyzer/graph corpus may publish only under explicit policy with all unsupported/conflicted/truncated areas retained.

## PSRC-025 — Determinism excludes provider timing

Download order, archive order, Git object order, filesystem enumeration, worker completion, clocks, and host paths do not alter logical IDs.

## PSRC-026 — Mirror fixture is not production hard-coding

`Gethe/wow-ui-source@027d26c...` is a pinned fixture. Repository/package/path names cannot become universal semantic conditions.

## PSRC-027 — Materializer security is fail-closed

Path traversal, links, submodules, LFS/filter requirements, special files, case/Unicode collisions, incomplete downloads, and budget overflow block or explicitly scope the snapshot.

## PSRC-028 — No hidden current source selection

Higher orchestration explicitly chooses exact corpus publication. A branch head movement never changes an existing request.
