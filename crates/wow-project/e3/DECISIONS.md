# E3-A Blizzard UI source-index decisions

**Status:** normative.

## BUI-001 — Platform source is a distinct universe

Blizzard UI source records use `blizzard_ui_source`; they never share identity with user project, reference, runtime, external candidate, or historical records.

## BUI-002 — Materialization is outside the library

E3-A consumes a closed byte snapshot. Git clone/fetch/archive download/client discovery belongs to an explicit external materializer/application boundary.

## BUI-003 — Exact content, not branch name, defines input

Provider repository, branch, release, or build labels are provenance. Canonical input identity uses exact file inventory and content digests plus profiles.

## BUI-004 — Source is implementation evidence, not platform contract authority

Source may prove exact code/declaration/load/template facts in the pinned snapshot. Reference Packs remain API/restriction authority; runtime evidence remains runtime authority.

## BUI-005 — Reuse the E2-C parsers and analyzer path

No second TOC/XML/Lua implementation is introduced for Blizzard UI source.

## BUI-006 — One exact client flavor/profile per index

Retail, PTR, Beta, Classic, and other variants cannot be merged to fill missing facts.

## BUI-007 — Complete inventory and semantic coverage are separate

All configured files may be inventoried while dynamic calls, generated globals, runtime branches, or malformed partitions remain unresolved.

## BUI-008 — One platform-source ProjectStore per logical source profile

The platform corpus publishes as its own project/store identity using the generic E2-D physical model. It is not inserted into a user project's store by path convention.

## BUI-009 — Graph partitions remain producer-owned

Source inventory, TOC/load, XML, analyzer-derived, project adapter, and recognizer rule partitions replace independently and atomically.

## BUI-010 — Core recognizers only in E3-A

Named product/framework calibration packs remain E5. Source/provider/package names never enable hidden rules.

## BUI-011 — Static load is not runtime execution

TOC/XML/dependency order and reachability never prove that the client executed a unit successfully or that a frame/event/state is available at runtime.

## BUI-012 — Exact cross-universe edges only

Project-to-reference or user-project-to-Blizzard-UI edges require explicit relation kinds, exact endpoint resolution, and evidence. Same names are insufficient.

## BUI-013 — No source-wide transitive edge materialization

Store direct edges and bounded reason paths; do not persist every reachability pair.

## BUI-014 — Historical continuity is deferred

E3-A may emit structural fingerprints for later comparison, but it does not assert `same_lineage_as`, `moved_to`, `replaced_by`, or patch impact.

## BUI-015 — Skeleton generation is downstream

E3-A exports exact structured inputs. `wow-context` owns L0/L1 layout, summaries, budgets, selection, and context-pack manifests.

## BUI-016 — Ordinary reads are bounded

No default full-source or whole-graph dump. Exact/bounded package, file, symbol, relation, span, and neighborhood operations only.

## BUI-017 — Source text is untrusted data

Comments, strings, docs, XML text, metadata, and generated declarations cannot alter parser configuration, graph schema, recognizer pack, or agent policy.

## BUI-018 — Redistribution is explicit

Persistent local indexing does not imply permission to ship source bytes. Every artifact states whether it contains only handles/metadata, bounded excerpts, or redistributable bytes.

## BUI-019 — Provider outage cannot change an existing generation

Once materialized and published, source generations remain immutable and independently addressable; later network/provider state is irrelevant.

## BUI-020 — Rename invariance is mandatory

Provider/repository/root/package display/path names cannot change universal facts except where exact path/package metadata is itself the declared source fact.

## BUI-021 — Build/profile mismatches block publication

Source labels, TOC Interface/profile, ReferenceProfile, annotation artifact, analyzer, graph registry, and store publication must form one explicitly accepted compatibility set.

## BUI-022 — Removed source leaves no dangling derived record

Target publication must prove stale source units, analyzer facts/findings, recognizer matches/proposals, graph assertions/indexes, source handles, and skeleton inputs are absent or retained only under the prior generation.

## BUI-023 — Physical persistence does not enter source semantic identity

SQLite rows/pages/WAL/checkpoints/epoch paths never affect source/project/graph/skeleton-input IDs.

## BUI-024 — No client-runtime data in E3-A

Installed SavedVariables contents, logs, combat state, event payloads, secure execution state, and runtime object instances are excluded.
