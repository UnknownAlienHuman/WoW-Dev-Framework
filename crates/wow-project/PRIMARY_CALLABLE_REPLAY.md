# Ordinary Primary callable replay

Project replay retains exact Main and Library files plus the selected declaration,
signature, hook, witness and native-input configuration. Every current replay uses
the same `ProjectReplay` DTO and `wow-project.live-replay` store record.

Feature choices are not encoded in a schema name. Hydration rebuilds
`ProjectConfiguration`, Emmy analysis and graph owners, verifies the captured
configuration and semantic identities, then compares the complete recaptured DTO.
A different selection is rejected by those owners.

Old derived records are rebuilt from source and configuration. There is no tag
ladder, compatibility reader or migration path. These source observations do not
claim runtime behavior. Broader context work is tracked by
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
