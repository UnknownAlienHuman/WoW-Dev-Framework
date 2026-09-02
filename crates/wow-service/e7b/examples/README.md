# E7-B release lifecycle fixture shapes

- `source-build.json` — exact source/lockfile/toolchain/materialization/build/reproducibility bindings.
- `evidence-bundle.json` — artifact self-description, SBOM, provenance, license/notices, signatures and deterministic bundle closure.
- `channel-manifest.json` — release candidate, provider-neutral publication, read-back, channel CAS and signed update manifest.
- `install-update-rollback.json` — exact installation, Windows replacement helper, migration, self-check, LKR, rollback, revocation and retirement states.
- `CHECKSUMS.json` — prerequisite, executor, signing, distribution, installer, platform, vector, benchmark, member and bundle freeze gate.

Implementation-dependent values remain null only while `implementation_state` is `not-started`. Fixtures are immutable inputs; tests verify rather than rewrite them.