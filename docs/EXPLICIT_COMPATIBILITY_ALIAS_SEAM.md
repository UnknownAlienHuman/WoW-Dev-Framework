# Explicit compatibility declaration owner seam

This E4 seam implements #175. `wow-reference` owns a configured compatibility
declaration, its original input and its exact target binding. It records a literal
string relation; it asserts no Blizzard/current/runtime truth, deprecation,
replacement, user intent or review authorization. Annotation type aliases,
Possible Lua hops, comments and similarity are not inputs to this producer.

One explicitly selected digest-pinned JSON resource binds the complete validated
profile identity fingerprint, base Reference generation/view digest, alias values
and target partition/key/digest. The fixed policy is case-sensitive exact matching
in that Reference universe; source data cannot choose a policy or confidence score.
Only native API, event and declaration targets are admitted. Alias-target partitions
are forbidden, so resolution never follows an alias chain. Missing/conflicting/
changed targets remain nonresolved; multiple exclusive target addresses for a
literal value block the whole value. Identical declarations retain all original
indices and source evidence without duplicating target identity.

The existing immutable `ReferenceView` carries a Partial metadata partition with
the original bounded input/header and owner-produced records. Final generation
derivation includes that partition; input binds the base generation, avoiding a
self-reference. Native partitions and conflicts are unchanged. Only the owner can
rebind its existing sealed caches after checking their original view binding and
partition equality; restriction IDs are regenerated for the final generation.
The first SourceReference route excludes type-resolution overlays instead of
retagging their evidence.

On retained reads, Reference revalidates its own original metadata against the
actual retained base records and reproduces the stored partition/final view.
This is not native AST/cache hydration from Search DTOs. Search copies resolved
and nonresolved metadata, deduplicates actual target documents and preserves alias
record/source evidence. Only a resolved relation yields band-3 alias evidence;
Partial alias coverage never authorizes a general absence decision. The same
Reference object and existing Store roots/retire/GC/replay are used. Omitted input
preserves existing reports, generations, profiles, handles and results.
