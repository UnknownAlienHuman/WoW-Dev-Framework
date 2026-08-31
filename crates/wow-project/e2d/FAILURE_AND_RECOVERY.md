# E2-D failure and recovery

**Status:** normative.

## Failure state table

### Before store transaction

```text
target store generation: absent
current head: unchanged
recovery: fix request/plan
```

### Transaction rolled back

```text
target: uncommitted staging
current head: unchanged
recovery: cleanup/quarantine
```

### Commit completed, seal failed

```text
target: committed staging, not published
current head: unchanged
recovery: classify; usually rebuild, never edit sealed current
```

### Sealed/open validation failed

```text
target: sealed invalid/quarantined or inactive
current head: unchanged
recovery: investigate/rebuild
```

### Domain post-open validation failed

```text
target: physically valid but semantically inactive
current head: unchanged
recovery: correct producer/plan/contract and rebuild unless exact nonmutable revalidation can prove a transient validator defect
```

### CAS conflict

```text
target: validated inactive
current head: another head
recovery: no adoption against stale base; retain/pin/GC
```

### Crash/ambiguity during CAS

Read registry:

- head equals target: Published;
- head equals expected old: InactiveValidated/RetryCAS only after exact revalidation;
- another head: Conflict;
- invalid registry: manual recovery.

No blind retry.

## Sealed inactive adoption

Project recovery operation receives:

```text
exact inactive generation/artifact
original request/candidate/publication bundle
fresh current head
fresh project/graph/store open validation
expected-base proof
```

Adoption is allowed only if current head still equals original expected head and all exact identities/gates pass. It constructs the same snapshot/head IDs and performs fresh CAS.

## Last-known-good

Failure result can include:

```text
current_head
last_known_good_head
failed_target_candidate
inactive_target_generation
```

Each keeps original identity. Status never says the last-known-good is current for the failed target unless it actually remains current head.

## Cancellation

- before CAS: current unchanged; target state classified;
- during ambiguous CAS: resolve registry;
- after CAS success: published result wins; cancellation is late;
- no background continuation;
- no completion envelope marked cancelled if target is current.

## Recovery and capability state

Physical recovery cannot upgrade project/graph capability coverage. Partial/conflicted candidate remains partial/conflicted after recovery.

## Retention interaction

Inactive target may be rooted during investigation/recovery. Once no recovery/evidence root remains, normal store GC may collect it. Project does not delete files directly.

## Fatal current corruption

If current headed generation fails integrity:

- report critical typed failure;
- do not silently switch head to LKG;
- optionally expose explicit exact LKG selection through higher policy;
- require reviewed registry recovery/CAS;
- preserve corrupt artifact for investigation unless safety policy requires quarantine.
