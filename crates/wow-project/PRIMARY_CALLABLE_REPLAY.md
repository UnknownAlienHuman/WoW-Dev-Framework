# Ordinary Primary callable replay

This route retains physical Main, exact Library files and the selected declaration/
signature recipe, then rebuilds the real Emmy analysis when read. Its replay payload
is `wow-project/native-project-replay/primary-callables/1`; hook/witness selections
use `wow-project/native-project-replay/selected-components/1`.

All current Project replay payloads share `publication::STORAGE_SCHEMAS`. The store
catalog no longer encodes feature combinations. Exact declaration, signature, hook,
XML and platform choices remain in `ProjectConfiguration`; hydration and readback
must reproduce those choices and analyzer/graph identities. A different selection
may open the same physical store but its read is rejected after configuration is
reconstructed.

Current publication performs inactive owner readback before Current activation.
Incompatible derived stores are rebuilt from source and configuration; no legacy
catalog reader, migration or feature-specific epoch is provided. These observations
remain source structure, not runtime guarantees. Broader context work is tracked by
[#122](https://github.com/UnknownAlienHuman/WoW-Dev-Framework/issues/122).
