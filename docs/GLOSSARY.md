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

## Calibration pack

A named set of recognizers and fixtures derived from structural conventions observed in a framework/addon corpus. It does not create repository-specific product behavior.

## Lineage

Evidence-backed identity or migration relationship between entities across builds, including moves, replacements, removals, and semantically surviving symbols.

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
