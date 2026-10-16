# Retained native Project generation pair

`replay::publication::RetainedGenerationPair` binds two genuine acquired Project
and Graph owners with borrowed live `ReadSnapshotCustody` values from one opened
ordinary Store. The proof cannot be deserialized from its descriptor. Borrowing
both custody values prevents closing either snapshot while the pair remains used;
the original acquired owner's nonretaining origin must match that exact read.
Logical EpochId/generation equality alone is insufficient: independent Stores can
share those IDs. Existing `is_bound_to_read` logical/twin behavior is unchanged.

The selected profile is `wow-project/retained-generation-pair/1`. It requires the
same logical project/kind/workspace/source origin/root, graph universe and opened
epoch, with explicit ordered before/after roles. Source namespace/Blizzard UI and
Reference comparison profiles remain separate. Configuration and registry
commitments are compared as compatibility metadata, never entity identity or
lineage. Configuration/registry differences remain `not_evaluated` for admission.

The descriptor retains original native publication, store/project/graph generation
and snapshot, source context, configuration, analyzer, Reference and profile IDs,
both original coverage arrays and deferred capability records. Its pair ID binds
the ordered immutable publication commitments and excludes runtime addresses,
paths, Current, process/time and delivery bounds. Same-generation observation
does not establish semantic `NoChange`; all results retain no lineage or negative
authority. Native source/evidence ownership stays in the held owners.

The CLI/Service consumer is `wow project pair` (Issue #177). It serializes a bounded
private descriptor, closes after/before/Store in reverse order and only then
delivers. This adds no stored schema, registry, producer format, durable lineage
retention or algorithm. E4-B still needs real typed producer partitions,
bounded proposals/components, ambiguity/review semantics and genuine sealed
inactive snapshot publication/validation. A descriptor cannot replace those
owner capabilities or the required corpus/runtime acceptance.
