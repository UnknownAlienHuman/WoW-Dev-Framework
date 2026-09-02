# E5-C CLI command grammar

**Status:** normative.

Common explicit options:

```text
--config <PATH>
--request <PATH|->
--output-mode <envelope-json|text|artifact>
--output <PATH|->
--operation-id <ID>
--expect-digest <SHA256>
--consumer-profile <ID>
--budget-profile <ID>
```

Exact selectors include `--submission`, `--artifact`, `--signature`, `--publication`, `--canary-plan`, `--canary-assignment`, `--observation`, `--rollout-plan`, `--rollout-stage`, `--execution-profile`, `--current-record`, `--expect-current-digest`, `--lkg-designation`, `--rollback-target`, `--revocation`, and `--closure-report`.

Command groups map mechanically to service operations:

```text
status -> core_pack_status
submission validate -> core_pack_submission_validate
artifact build|validate -> core_pack_artifact_build|core_pack_artifact_validate
sign request|validate -> core_pack_sign_request|core_pack_signature_validate
publication publish|get|list|validate -> corresponding core_pack_publication_* operation
canary plan|start|status|observe|evaluate -> corresponding core_pack_canary_* operation
rollout plan|advance|pause -> corresponding core_pack_rollout_* operation
activation get|activate -> core_pack_activation_get|core_pack_activate
lkg get|designate -> core_pack_last_known_good_get|core_pack_last_known_good_designate
rollback -> core_pack_rollback
revoke -> core_pack_revoke
deactivate -> core_pack_deactivate
partition-closure validate -> core_pack_partition_closure_validate
```

No `--latest`, `--best`, `--previous`, `--default`, `--auto`, `--force-without-cas`, private-key flag, generic tool call, source path, or public-release command is valid.

At most one stdin consumer. Config/input is strict JSON with no include, interpolation, environment expansion, script, plugin, archive extraction, or implicit cwd/home/Git/editor/WoW discovery.