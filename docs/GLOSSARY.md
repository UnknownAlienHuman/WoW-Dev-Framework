# Glossary

**Status: normative terminology**

## Reference Pack

An immutable, profile-specific artifact containing normalized Blizzard API/UI facts, raw metadata, annotations, source maps, skeletons, checksums, licenses, and capability/coverage state.

## Profile

An exact World of Warcraft target identified by flavor/edition, Interface, client build, source revision/digest, schema versions, builder version, and correction set. “Current” is not a durable profile ID.

## Reference generation

The immutable identity of a built Reference Pack.

## Project generation

An immutable snapshot published by the project actor after applying a coherent set of file/configuration updates against one Reference Pack profile.

## External generation

The revision/index identity of an external repository or Codebase Memory result set.

## Capability

A named analysis/query function whose availability depends on specific source partitions and tool behavior, such as `apidoc.signature.complete` or `project.toc.complete`.

## Coverage partition

The smallest named source/analysis area for which completeness can be reported independently.

## Negative authority

Permission to state that an entity/fact is absent. It requires complete relevant coverage, a known profile/generation, and no unresolved conflict.

## Provenance

The class and exact origin of evidence, such as platform source, project source, runtime probe, curated correction, differential oracle, external implementation, semantic candidate, or historical record.

## Confidence

The relation between evidence and conclusion: `Proven`, `Derived`, `Possible`, or `Candidate`.

## Stable source handle

A compact identity for source detail containing repository/pack identity, revision/profile, path, span, digest, and optional symbol/entity key.

## Entity

A typed graph node such as API symbol, function, package, frame, template, module, registry, state path, restriction facet, or source span.

## Relation

A typed directed edge between entities, with evidence, confidence, generation, and coverage.

## Parent axis

One independent hierarchy such as lexical, owner, load, object, inheritance, registration, lifecycle, state, or call. WoW does not have one universal parent relation.

## Recognizer

A deterministic declarative pattern over normalized Lua/TOC/XML facts that emits universal roles and relations.

## Core recognizer pack

A repository-owned immutable pack of universal E2-B recognizer rules eligible for the reviewed core execution profile. Its activation, versions, fixtures, and producer partitions are exact and independently validated.

## Calibration pack

A named set of declarative recognizers and fixtures derived from structural conventions observed in a corpus. The name is audit metadata; the rules emit universal roles and relations and cannot branch on repository, addon, owner, path, popularity, split, label, reviewer, search, or model identity.

## Calibration candidate source

An exact repository revision plus audit metadata proposed for a calibration corpus. A commit pin alone is not admission: exact tree/source inventory, owner publications/facts, provenance grouping, license/privacy decisions, independent labels, and split eligibility remain separate gates.

## Admitted calibration corpus

An immutable manifest of candidate sources/examples that passed the claimed materialization, fact-publication, provenance, license/privacy, label, coverage, and split gates. Quarantined or partially pinned sources are not admitted members.

## Calibration corpus example

One exact bounded fact/source-evidence unit in an admitted corpus, linked to an independent expected label set, provenance group, structural-shape groups, capability/coverage state, mutation family, and split assignment.

## Calibration expected label set

An immutable independent statement of expected universal entity/relation outputs, confidence ceiling, ambiguity/cardinality, decisive evidence, and label class: `Positive`, `Negative`, `Possible`, `NotEvaluated`, `Unknown`, or `Conflict`. It is not copied from candidate-pack output.

## Calibration provenance group

A conservative connected group of examples related by upstream/fork history, copied or vendored code, generated templates, near-duplicate structure, shared authoring lineage, mutation ancestry, or another reviewed dependence. The strongest applicable closure is the atomic unit for split independence.

## Calibration split manifest

An immutable assignment of whole provenance groups to `Train`, `Dev`, `Test`, `SealedHoldout`, `Challenge`, or `Quarantine`, together with visibility rules, leakage analysis, consumed-generation history, and canonical identity.

## Sealed holdout

An evaluation split whose exact membership digest is frozen while labels/results remain inaccessible to pack authors until the candidate pack bytes, implementation/profile identities, and run request are frozen. Once results influence a change, that holdout generation is consumed and no longer untouched evidence for the changed candidate.

## Calibration pack candidate

An exact `trust_class = calibration`, `rollout_state = shadow_only` E5-A pack bound to frozen corpus/split/mutation/evaluation profiles. It uses the E2-B operator language and produces only registered universal graph proposals at `Derived` or `Possible` confidence.

## Calibration shadow output partition

An exact pack/rule/version/input/profile-owned evaluation partition containing matches, proposals, evidence, coverage, conflicts, graph-validation receipts, and resource state. It is not a production GraphSnapshot and cannot satisfy default core coverage.

## ShadowValidated

An E5-A evaluation state indicating that the frozen candidate completed the applicable shadow hard gates and reports. It is not reviewer authorization, a promotion submission, core activation, publication, or runtime proof.

## PromotionEligibleByMetrics

An E5-A state indicating that frozen metric and hard-gate criteria passed for the exact candidate generation. It does not authorize promotion; E5-B review/authorization and E5-C immutable publication remain required.

## Calibration candidate artifact

An immutable E5-A bundle containing exact pack/corpus/split/run identities, per-case and metric reports, mutation/anti-overfitting and graph-validation evidence, license/provenance records, blockers/nonclaims, and a deactivation plan. It is input to future E5-B review, not a promoted core pack.

## Calibration deactivation plan

An exact plan proving that disabling, rejecting, quarantining, or superseding a candidate removes only its owned shadow partitions and stale references, preserves core/foreign partitions and historical evidence, and reports the precise coverage downgrade.

## Promotion submission

A future E5-B immutable review artifact that binds an exact E5-A candidate artifact, reviewer authorization, sealed-holdout access audit, retained evidence, and decision state. It requests consideration for publication but is not itself a published core pack.

## Holdout access audit

A future E5-B record of who or what was authorized to unseal exact holdout material, under which scope, candidate/run identity, time/revocation/replay constraints, and result-retention policy. Authorization does not alter labels, metrics, or graph proof.

## Lineage

An evidence-backed explicit relationship between entities from exact different generations. Generation-local identities remain distinct; lineage does not by itself imply deprecation, replacement, edit compatibility, migration success, or runtime behavior.

## Skeleton L0

Signature, role, chains, direct effects, and neighborhood counts.

## Skeleton L1

Collapsed control-flow and effect structure: branches, loops, calls, guards, returns, and state effects.

## Skeleton L2

Exact source span or full source.

## Project Map

A generated compact architecture summary for one project generation, intended to fit in routine agent context.

## Restriction facet

A versioned metadata unit describing Secret, protected, combat, hardware-event, forbidden-object, private-partition, or related runtime constraints.

## Secret Value

A World of Warcraft runtime value whose accessibility or permitted operations are restricted by the game. Static nominal Secret types are analysis projections, not runtime wrapper claims.

## Differential oracle

An external implementation used to compare expected output or behavior. It can reveal disagreement but does not automatically override canonical source.

## Candidate

An item selected for investigation by fuzzy, text, semantic, or external implementation similarity. It is not a proven relation or replacement.

## Root-cause folding

Deterministic grouping of downstream diagnostics under a known causal failure while preserving raw findings for inspection.

## Universe

A separated source domain: reference, first-party workspace, declared dependency, external example, or installed runtime data.
