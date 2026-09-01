# E5-A example artifacts

These files are closed documentation fixtures for the calibration-corpus and named-pack contract. They define shapes and failure behavior; they do not claim that a recognizer implementation, real admitted corpus, sealed holdout, reviewer authorization workflow, benchmark, or promotion exists.

| File | Purpose |
|---|---|
| `candidate-sources.json` | Records eight exact user-repository revision pins as candidate inputs with all remaining admission gates explicit. |
| `corpus-manifest.json` | Defines a closed synthetic corpus used only to exercise identities, examples, coverage, and nonclaims. |
| `label-sets.json` | Covers Positive, Negative, Possible, NotEvaluated, and Conflict label shapes with synthetic independent-review records. |
| `split-manifest.json` | Demonstrates atomic provenance-group assignment, Challenge/Quarantine, leakage state, and an explicitly unconfigured sealed holdout. |
| `mutation-suite.json` | Defines metadata/path/name invariance plus decisive literal/join/resolution/coverage sensitivity and resource mutations. |
| `calibration-pack.json` | Defines one synthetic `trust_class=calibration`, `shadow_only` pack over the E2-B operator language and registered universal `module` output. |
| `run-and-results.json` | Demonstrates per-case-first run/metric artifacts while all executable results remain honestly `NotEvaluated`/blocked. |
| `candidate-artifact.json` | Demonstrates candidate/deactivation shape and why it is not a promotion submission or core pack. |
| `CHECKSUMS.json` | Holds prerequisite/profile/member/bundle freeze gates; all implementation-dependent values remain null before Rust implementation. |

## Fixture rules

- Real repository pins are audit identities only and are not admitted sources.
- The synthetic module-factory convention is not a real donor/framework claim.
- No raw repository source, private material, credentials, holdout secrets, or executable content is included.
- Tests verify committed fixtures; they never rewrite expected bytes automatically.
- Missing executable/profile/benchmark/checksum values are blocking or `NotEvaluated`, never pass.
