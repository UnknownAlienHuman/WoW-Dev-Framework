# `wow` application

`wow-cli` is the offline frontend. Delivery state lives in
[current status](../../docs/CURRENT_STATUS.md); command/config/result contracts
live in [FRONTENDS](../../docs/FRONTENDS.md); implementation order lives in the
[queue](../../docs/REFACTOR_QUEUE.md).

Current check/status routes are transitional. Slice 1A replaces them with one
ordinary `wow check` adapter to the public Service operation. The application owns
argument/config precedence, caller-facing display roots, rendering and exit
status only. Project/Service own source admission, authority, limitations,
snapshots and analysis assembly.

Do not duplicate the Slice 1A contract here. Do not open source, parse load facts,
construct lower requests or preserve legacy profile/publication/result machinery.
