# Tests and evaluation

This directory will hold cross-crate fixtures, golden outputs, integration tests, compatibility probes, security cases, and agent task evaluations.

Planned layout:

```text
fixtures/
    apidoc/
    annotations/
    toc/
    xml/
    lua/
    restrictions/
    lineage/
    malicious/

golden/
    findings/
    manifests/
    skeletons/
    search/
    impact/

compat/
    emmy/
    ketho/
    numy/
    schemas/

eval/
    tasks/
    expected/
    reports/
```

Fixtures are pinned, small, independently understandable, and licensed for repository use. Golden output is canonicalized and deterministic. Runtime WoW observations are stored only as structured external evidence with exact build/scenario identity.

See `docs/TEST_STRATEGY.md`.
