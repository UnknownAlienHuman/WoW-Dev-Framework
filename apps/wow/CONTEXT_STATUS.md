# Context status

`wow context status` requests one metadata-only status operation from
`wow-service`. It requires an explicit native host configuration and project
store registration. The app never opens the store, resolves Current, chooses a
Reference view, or retries a failed guard.

```text
wow context status --native-host-config <host.json>
  --project-store <registration-key>
  --publication current|store-generation:<ProjectStoreGenerationId>
  [--detail summary|capabilities|profiles|generations]
  [--platform omitted]
  [--expect-current-record <CurrentPublicationRecordId>]
  [--expect-current-digest <ContentDigest>]
  [--expect-profile <ProfileId>
   --expect-reference-generation <ReferenceGenerationId>
   --expect-reference-view <ReferenceViewId>]
  [--output envelope-json]
```

The default detail is `summary`. The CLI accepts `current` or an exact retained
store generation; it does not accept a publication-set selector on this route.
Current guards may be supplied only with `current`. A stale record or digest
fails in Service. Exact-generation selection stays on the named generation if
Current changes. The Reference guard is optional as a whole and otherwise
requires the complete exact profile, generation, and view tuple. This route
accepts only omitted platform selection.

`summary`, `capabilities`, `profiles`, and `generations` select closed detail
forms. The Service envelope reports exact publication/store, project, graph,
and Reference identity, coverage/conflict and deferred-capability summaries.
Capabilities and profiles describe the finite selected registry. Platform and
combined profiles report `requires_platform_selection`. Generations
detail includes project and graph snapshot/generation IDs, graph universe and
registry digest, source registry ID, and analyzer snapshot ID.

Successful status is always `partial`. The payload keeps context generation,
evaluation, last-known-good, and failed-target state `NotEvaluated`; continuation
retention is `not_implemented`; optional platform status is `not_selected`.
Those values distinguish available metadata from context generation or runtime
evaluation.

Status does not invoke map, skeleton, semantic-pack, source-excerpt, or renderer
operations. It returns no such artifact and no source body. Service publishes
the result only after checked closure of the retained read and store. Guard,
identity, budget, cancellation, and closure failures have no success payload.

## Exact platform candidate metadata

```text
wow context status --platform native --native-host-config <host.json>
  --platform-status-request-input <request.json> [--output envelope-json]
```

The strict request schema `wow-service/context-platform-status-request/1`
requires the registered `project_store_id`, exact `store_generation_id`,
`publication_set_id`, `candidate_id`, `candidate_digest`,
`profile_id=wow-context/platform-native-one-hop/1`, and one of the four `detail`
strings. The optional `reference` guard requires all three fields `profile`,
`generation`, and `view`. This route has no Current selector or root input.

Service reacquires the genuine selected namespace and retained candidate and
checks exact Project/Reference compatibility. The
`wow-service/context-platform-status-result/1` envelope exposes original
Project/Graph/Reference metadata and coverage; capabilities requiring Primary
selection remain explicit. It invokes no root, Map, skeleton, pack, or renderer.
Root coverage, generation, evaluation, last-known-good and failed targets remain
`NotEvaluated`; continuation retention remains `not_implemented`. Delivery waits
for checked read/store closure. Success remains `partial`.

Full E3-C acceptance remains pending.

## Composed Primary and platform candidate metadata

```text
wow context status --platform combined --native-host-config <combined-host.json>
  --combined-status-request-input <request.json> [--output envelope-json]
```

The strict request schema is
`wow-service/context-combined-status-request/1`. It selects an exact Primary
`project_store_id`, `store_generation_id`, and `publication_set_id`, plus an
exact platform `project_store_id`, `store_generation_id`, `publication_set_id`,
`candidate_id`, `candidate_digest`, and
`profile_id=wow-context/platform-native-one-hop/1`. `detail` is one of
`summary`, `capabilities`, `profiles`, or `generations`. The request has no
Current selector or root. The explicit combined host supplies the shared exact
Reference view and limits.

The single Service operation is
`execute_context_combined_status`; it reacquires both exact retained selections,
checks candidate and shared Reference compatibility, and returns separate
`primary` and `platform` metadata plus `shared_reference`. Success remains
`partial` after checked closure, with root coverage, context generation,
evaluation, last-known-good, and failed targets `NotEvaluated`; continuation
retention remains `not_implemented`. It produces no root result or context
artifact.

Service holds Primary while it acquires the exact platform candidate, then
checks and closes platform before Primary. Both configured Reference copies
must validate against one cumulative work budget; matching claimed digest fields
alone do not prove compatibility. The platform metadata includes the original
`native_project_store_id`; both owners expose their original Project/Graph
identities without constructing a context universe or artifact. Resolved identity
binds those owners, namespace, shared Reference schema/profile/generation/view
and limits. Full E3-C acceptance remains pending.
