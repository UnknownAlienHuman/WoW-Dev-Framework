# Closed primary structural pack and renderer profiles

Reviewed before implementation on 2026-10-11 for #122. The exact descriptors are
frozen in [profiles.json](../../wow-service/e3/native-primary-pack/profiles.json).
This selected recipe extends the native primary maps and L0/L1 without declaring
full E3-B acceptance. The compiled descriptors must equal those canonical values.

The pack profile `wow-context/primary-structural-pack/1` selects one exact root
and explicit L0, L1 or Both, with the existing level/root compatibility rules.
It retains the exact universe binding and immutable skeleton/source/support
closure. Original selected native graph nodes and edges become typed context
items with root/inclusion reason, original record and origin-map references,
payload digest, stable item identity, privacy/boundary labels and selection trace.
Node and edge ordering is native ID ordering. Items are mandatory; no pack stage
silently drops them to fit. Upstream omissions remain scoped to their producer.

Pack construction continues the existing skeleton Budget. Private renderer work
continues the same ledger outside canonical pack identity; rendering cannot reset
it. The pack budget records reservations before pack finalization; exact final
canonical byte length belongs to its accessor/delivery metadata, avoiding a
self-inclusive hash. Renderer selection, validation reports, metrics and delivery
state are excluded from the semantic pack ID. Both renderers use the same pack.

`wow-context/primary-pack-json/1` emits the exact lossless canonical pack JSON,
UTF-8 without a trailing LF. `wow-context/primary-pack-markdown/1` emits LF-ended
fixed typed records in the frozen template order. Header/profile, exact binding,
request, boundary notice, native skeletons, each entity/relation context item,
selection trace, coverage/omissions and budget are represented once. Each record
has a template/semantic target and exact UTF-8 byte range. Markdown payloads are
quoted canonical JSON strings with Markdown metacharacters, line separators and
bidi controls encoded as JSON Unicode escapes; decoding preserves the value.
Source metadata cannot introduce a template, boundary or executable Markdown.
Both renderer profiles declare no rendering loss. Overflow refuses, without
clipping or changing semantic selection. Requested renderers are required.

Privacy remains LocalMetadataOnly, bodies/excerpts are excluded, tokens are
Unavailable and continuation is NotEvaluated. No external-source permission,
current corpus, reference enrichment, primary signature admission, runtime,
lineage/search or launch claim follows. Public CLI uses the normative exact
context-profile and renderer-profile options; zero renderers returns a validated
semantic pack, while two renderers use fixed JSON-then-Markdown order.
