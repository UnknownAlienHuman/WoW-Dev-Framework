# Emmy adapter

Follow root [AGENTS.md](../../AGENTS.md), the full
[manifesto](../../docs/ARCHITECTURE.md) and the relevant
[implementation route](../../docs/IMPLEMENTATION.md).

Use one configured Main/Library/STD session and upstream public APIs. Preserve
message/code/severity/ranges. Read [EmmyLua](../../docs/EMMYLUA_UPSTREAM.md) before
changing setup, updates or pins. The shared session already exists; do not
recreate it because an old Issue describes separate engines.
