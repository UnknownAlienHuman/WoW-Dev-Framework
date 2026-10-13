# Primary context status

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

This implementation covers Primary metadata only. Platform/composed status and
full E3-C acceptance remain pending.
