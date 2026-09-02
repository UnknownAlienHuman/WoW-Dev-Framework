# E6-A unverified locators and E6-B mapping handoff

**Status:** normative boundary.

## Unverified locator

A provider may return repository/root/revision/path/URI/symbol/span/digest fields. E6-A stores them as `UnverifiedProviderLocator` with exact raw-field origins, normalization/loss state, and `verification_state=Unverified`.

It is not:

```text
StableSourceHandle
ProjectEntityId
ReferenceEntityId
GraphEntityKey
verified repository revision
verified filesystem path
verified source span
```

## E6-A prohibitions

The crate does not:

- open a path or URI;
- clone/fetch a repository;
- read a source file;
- inspect cwd/home/editor/Git state;
- compare provider bytes to owner bytes;
- search same-name project/reference entities;
- choose one of multiple possible mappings;
- create graph relations from a locator;
- convert a snippet into source evidence;
- follow provider links or execute source.

## Handoff record

E6-A may emit:

```text
ExternalLocatorMappingRequestCandidate
    external candidate/result/provider/state IDs
    one or more UnverifiedProviderLocator IDs
    exact raw/normalized fields and digests
    requested owner universe kinds
    privacy/license constraints
    Candidate authority and nonclaims
```

This is only an input candidate for E6-B orchestration.

## E6-B owner ports

E6-B may acquire exact retained project/reference views and submit the handoff to narrow ports such as:

```text
ProjectExternalLocatorMappingPort
ReferenceExternalLocatorMappingPort
```

Owners validate exact repository/profile/generation/content/path/symbol/span identity and return one of:

```text
ExactMapped
MultipleMappings
NoMappingWithOwnerAuthority
NoMappingPartial
Conflict
NotEvaluated
Failed
```

`ExactMapped` proves only that the locator maps to one exact owner record under the mapping profile. It does not verify provider summaries, relationships, rank, or interpretation.

## Selection and context

After exact mapping, E6-B still requires an explicit candidate selection receipt before context handoff. Top-1/sole/high-score/same-name is never selected automatically.

The context owner receives the exact mapped project/reference root and mapping/selection audit separately. Provider snippets/summaries do not enter `ContextSemanticPack` as framework facts.

## Privacy/license

Mapping may reveal private repository/path information. The request and response bind consumer, source-owner, privacy, license, and disclosure profiles. E6-A locator artifacts expose only fields permitted by the external result profile; E6-B may further narrow.

## Mapping nonclaims

Even exact mapping does not establish API contract, runtime behavior, lineage, replacement, migration safety, impact, or correctness of the provider candidate.