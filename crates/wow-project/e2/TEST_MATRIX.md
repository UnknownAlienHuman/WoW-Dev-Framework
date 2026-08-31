# E2-C project indexing test matrix

**Status:** normative executable acceptance and mutation matrix.

## Configuration/source/universes

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-CONFIG-001` | Valid exact E2-C request/profiles | accepted |
| `PROJECT-E2-CONFIG-002` | Dependency beyond core/emmy/graph/recognizers | architecture test fails |
| `PROJECT-E2-CONFIG-003` | Store dependency activated in E2-C | rejected |
| `PROJECT-E2-CONFIG-004` | Floating repo/branch/profile/variant | rejected |
| `PROJECT-E2-CONFIG-005` | Mixed profile/reference/analyzer/rule/graph generations | rejected |
| `PROJECT-E2-CONFIG-006` | Missing output-affecting profile from generation inputs | mutation fails |
| `PROJECT-E2-SOURCE-001` | Exact materialized first-party snapshot | accepted |
| `PROJECT-E2-SOURCE-002` | Same bytes under different host root/repository name | same semantic snapshot/candidate inputs |
| `PROJECT-E2-SOURCE-003` | Revision changes, content same | provenance changes; content semantics classified explicitly |
| `PROJECT-E2-SOURCE-004` | Content changes, revision same | new snapshot/generation |
| `PROJECT-E2-SOURCE-005` | Absolute/traversal/UNC/device/URI/token path | reject |
| `PROJECT-E2-SOURCE-006` | Case collision | reject deterministically |
| `PROJECT-E2-SOURCE-007` | Symlink/reparse/submodule default policy | unsupported/skipped with coverage |
| `PROJECT-E2-SOURCE-008` | Unexpected/unreadable TOC/XML/Lua | exact completeness impact |
| `PROJECT-E2-SOURCE-009` | Dependency source relabeled first-party | reject |
| `PROJECT-E2-SOURCE-010` | Analyzer library relabeled project source | reject |
| `PROJECT-E2-SOURCE-011` | Installed addon/SavedVariables/log/client root | unavailable/reject |
| `PROJECT-E2-SOURCE-012` | Repo hook/build/test/generator | never executed |
| `PROJECT-E2-SOURCE-013` | Prompt/tool instruction in source metadata | data only |
| `PROJECT-E2-SOURCE-014` | Source/file/byte budget exceeded | bounded failure/partial per policy |

## TOC parsing and variants

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-TOC-001` | Valid comments/metadata/file lines with spans | exact records |
| `PROJECT-E2-TOC-002` | LF/CRLF/BOM profile cases | canonical equivalent or explicit profile difference |
| `PROJECT-E2-TOC-003` | Known Interface/dependency/LOD/SavedVariables keys | exact normalized records |
| `PROJECT-E2-TOC-004` | Unknown/X directive | raw preserved; dependent coverage classified |
| `PROJECT-E2-TOC-005` | Malformed recoverable line | diagnostic/raw record, no invented semantics |
| `PROJECT-E2-TOC-006` | Multiple compatible variants | explicit ambiguity/reject |
| `PROJECT-E2-TOC-007` | No compatible variant | failure/partial by policy |
| `PROJECT-E2-TOC-008` | Mainline/Classic variant union | prohibited |
| `PROJECT-E2-TOC-009` | Unselected variant fills missing file/directive | mutation fails |
| `PROJECT-E2-TOC-010` | File source order | preserved exactly |
| `PROJECT-E2-TOC-011` | Filesystem alphabetical order substituted | mutation fails |
| `PROJECT-E2-TOC-012` | Missing file entry target | exact load/files blocker |
| `PROJECT-E2-TOC-013` | Duplicate file entry | retained/classified |
| `PROJECT-E2-TOC-014` | Entry traversal/absolute/URI | reject |
| `PROJECT-E2-TOC-015` | Known Bootstrap tag | bootstrap role only |
| `PROJECT-E2-TOC-016` | Unknown suffix/tag | preserved; phase capability partial |
| `PROJECT-E2-TOC-017` | LOD true/false/unknown | exact tri-state |
| `PROJECT-E2-TOC-018` | LOD/bootstrap implies full addon/frame readiness | mutation fails |
| `PROJECT-E2-TOC-019` | Required dependency resolved | exact edge |
| `PROJECT-E2-TOC-020` | Required dependency missing | incomplete/fail |
| `PROJECT-E2-TOC-021` | Optional dependency missing | explicit optional unresolved |
| `PROJECT-E2-TOC-022` | Dependency auto-download/discovery | prohibited |
| `PROJECT-E2-TOC-023` | SavedVariables account declaration | exact root seed |
| `PROJECT-E2-TOC-024` | SavedVariablesPerCharacter | distinct scope |
| `PROJECT-E2-TOC-025` | Duplicate/conflicting variable declarations | retained/conflict |
| `PROJECT-E2-TOC-026` | Same Lua global, no TOC declaration | no persistent root |
| `PROJECT-E2-TOC-027` | TOC budget truncation | Partial, never complete |
| `PROJECT-E2-TOC-028` | Shuffled parser scheduling | same records/digest |

## XML parsing and Lua units

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-XML-001` | Valid document/template/object/parent | exact records |
| `PROJECT-E2-XML-002` | Exact inheritance/template refs | exact ordered records |
| `PROJECT-E2-XML-003` | Multiple/unresolved inheritance | ambiguity/Possible |
| `PROJECT-E2-XML-004` | External XML include | resolved bounded edge |
| `PROJECT-E2-XML-005` | External Lua Script file | Lua unit + load edge |
| `PROJECT-E2-XML-006` | Inline script body | virtual Lua unit/source map |
| `PROJECT-E2-XML-007` | Inline analyzer span maps to XML bytes | pass |
| `PROJECT-E2-XML-008` | Inline wrapper changes semantics | reject/unreported loss fails |
| `PROJECT-E2-XML-009` | Unknown element/attribute | raw preserved, narrow gap |
| `PROJECT-E2-XML-010` | Duplicate object/template identity | conflict/reject by schema |
| `PROJECT-E2-XML-011` | Anonymous objects | deterministic scoped identity |
| `PROJECT-E2-XML-012` | Object parent confused with inheritance | mutation fails |
| `PROJECT-E2-XML-013` | Missing include/script | exact blocker |
| `PROJECT-E2-XML-014` | Include cycle | bounded conflict/failure |
| `PROJECT-E2-XML-015` | Include depth/fanout budget | Partial/fail |
| `PROJECT-E2-XML-016` | DTD/external entity/network/XInclude | reject/no access |
| `PROJECT-E2-XML-017` | Entity expansion/deep node/text bomb | bounded failure |
| `PROJECT-E2-XML-018` | Processing instruction/extension | unsupported data, no execution |
| `PROJECT-E2-XML-019` | XML or script execution attempt | never executed |
| `PROJECT-E2-XML-020` | Path traversal/absolute/URI in include | reject |
| `PROJECT-E2-XML-021` | Streaming chunk/worker variation | identical records/units |
| `PROJECT-E2-XML-022` | Failed embedded script with valid templates | template facts retained; script capability partial |

## Load model

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-LOAD-001` | One package exact TOC order | exact units/edges |
| `PROJECT-E2-LOAD-002` | Required dependency chain | ordered package graph |
| `PROJECT-E2-LOAD-003` | Optional dependency conditional | conditional reachability |
| `PROJECT-E2-LOAD-004` | Dependency cycle | conflict/no arbitrary break |
| `PROJECT-E2-LOAD-005` | Nested XML include/script expansion | exact reason path/order |
| `PROJECT-E2-LOAD-006` | Repeated include | frozen profile semantics |
| `PROJECT-E2-LOAD-007` | Reachable selected file | Reachable |
| `PROJECT-E2-LOAD-008` | Source inventory file not referenced | Unreachable/separate scope |
| `PROJECT-E2-LOAD-009` | Optional/unknown condition | ConditionallyReachable |
| `PROJECT-E2-LOAD-010` | Mandatory parser gap | Unknown/NotEvaluated |
| `PROJECT-E2-LOAD-011` | Materialize all transitive order edges | mutation fails |
| `PROJECT-E2-LOAD-012` | Bootstrap unit implies full UI state | mutation fails |
| `PROJECT-E2-LOAD-013` | ADDON_LOADED implies child frames ready | mutation fails |
| `PROJECT-E2-LOAD-014` | Static order claimed runtime success | prohibited |
| `PROJECT-E2-LOAD-015` | Selected variant isolation | pass |
| `PROJECT-E2-LOAD-016` | Load graph budget/truncation | explicit Partial |
| `PROJECT-E2-LOAD-017` | Explain direct/transitive load path | exact evidence path |
| `PROJECT-E2-LOAD-018` | 1/2/N deterministic load manifest | pass |

## Analyzer and virtual units

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-AN-001` | Exact physical/virtual Main unit manifest | analyzer plan valid |
| `PROJECT-E2-AN-002` | Exact annotation Library role | separate valid binding |
| `PROJECT-E2-AN-003` | Nonloaded test file in Main by default | excluded/separate explicit scope |
| `PROJECT-E2-AN-004` | Duplicate physical/virtual unit ID | reject |
| `PROJECT-E2-AN-005` | Analyzer project/profile/reference mismatch | reject candidate |
| `PROJECT-E2-AN-006` | Analyzer pin/config/library mismatch | reject |
| `PROJECT-E2-AN-007` | Extra/missing/wrong-digest Main unit | reject |
| `PROJECT-E2-AN-008` | XML inline source-map mismatch | reject |
| `PROJECT-E2-AN-009` | Removed unit facts/findings retained | reject |
| `PROJECT-E2-AN-010` | Project reparses Lua | architecture test fails |
| `PROJECT-E2-AN-011` | Project rewrites facts/findings | prohibited |
| `PROJECT-E2-AN-012` | Per-file analyzer failure | exact partial/fail policy |
| `PROJECT-E2-AN-013` | Cancellation | no complete target candidate |
| `PROJECT-E2-AN-014` | Analyzer fact order shuffled | candidate canonical output stable |

## Fact adapters and recognizers

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-ADAPT-001` | Every active E2-B fact kind | exact adapter mapping |
| `PROJECT-E2-ADAPT-002` | Source IDs/evidence/generation preserved | pass |
| `PROJECT-E2-ADAPT-003` | Unknown source field silently dropped | fail |
| `PROJECT-E2-ADAPT-004` | Raw source/AST passed | reject |
| `PROJECT-E2-ADAPT-005` | Main/Library/dependency roles mix | reject |
| `PROJECT-E2-ADAPT-006` | Cross-partition bundle undeclared scope | reject |
| `PROJECT-E2-ADAPT-007` | Empty partial facts treated complete | fail |
| `PROJECT-E2-ADAPT-008` | Path/repository name creates module role | mutation fails |
| `PROJECT-E2-RECOG-001` | TOC/XML/frame/event/hook/library/state bundles | exact applicable outcomes |
| `PROJECT-E2-RECOG-002` | Native/custom/CVar signal classes | distinct |
| `PROJECT-E2-RECOG-003` | Custom callback without TriggerEvent producer | unresolved/Possible, not confirmed |
| `PROJECT-E2-RECOG-004` | Hook output claims safety | reject |
| `PROJECT-E2-RECOG-005` | State root without TOC | no match/reject proposal |
| `PROJECT-E2-RECOG-006` | Partial/truncated recognizer output | not complete |
| `PROJECT-E2-RECOG-007` | Project mutates rule for fixture | prohibited |
| `PROJECT-E2-RECOG-008` | Rule version/disable | target partition plan exact |
| `PROJECT-E2-RECOG-009` | Recognizer cancellation/failure | isolated output partition |
| `PROJECT-E2-RECOG-010` | Shuffled bundle/worker order | identical outputs |

## Graph proposals

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-GRAPH-001` | Valid project direct proposals | graph accepts |
| `PROJECT-E2-GRAPH-002` | Valid recognizer proposals | graph accepts |
| `PROJECT-E2-GRAPH-003` | Undeclared kind/endpoint/key/attribute | rejection visible |
| `PROJECT-E2-GRAPH-004` | Rejected proposal omitted | candidate invalid |
| `PROJECT-E2-GRAPH-005` | Project weakens graph schema to accept | prohibited |
| `PROJECT-E2-GRAPH-006` | Project creates recognizer-owned role directly | reject |
| `PROJECT-E2-GRAPH-007` | Direct and recognizer producer partitions distinct | pass |
| `PROJECT-E2-GRAPH-008` | Final GraphGeneration/current publication in E2-C | prohibited |
| `PROJECT-E2-GRAPH-009` | Graph conflict affects candidate capability | explicit |
| `PROJECT-E2-GRAPH-010` | Graph validation profile changes | affected proposals revalidated |

## Invalidation and stale removal

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-INV-001` | Lua-only content edit | analyzer + dependent adapter/recognizer/proposals only |
| `PROJECT-E2-INV-002` | TOC file order edit | package/load/dependent partitions invalidated |
| `PROJECT-E2-INV-003` | TOC variant selection change | all old selected-variant active partitions stale |
| `PROJECT-E2-INV-004` | Dependency/LOD/bootstrap edit | exact load/dependent invalidation |
| `PROJECT-E2-INV-005` | SavedVariables add/remove | state root/path outputs updated |
| `PROJECT-E2-INV-006` | XML object/template edit | XML/recognizer/proposals invalidated |
| `PROJECT-E2-INV-007` | XML include/script/inline edit | expansion/Lua/analyzer/downstream invalidated |
| `PROJECT-E2-INV-008` | File remove | complete stale output closure |
| `PROJECT-E2-INV-009` | File add | visible only in target candidate |
| `PROJECT-E2-INV-010` | Analyzer pin/config change | all Lua-dependent partitions invalidated |
| `PROJECT-E2-INV-011` | Recognizer pack/rule change | rule output/proposals only where adapter stable |
| `PROJECT-E2-INV-012` | Graph registry change | proposal revalidation; parser reuse allowed if exact |
| `PROJECT-E2-INV-013` | Unknown impact reused narrowly | mutation fails |
| `PROJECT-E2-INV-014` | Unknown impact widened conservatively | pass |
| `PROJECT-E2-INV-015` | Mtime/path equality used as reuse proof | reject |
| `PROJECT-E2-INV-016` | Old records relabeled target generation | reject |
| `PROJECT-E2-INV-017` | Removed source/fact/match/proposal retained | reject |
| `PROJECT-E2-INV-018` | Same final state, different update order | same generation/candidate |
| `PROJECT-E2-INV-019` | NoChange | no expensive work/new generation |
| `PROJECT-E2-INV-020` | Cancel/fail target | base candidate unchanged |

## Candidate/publication boundary

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-CAND-001` | Complete candidate | valid NotPublishedE2C |
| `PROJECT-E2-CAND-002` | Permitted partial candidate | exact PartialCandidate |
| `PROJECT-E2-CAND-003` | Mandatory failure hidden | reject |
| `PROJECT-E2-CAND-004` | Mixed generations/manifests | reject |
| `PROJECT-E2-CAND-005` | Candidate mutated after validation | reject |
| `PROJECT-E2-CAND-006` | Persistent current pointer/store/GraphGeneration field | architecture fail |
| `PROJECT-E2-CAND-007` | Publication bundle exact logical manifests | pass |
| `PROJECT-E2-CAND-008` | Raw SQL/SQLite/WAL/connection in bundle | reject |
| `PROJECT-E2-CAND-009` | Prior candidate relabeled target | reject |
| `PROJECT-E2-CAND-010` | Physical future store model changes | E2-C candidate digest unchanged |

## Security/determinism/freeze

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-SEC-001` | Source/TOC/XML/Lua/hook execution | absent |
| `PROJECT-E2-SEC-002` | Dependency fetch/network | absent |
| `PROJECT-E2-SEC-003` | Process/shell/editor/client access | absent |
| `PROJECT-E2-SEC-004` | Private path/token/raw payload leak | fail |
| `PROJECT-E2-SEC-005` | Huge/fanout/path/entity bombs | bounded |
| `PROJECT-E2-SEC-006` | Cancellation every stage | bounded stop/no complete candidate |
| `PROJECT-E2-DET-001` | 1/2/N workers | identical canonical candidate |
| `PROJECT-E2-DET-002` | File/fact/parser/recognizer order shuffle | identical output |
| `PROJECT-E2-DET-003` | Temp root/clock/row/process ID | excluded |
| `PROJECT-E2-DET-004` | Same final state different updates | identical generation/candidate |
| `PROJECT-E2-FIX-001` | Null pins while not-started | allowed |
| `PROJECT-E2-FIX-002` | First Rust commit with required nulls | fail |
| `PROJECT-E2-FIX-003` | Fixture bytes change without checksums | fail |
| `PROJECT-E2-FIX-004` | Synthetic + pinned real addon fixture frozen | pass |
| `PROJECT-E2-FIX-005` | All vectors/member/bundle SHA-256 valid | pass |

## Deferred

| ID | Case | Expected |
|---|---|---|
| `PROJECT-E2-DEFER-001` | Persistent ProjectStore/WAL/current publication | unavailable until E2-D |
| `PROJECT-E2-DEFER-002` | Installed addons/SavedVariables/logs | unavailable |
| `PROJECT-E2-DEFER-003` | Full dependency source auto-index | unavailable unless explicitly supplied/profiled |
| `PROJECT-E2-DEFER-004` | Blizzard UI/reference graph/skeletons | E3 |
| `PROJECT-E2-DEFER-005` | Search/lineage/impact/CBM/runtime | later |
| `PROJECT-E2-DEFER-006` | Empty/default success | prohibited |

## Acceptance gate

E2-C is incomplete until all nondeferred cases execute; TOC/XML parsers preserve unknowns and never execute content; selected variants remain isolated; static load claims stay static; analyzer physical/virtual units bind exactly; E2-B fact/output seams preserve evidence/coverage; graph rejections remain visible; invalidation removes stale outputs and widens safely; candidate/persistent publication boundaries remain separate; one synthetic and one pinned user-owned addon fixture pass; and canonical results are byte-identical under worker/order/update permutations.
