# Owned foreground stdio boundary

Issue #187 selects one explicit foreground LSP execution profile. Its App I/O
controller owns duplicated stdin/stdout pipe endpoints and never sends a file,
descriptor, raw handle or I/O callback to the Service worker. Only that controller
reads or writes those endpoints; it has no outstanding input operation when it
checks readiness. The worker returns owned use-case results and bounded protocol
output; the controller frames the read results.

The App keeps `deny(unsafe_code)` by default. The one exception is the private
`src/lsp/io/windows/ffi.rs` metadata module, with four fixed Windows calls:
`GetNamedPipeInfo`, `GetNamedPipeHandleStateW`, `SetNamedPipeHandleState`, and
`PeekNamedPipe`. Other framework crates retain their inherited forbid policy.
The App preserves the workspace Clippy rules and additionally denies undocumented
unsafe blocks. This is an App transport adapter, not a semantic owner or a public
OS execution API.

Each unsafe block borrows a live owned `File`; output pointers refer to exclusive
initialized stack integers for the duration of a synchronous call. Optional
outputs are null, with a zero name/data buffer size. No pointer, source buffer,
handle or pending OS operation escapes. Errors are captured immediately. The
safe callers first validate byte-pipe type/read mode, access and synchronous
handle mode. Unsupported handles fail before mutation. No caller chooses a raw
handle, function pointer, syscall, external endpoint or capability through wire
input. The only endpoints are the process's selected standard streams.

Windows input uses a readiness byte count, then reads at most that count on the
sole I/O thread. Zero available bytes are pending, not EOF. Only broken/disconnected
pipe errors map to EOF. Output uses small nonblocking writes; a zero-byte write
is backpressure. The exact original output mode is captured, restored and read
back on checked close, including an initially nonblocking mode. RAII restores
on construction failure and unwinding; an explicit restoration failure overrides
normal completion. There is no blocking pipe flush, linger pool or detached reader.

Unix accepts pipe endpoints, captures both complete status-flag sets before any
mutation, uses nonblocking reads/writes, and restores both originals on close.
Both restoration attempts run even if one fails. EOF, pending input and output
backpressure remain distinct. These operational changes never alter an overlay,
source file, owner identity, semantic budget or authorization.

The official Windows API documentation was read at sdk-api revision
`656e9656bfaef34dd29cd7ee9e1146aec17e924c` for the implementation operation. It
warns that synchronous `PeekNamedPipe` can block with concurrent I/O; the exclusive
controller/no-pending-I/O invariant is required. This adapter does not claim hard
OS-driver latency, native analyzer preemption, client receipt acknowledgement or
durable replay. Service owns cancellation intent, owner acknowledgement, exact
result and delivery state; transport disconnect never fabricates cancellation.

Product qualification must reach the actual LSP process with data/EOF, partial
writes, a nonreading peer, exact original-mode restoration, output failure,
constructor refusal and panic cleanup, then cancellation and another request in
the same session. An artifact-only adapter probe is feasibility evidence only.
