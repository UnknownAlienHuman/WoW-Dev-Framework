# E4-A exact, alias, member, and prefix lanes

**Status:** normative.

## Common lane contract

Every lane receives:

```text
exact SearchShardView
NormalizedSearchQuery
lane profile and budget
allowed universe/entity kinds
confidence/provenance/coverage policy
cancellation
```

Every lane returns an ordered `SearchLaneResult` with exact candidate signals, field origins, coverage, conflicts, omissions, and execution state.

No lane mutates owner facts or another lane's output.

## Exact entity identity lane

### Input

```text
exact owner EntityKey/EntityId
allowed exact universe/shard
```

### Match

The key must match the owner entity identity exactly under the frozen owner schema. SearchDocument presence is validated against the owner record.

### Signal

```text
ExactEntityIdentity
```

This is the highest retrieval authority band, but still does not prove the caller intended that entity when the caller supplied a malformed or wrong exact key.

### Miss

An authoritative exact-identity miss requires:

- complete shard coverage for the key space;
- exact owner negative-authority support where applicable;
- no conflicting identity/index state;
- no lane or query truncation.

Otherwise return `ExactNotFoundPartial`.

## Exact canonical-name lane

Separate signals:

```text
ExactCanonicalQualifiedName
ExactCanonicalShortName
```

Rules:

- exact case-sensitive normalized equality;
- exact entity kind/universe filters;
- no suffix, substring, case-fold, or punctuation repair;
- multiple exact short-name matches remain multiple candidates;
- qualified name may outrank short name under the ranking profile;
- same canonical name in different universes remains distinct.

## Exact alias lane

An alias signal requires a stored explicit alias record with exact:

```text
alias value
target entity
profile/generation/scope
alias kind
source/evidence
confidence/provenance/coverage/conflict
```

Signals include:

```text
ExactExplicitAlias
ExactDeprecatedNameAlias
ExactOwnerCompatibilityAlias
```

Only alias kinds actually provided by the owner contract are permitted. A deprecated source name is not automatically a replacement relation.

Rules:

- alias value matching is case-sensitive unless the alias record/profile explicitly defines another exact comparison;
- unresolved/conflicting alias targets do not produce an exact target signal;
- alias loops or multiple exclusive targets are conflicts;
- search history, fuzzy spelling, comments, popularity, or external text never creates an alias.

## Namespace/member lane

Inputs can be:

```text
exact namespace + exact member
exact receiver entity/kind + exact method/member
exact package/module + exact exported member
```

The lane uses explicit owner/graph fields and relations. It does not split arbitrary user text and guess a receiver.

Signals:

```text
ExactNamespaceMember
ExactReceiverMethod
ExactExportedMember
```

A same-named method on an unrelated receiver remains a different candidate.

## Prefix lane

Prefix matching is candidate retrieval, not exact identity.

Initial signals:

```text
CaseSensitiveIdentifierPrefix
SegmentBoundaryPrefix
QualifiedNameComponentPrefix
```

Optional case-folded prefix is a lower approximate signal:

```text
FoldedIdentifierPrefix
```

Rules:

- prefix length has a frozen minimum;
- maximum expanded terms/candidates is bounded;
- no unbounded `*` query;
- no arbitrary substring/suffix lane;
- preserve the exact matched field and prefix range;
- stable order by field class, prefix specificity, canonical identifier, entity ID;
- a prefix signal never registers an alias.

## Coverage

Each exact/prefix index has independent coverage:

```text
owner entity enumeration
document projection
field projection
exact index build
alias target resolution
member relation resolution
prefix index build
lane query execution
```

Complete document count does not imply alias coverage. A prefix miss is never authoritative identity absence.

## Duplicates

The same entity retrieved by qualified name, short name, alias, member, and prefix becomes one `SearchCandidate` with distinct signals. Do not collapse the signal classes or discard their origins.

## Budgets

Bound:

- exact key/name/alias/member lengths;
- number of aliases per entity;
- ambiguous exact matches;
- prefix terms and expansions;
- result candidates and origin/evidence refs;
- serialized bytes, time, and memory.

If an exact lane exceeds an ambiguity/output limit, return explicit truncation/partial state; do not choose the first row.

## Security

- names and aliases are data, not FTS/SQL syntax;
- no source-controlled alias profile;
- no raw field/index/table names from callers;
- no private alias/documentation text in external result profiles;
- errors identify stable field/record IDs rather than source bodies.

## Determinism

Results are independent of row ID, insertion order, hash order, worker count, and storage layout. Ties end with exact universe/entity/document IDs.
