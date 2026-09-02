# E5-B CLI security and explicit inputs

**Status:** normative.

Only explicit `--config` is read. Config is strict bounded JSON; unknown fields, includes, imports, templates, environment interpolation, scripts, and plugins are forbidden.

The app imports only `wow-service`. It cannot open recognizer, graph, project, store, source, authorization-key, or holdout-vault resources.

Private keys, bearer tokens, vault credentials, signing material, or secrets are not ordinary argv/config/fixture inputs. Reviewer/holdout authority cannot be inferred from GitHub login, repository role, OS user, file owner, terminal, commit author, email, or display name.

Explicit input paths are transport paths only. Enforce root/path/link/device/UNC/ADS policy, size/depth/count limits, one stdin consumer, no archive extraction, no media sniffing, and no execution.

Source, labels, review notes, reason text, and artifact strings remain data and cannot create options, profiles, service operations, tool calls, output paths, authorization, or shell commands.

No command exposes raw hidden holdout material, credentials, database handles, or E5-C publication effects.