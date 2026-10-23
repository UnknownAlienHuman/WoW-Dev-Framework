# Crate guidance

Follow root [AGENTS.md](../AGENTS.md). Use the
[implementation map](../docs/IMPLEMENTATION.md#code-ownership) and actual callers.
Keep Lua analysis in Emmy, Blizzard normalization in Reference, emission in
Annotations, load facts in Project and SQL in Store. Applications can call a
single cohesive owner directly; Service is for genuine composition. Keep
acyclic dependencies and delete forwarding layers that add no behavior.
