# Repository automation policy

The repository has three maintained workflows:

- `ci.yml`: current stable Rust on Linux and Windows, Python/source-bridge tests,
  strict Clippy and rustdoc, plus a compatible dependency-update lane.
- `current-source-bundle.yml`: a current Gethe/live checkout, coherent source
  manifest, generated API/topology producers and Rust validation.
- `branch-hygiene.yml`: remove only branches already contained in main or with
  an identical tree; do not discard unique divergent work.

Build and source CI are read-only and never modify source or publish commits.
The branch-hygiene job alone has the write permission needed for its stated role.
No self-mutating finalizer, recovery payload or embedded implementation workflow
is part of the supported development path. Publish reviewed changes separately,
using a non-forced update of the expected branch, and verify the resulting commit.

A new workflow requires real executable commands, an explicit owner and gate,
bounded inputs, minimal permissions and a documented failure path. A successful
source/build job is not a supported release, in-client test or installer proof.
