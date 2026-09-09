# Ketho type-inventory drift gate

The current-source workflow does not assume that Ketho's `Annotations/Core/Type` directory is permanently fixed.

After resolving the donor ref once, the workflow reads the exact Git tree for that commit and requires every entry under the directory to be a regular blob. It records and compares the normalized path inventory against the reviewed set used by the complete type-catalog probe. Additions, removals, symlinks, submodules and path changes fail before annotation generation.

The retained workflow artifact includes:

- the raw `git ls-tree` records with modes, object kinds and object IDs;
- the normalized actual path inventory;
- the reviewed expected inventory;
- the exact donor revision used for all subsequent resource reads.

This gate detects coverage drift; it does not automatically trust or ingest a new resource. A new file requires review of its syntax and authority boundaries, implementation or explicit rejection of its profile, and an intentional inventory update. Existing resources are still independently constrained by byte, declaration and output budgets and by the artifact verifier.
