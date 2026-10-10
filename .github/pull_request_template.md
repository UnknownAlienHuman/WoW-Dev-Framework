## Canonical work item and ownership

<!-- Link the canonical Issue. Name the integration manager, exact base SHA, owned crates/modules and validation tier. A PR is a merge source, not a tracking-only ledger. -->

## Problem

<!-- What concrete failure, gap, or milestone item does this change address? -->

## Scope

<!-- What is included, and what is intentionally excluded? Distinguish hard blockers from acceptance debt. -->

## Contracts and decisions

<!-- List affected docs, schemas, profiles, registries, persistence formats, migrations, invariants or ADRs. Explain every new version. -->

## Profile and evidence

<!-- Exact WoW flavor/source selector resolved once to an exact revision when relevant. State evidence provenance, support, coverage, omissions and conflicts. -->

## Validation tier and exact head

```text
Tier S | Tier M | Tier A | Tier X
commit:
tree:
command — pass | fail | skipped | NotEvaluated
```

<!-- Tier S is affected-owner fmt/check/Clippy/xtask. Tier M is the lightweight remote exact-head workspace gate. Tier A/X are milestone acceptance/external/runtime gates. -->

## Compatibility and migration

<!-- Public/persisted format, database, profile, fixture or behavior impact. Identify the admitted consumer that requires compatibility. -->

## Known gaps

<!-- NotEvaluated capabilities, partial partitions, deferred acceptance, follow-up Issues or unresolved conflicts. -->

## Checklist

- [ ] This PR contains code intended to merge; it is not a stale specification/tracking branch.
- [ ] One canonical Issue owns remaining functional and acceptance work.
- [ ] The change preserves accepted architectural invariants or updates an ADR/contract deliberately.
- [ ] New schema/profile/storage versions correspond to real byte/identity/interpretation changes.
- [ ] The applicable validation tier ran on the exact recorded head; wider gates are not falsely claimed.
- [ ] Output is deterministic where required.
- [ ] Dynamic/partial/conflicted evidence is not presented as proven or authoritative absence.
- [ ] Documentation routing is updated once, without duplicating the same evidence ledger.
- [ ] Third-party provenance and license are recorded when applicable.
- [ ] A superseded branch/PR will be closed and reconciled after merge/publication.
