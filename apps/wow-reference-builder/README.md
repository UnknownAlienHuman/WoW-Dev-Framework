# `wow-reference-builder`

Supported native Blizzard API annotation generator. It reads exact objects from
a local `wow-ui-source` Git revision, ingests APIDocumentation through
`wow-reference`, projects Ketho-compatible EmmyLua files through
`wow-annotations`, and writes an ordinary new directory.

```text
wow-reference-builder generate <checkout> <ref> <GeneratedApi.toc>   <environment> <new-output-directory>   [--corrections <reviewed.json>]   [--alias-catalog <checkout> <ref> <resource.lua>]...
```

The destination must not exist. Output is generated `.lua` files plus
`source-report.json`; exit `3` means useful partial output, `0` means complete
projection, `2` is usage, `5` is generation failure, and `6` is cancellation.
The command reads committed Git blobs and never executes donor Lua.

There is no pack, signing, publication journal, validation/rebuild-compare
lifecycle, installer, migration path or compatibility command. Generated output
is derived and may be rebuilt from source, corrections and alias catalogs.
