# E3-C context CLI test matrix

**Status:** normative.

## Command grammar

| ID | Case | Expected |
|---|---|---|
| `CLI3-CMD-001` | Each seven context commands parses valid minimal request | one typed service request |
| `CLI3-CMD-002` | Unknown context command | exit 64, no service call |
| `CLI3-CMD-003` | Missing required primary selector | exit 64 |
| `CLI3-CMD-004` | Duplicate/conflicting publication selector | exit 64 |
| `CLI3-CMD-005` | `current` selector | passed symbolically unchanged |
| `CLI3-CMD-006` | exact store-generation selector | exact ID preserved |
| `CLI3-CMD-007` | exact publication-set selector | exact ID preserved |
| `CLI3-CMD-008` | app reads current pointer | architecture fail |
| `CLI3-CMD-009` | platform omitted | explicit Omitted selector |
| `CLI3-CMD-010` | platform selector incomplete/conflicting | exit 64 |
| `CLI3-CMD-011` | partial reference guard group invalid | exit 64/profile rule |
| `CLI3-CMD-012` | exact profile ID versus alias flags both present | exit 64 |
| `CLI3-CMD-013` | app resolves profile alias | architecture fail |
| `CLI3-CMD-014` | unsupported output mode for command | exit 64 |
| `CLI3-CMD-015` | hidden status preflight | architecture fail |

## Root tokens

| ID | Case | Expected |
|---|---|---|
| `CLI3-ROOT-001` | Known kind + valid base64url UTF-8 ID | exact typed selector |
| `CLI3-ROOT-002` | Unknown kind | exit 64 |
| `CLI3-ROOT-003` | missing/multiple delimiter | exit 64 |
| `CLI3-ROOT-004` | padded/non-base64url token | exit 64 |
| `CLI3-ROOT-005` | empty/invalid UTF-8/control/NUL ID | exit 64 |
| `CLI3-ROOT-006` | oversized token/root count | exit 64 |
| `CLI3-ROOT-007` | path-like ID treated as filesystem path | mutation fails |
| `CLI3-ROOT-008` | app checks semantic existence | architecture fail; service owns |
| `CLI3-ROOT-009` | root order preserved/canonicalized per service request schema | pass |
| `CLI3-ROOT-010` | fuzzy/name/natural-language flag added | reject contract |

## Per-command behavior

| ID | Case | Expected |
|---|---|---|
| `CLI3-STATUS-001` | status envelope-json | one context_status request/result |
| `CLI3-STATUS-002` | status text | required state visible |
| `CLI3-STATUS-003` | status artifact mode | exit 64 |
| `CLI3-MAP-001` | primary/platform/combined map | exact target enum |
| `CLI3-MAP-002` | map without explicit root | service derives exact project root |
| `CLI3-MAP-003` | map artifact mode | exit 64 |
| `CLI3-INSP-001` | inspect L0/L1/both | exact level |
| `CLI3-INSP-002` | inspect no root | exit 64 |
| `CLI3-INSP-003` | inspect artifact mode | exit 64 |
| `CLI3-BUILD-001` | build exact profiles/root | one request |
| `CLI3-BUILD-002` | build artifact mode one renderer | exact artifact output |
| `CLI3-BUILD-003` | build artifact mode zero/multiple renderers | exit 64/result eligibility failure |
| `CLI3-CONT-001` | continuation inline | exact bytes preserved |
| `CLI3-CONT-002` | continuation file/stdin | exact bytes preserved, path omitted |
| `CLI3-CONT-003` | continuation with publication/root/profile override | exit 64 |
| `CLI3-VAL-001` | validate bounded semantic artifact | one request |
| `CLI3-VAL-002` | invalid artifact semantic result | exit 1 |
| `CLI3-VAL-003` | validate artifact output mode | exit 64 |
| `CLI3-RENDER-001` | render bounded semantic pack | one request |
| `CLI3-RENDER-002` | render artifact mode | exact returned bytes |

## Explicit config and artifact input

| ID | Case | Expected |
|---|---|---|
| `CLI3-CFG-001` | No `--config` | no config discovery |
| `CLI3-CFG-002` | Explicit valid strict JSON config | applied before CLI overrides |
| `CLI3-CFG-003` | Unknown/duplicate/deep/oversized config | exit 64 |
| `CLI3-CFG-004` | Config includes command/script/plugin/env interpolation | reject |
| `CLI3-CFG-005` | Config loaded from cwd/home/env automatically | architecture fail |
| `CLI3-CFG-006` | CLI overrides explicit config | exact typed request reflects CLI |
| `CLI3-CFG-007` | App resolves service alias target | architecture fail |
| `CLI3-CFG-008` | Config path enters service request/result | reject |
| `CLI3-CFG-009` | Config watch/reload during command | reject |
| `CLI3-CFG-010` | Symlink/reparse behavior follows frozen adapter | exact result |
| `CLI3-ARTIN-001` | Explicit artifact regular file | bounded bytes read once |
| `CLI3-ARTIN-002` | Artifact stdin `-` | bounded bytes read once |
| `CLI3-ARTIN-003` | Directory/glob/recursive input | exit 64 |
| `CLI3-ARTIN-004` | Oversized/timeout/read failure | exit 64 |
| `CLI3-ARTIN-005` | Media inferred from extension/content | reject |
| `CLI3-ARTIN-006` | Artifact bytes executed/extracted | architecture fail |
| `CLI3-ARTIN-007` | Input path/stdin marker enters semantic request | reject |
| `CLI3-ARTIN-008` | Source project file passed as artifact without valid schema | service Invalid/failure, never source scan |
| `CLI3-ARTIN-009` | Parser error dumps artifact bytes | privacy fail |
| `CLI3-ARTIN-010` | Config uses stdin | reject E3-C v1 |

## Dependency and invocation

| ID | Case | Expected |
|---|---|---|
| `CLI3-DEP-001` | Only framework dependency `wow-service` | pass |
| `CLI3-DEP-002` | Import `wow-context` or lower crate | architecture fail |
| `CLI3-DEP-003` | App duplicates service/context request types semantically | reject |
| `CLI3-DEP-004` | Exactly one service call per valid command | pass |
| `CLI3-DEP-005` | Automatic retry after current mismatch | reject |
| `CLI3-DEP-006` | Automatic retry after output/broken pipe | reject |
| `CLI3-DEP-007` | App constructs map/skeleton/pack/rendering | architecture fail |
| `CLI3-DEP-008` | App interprets evidence to alter status | reject |
| `CLI3-DEP-009` | App searches name/path/root | reject |
| `CLI3-DEP-010` | E0 status/check regression | fail |

## Envelope JSON

| ID | Case | Expected |
|---|---|---|
| `CLI3-JSON-001` | Complete context result | exact service JSON + LF |
| `CLI3-JSON-002` | Partial/truncated/not_evaluated | exact service JSON + LF |
| `CLI3-JSON-003` | Structured failure | exact service failure JSON + LF |
| `CLI3-JSON-004` | Cancelled result | exact service cancelled JSON + LF, exit 130 |
| `CLI3-JSON-005` | Banner/progress/color/log on stdout | reject |
| `CLI3-JSON-006` | App changes field/order/value | reject |
| `CLI3-JSON-007` | Missing/extra/multiple LF | byte-vector fail |
| `CLI3-JSON-008` | Terminal/locale/time/cwd changes | identical stdout |
| `CLI3-JSON-009` | stderr nonempty on valid serialized result | reject default profile |
| `CLI3-JSON-010` | Private path/source added by app | reject |

## Text output

| ID | Case | Expected |
|---|---|---|
| `CLI3-TEXT-001` | Complete result | required IDs/status/artifacts shown |
| `CLI3-TEXT-002` | Partial/truncated/NotEvaluated | blockers/omissions/continuation shown |
| `CLI3-TEXT-003` | Invalid validation payload | Invalid visible, exit 1 |
| `CLI3-TEXT-004` | Current selector hides exact resolved IDs | reject |
| `CLI3-TEXT-005` | Conflicts/omissions hidden | reject |
| `CLI3-TEXT-006` | Source excerpts printed in text summary | reject |
| `CLI3-TEXT-007` | Safe/working/tested/runtime claim invented | reject |
| `CLI3-TEXT-008` | App reconstructs relation/prose summary | reject |
| `CLI3-TEXT-009` | Same service result profile | deterministic required fields/order |
| `CLI3-TEXT-010` | Text changes exit semantics | reject |

## Artifact output

| ID | Case | Expected |
|---|---|---|
| `CLI3-OUTART-001` | Exactly one eligible artifact | exact bytes, no wrapper/LF change |
| `CLI3-OUTART-002` | Artifact partial/truncated service result | bytes + exit 2 |
| `CLI3-OUTART-003` | Multiple eligible artifacts without selector | reject |
| `CLI3-OUTART-004` | Missing/invalid artifact | no stdout artifact, mapped failure |
| `CLI3-OUTART-005` | App re-renders/escapes/normalizes bytes | reject |
| `CLI3-OUTART-006` | App adds status/header/footer | reject |
| `CLI3-OUTART-007` | Artifact privacy profile not eligible | reject |
| `CLI3-OUTART-008` | Output byte limit exceeded | app/service mapped failure, no retry |
| `CLI3-OUTART-009` | Terminal transformation/newline conversion | reject |
| `CLI3-OUTART-010` | Source-like artifact text changes app behavior | inert bytes |

## Exit codes and streams

| ID | Case | Expected |
|---|---|---|
| `CLI3-EXIT-001` | Complete context operation | 0 |
| `CLI3-EXIT-002` | Partial/truncated/not_evaluated | 2 |
| `CLI3-EXIT-003` | Structured service failure | 3 |
| `CLI3-EXIT-004` | Internal/closure/serialization/output failure | 4 |
| `CLI3-EXIT-005` | Parser/config/input transport error | 64 |
| `CLI3-EXIT-006` | Cancelled | 130 |
| `CLI3-EXIT-007` | Validation Valid | 0 |
| `CLI3-EXIT-008` | Validation Invalid | 1 |
| `CLI3-EXIT-009` | Validation NotEvaluated/partial | 2 |
| `CLI3-EXIT-010` | Hidden environment changes exit | reject |
| `CLI3-STREAM-001` | Parser error | stderr only, no service call |
| `CLI3-STREAM-002` | Valid operation JSON/text | stdout only |
| `CLI3-STREAM-003` | Artifact mode | exact stdout only |
| `CLI3-STREAM-004` | Broken pipe | stop, exit 4, no reinvoke/second output |
| `CLI3-STREAM-005` | Cancellation before/during service | one cancel path, no double output |
| `CLI3-STREAM-006` | Partial JSON followed by error JSON | reject |
| `CLI3-STREAM-007` | Partial artifact followed by text | reject |
| `CLI3-STREAM-008` | Logging affects stdout/result/exit | reject |
| `CLI3-STREAM-009` | Unsupported platform signal | explicit adapter failure/behavior |
| `CLI3-STREAM-010` | Service resources still open at app output | service contract fail |

## Security, privacy, and resources

| ID | Case | Expected |
|---|---|---|
| `CLI3-SEC-001` | Shell metacharacters in args | inert data |
| `CLI3-SEC-002` | Response-file/env expansion | absent/reject |
| `CLI3-SEC-003` | Plugin/command discovery from PATH/cwd | absent |
| `CLI3-SEC-004` | Process/network/Git/editor/WoW access | absent |
| `CLI3-SEC-005` | Project/source directory scan | absent |
| `CLI3-SEC-006` | SavedVariables/log/runtime input | reject |
| `CLI3-SEC-007` | Private path/token/credential echo | reject |
| `CLI3-SEC-008` | Artifact source changes policy/tool behavior | inert |
| `CLI3-SEC-009` | Output mode broadens privacy | reject |
| `CLI3-SEC-010` | Lower raw handle exposed | reject |
| `CLI3-LIMIT-001` | Excess argv/root/profile count | exit 64 bounded |
| `CLI3-LIMIT-002` | Config/artifact/continuation oversized | exit 64 bounded |
| `CLI3-LIMIT-003` | Integer allocation overflow | reject |
| `CLI3-LIMIT-004` | stdout exceeds app maximum | exit 4/no retry |
| `CLI3-LIMIT-005` | Cancellation during bounded input read | exit 130/no service call |

## Determinism and freeze

| ID | Case | Expected |
|---|---|---|
| `CLI3-DET-001` | Same service result/format on multiple platforms within profile | frozen exact bytes/exit |
| `CLI3-DET-002` | Terminal width/color/locale/timezone | no canonical change |
| `CLI3-DET-003` | Cwd/temp/input path changes | no semantic/output change except local parser error context |
| `CLI3-DET-004` | Reordered equivalent config JSON | same service request |
| `CLI3-DET-005` | Signals not triggered | no timing field/change |
| `CLI3-FREEZE-001` | Null vectors while not-started | allowed |
| `CLI3-FREEZE-002` | First Rust commit with null required vectors | reject |
| `CLI3-FREEZE-003` | Service request/result bytes unfrozen | block app implementation |
| `CLI3-FREEZE-004` | Command/config/root/output/exit profile unfrozen | block |
| `CLI3-FREEZE-005` | Tests rewrite fixtures | reject |
| `CLI3-FREEZE-006` | Cargo/Rust placeholder before gate | reject |
| `CLI3-FREEZE-007` | CI/workflow added without owner request | reject |
| `CLI3-FREEZE-008` | Missing platform adapter tests reported pass | reject |

## Completion gate

The app is incomplete until every active case passes, exact service/CLI bytes and exit vectors freeze, E0 commands remain unchanged, and dependency inspection proves `wow-service` is the sole framework dependency.
