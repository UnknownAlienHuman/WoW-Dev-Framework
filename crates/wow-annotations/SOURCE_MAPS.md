# Native source maps and navigation

The emitter records ranges while writing the final annotation text.
`src/native/maps.rs` joins these ranges to normalized native fields and their raw
source spans. `src/navigation.rs` and `src/navigation/` read the resulting maps; they
do not parse or regenerate the library or run an analyzer.

## What a map means

- Ranges are zero-based, half-open UTF-8 byte spans in the actual generated and
  source text. Do not reuse offsets against a changed buffer or reconstruct
  them by searching for a repeated name.
- Member ranges take precedence over declaration ranges, then whole-file
  ranges. Within one precision the smallest covering range wins; retain all
  equal candidates. A range query needs a map covering the whole requested span.
- Literal files can have coarse whole-file mappings. This is useful source
  navigation, not a promise of exact mapping for every emitted fragment.
- `Unmapped` means no stored map covers the query. EOF and unmapped headers can
  return it; it does not mean the source API is absent. Invalid code-point
  positions, stale text and invalid links return errors rather than guesses.
- Preserve the actual source revision/path and external alias origin. A Ketho
  catalog's revision must not be relabeled as the Blizzard source revision.

Raw documents and unprojected metadata remain with the native result. A map
does not certify type completeness or runtime behavior; partial generation
stays partial even when a particular member maps successfully.

## Reuse the current APIs

| Need | Existing API |
| --- | --- |
| One generated point/range | `source_at`, `source_for_range`. |
| Protocol text positions | `source_at_position`, `source_for_text_range`; explicit UTF-8/UTF-16/UTF-32 encoding. |
| Repeated queries | `NavigationIndex::new`; prepare once for the immutable generation. |
| Source to generated occurrences | `NavigationIndex::generated_for` and range/position methods. |
| Repeated source-buffer queries | `NavigationIndex::bind_source`. |

Current APIs check generated/source digests to reject stale buffers. This
describes existing code, not a requirement for a new proof protocol. Retain
the same-text guarantee as #196/#107 remove library reconstruction and #112
consolidates internal formats. Do not add another map inventory or cache layer.

## Check the changed behavior

Use the affected test in `tests/native_member_maps.rs`, `tests/field_mappings.rs`
or `tests/navigation*.rs`. A moved member must still point to its real source;
a relevant stale/ambiguous/unmapped case must remain honest. Reuse the current
fixtures and range/position helpers, without an artifact certification matrix.
