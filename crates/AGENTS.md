# Crate contribution guidance

Follow root [AGENTS.md](../AGENTS.md). Use the
[implementation map](../docs/IMPLEMENTATION.md#code-ownership) and selected
Issue to find the current owner, real caller and donor.

Keep parsing/normalization/analysis/SQL in their existing cohesive owner.
Applications may call one owner directly; Service composes multi-owner work.
Keep dependencies acyclic: owners do not depend on Service or applications.
Do not add a crate, shared type or forwarding adapter solely to preserve an
old diagram. Replace obsolete paths and their exclusive tests/docs together.
