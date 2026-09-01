# E5-A corpus admission, provenance, and source closure

**Status:** normative.

## Candidate versus admitted

A candidate source can be listed as soon as an exact repository revision is known. It becomes an admitted corpus member only after every mandatory admission gate passes.

```text
CandidateCorpusInput
-> MaterializedExactSource
-> PublishedExactFacts
-> ProvenanceGrouped
-> LicensePrivacyReviewed
-> IndependentlyLabeled
-> SplitEligible
-> AdmittedCorpusMember
```

No intermediate state is described as admitted.

## Candidate-source requirements

- exact repository/provider/owner/name for audit only;
- exact immutable commit and, before implementation, tree/content manifest;
- selected source roots and materialization profile;
- no floating branch/tag/current/latest in canonical identity;
- source inventory with explicit excluded, unsupported, symlink/reparse/submodule/LFS/archive/generated/vendor states;
- exact TOC/XML/Lua/project/analyzer/graph/fact publication identities where needed;
- source-handle and evidence closure;
- explicit admission blockers and status.

## Source materialization

Materialization is owned outside `wow-recognizers`. The recognizer crate accepts exact immutable fact/source artifacts and never clones, downloads, updates or scans repositories.

Materialization must not execute:

```text
repository hooks
Git filters not explicitly sandboxed/frozen
workflows or actions
build/test/release scripts
package managers
source generators
Lua/XML/script handlers
installers
addons or WoW client
```

## Provenance grouping

Each source/example receives one or more immutable group identities:

```text
current repository group
upstream lineage/fork family
copied/vendor library family
generated-template family
shared authoring lineage when evidenced
source-content near-duplicate cluster
semantic pattern family
```

The split key uses the strongest applicable group closure. Unknown independence is explicit and cannot be counted as independent generalization.

Repositories transferred or forked into the same current owner do not become independent merely because names differ. Conversely, same current owner does not prove same origin; evidence is required.

## Real initial candidate revisions

E5-A records these exact current revisions as candidate inputs:

| Candidate | Exact commit | Admission state |
|---|---|---|
| `UnknownAlienHuman/roth-ui` | `1656d4b9d33be914be2058460520e7423668d95c` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/roth-chat` | `3c995183626002965043e38a837346fb290acd8a` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/roth-tooltip` | `28426fef16daadc5808fec6d38b445a97f42a71a` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/interrupt-glow` | `786ef9f11059b28541007af92963bc9e2234f154` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/old-runes` | `9938d95759970953a7ac178a95bb5ad7aa62cb81` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/trash-panda` | `f27ba9f09be0f716cb2c5f7605ed697d8aabb320` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/gcd-optimizer` | `00d8bd22f03b1136841f548c0a4a5a776c1a7c71` | pinned revision; remaining gates pending |
| `UnknownAlienHuman/roth-blizz-plates` | `61de4d4d49ccf229ff3b7bff1ae1b5f97351b762` | pinned revision; remaining gates pending |

These names may appear in manifests/reports only. Rules cannot branch on them.

## License, notices, and artifact classes

Review separately:

```text
raw source bytes
bounded source excerpts
normalized facts
source maps/evidence handles
hand-authored labels
synthetic mutations
recognizer pack rules
case/metric reports
graph proposals
committed fixtures
public release artifacts
```

A repository license observation does not automatically answer all derived-artifact or redistribution questions. Unknown/conflicting state blocks the affected artifact class, not unrelated local metadata analysis.

## Privacy

Public repository visibility is not a privacy classification for every historical/local artifact. Corpus admission records whether source/facts/excerpts may be stored locally, committed, shared with reviewers or published.

Private/local source requires an explicit local-only corpus profile and cannot enter public fixtures by reference to owner permission alone.

## Label independence evidence

Admission records how labels were created and confirms that reviewers did not copy candidate-pack/current-recognizer output. Review tools may display exact source/facts/graph inputs, but candidate output is hidden for blind labeling where the profile requires.

## Exclusions and quarantine

Quarantine examples with:

- unresolved license/privacy;
- incomplete materialization or fact coverage affecting labels;
- unknown upstream/fork/copy grouping;
- unresolved label conflicts;
- source corruption or generation mismatch;
- unsupported fact schema;
- potential train/test leakage;
- unbounded/unsafe source or fixture requirements.

Quarantine is not negative training data.

## Updates

A new commit/source publication creates a new candidate-source identity and examples. It does not mutate prior corpus bytes or silently inherit labels. Labels may be revalidated/migrated only through an explicit evidence-preserving versioned operation.

## Removal

Removing a repository or source from future corpus versions does not delete historical evaluation evidence needed to reproduce prior decisions. Distribution/privacy policy may require redaction/tombstones while preserving stable non-sensitive audit records.

## Admission validation

- every exact revision/tree/content/source/publication/fact ID resolves;
- materialization/security report complete for admitted roots;
- no executed source path;
- provenance group closure complete enough for split policy;
- license/privacy/notice decision covers each artifact class;
- labels independently reviewed with evidence;
- mandatory capabilities/coverage available or explicit `NotEvaluated` example;
- no pack output in expected-label identity;
- budgets and canonical bytes pass;
- all blockers resolved for the claimed admission class.
