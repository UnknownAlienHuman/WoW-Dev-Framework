# Applications

- [wow](wow/README.md): current CLI and selected stdio LSP/MCP routes.
- [reference builder](wow-reference-builder/README.md): existing pack commands;
  #196 replaces that workflow with simple native annotation generation.

[Architecture](../docs/ARCHITECTURE.md) defines current scope and
[the queue](../docs/REFACTOR_QUEUE.md) defines work. CLI is the primary complete
entry point; upstream EmmyLua supplies general Lua language-server features.
No daemon, HTTP, release client or provider shell is required. Keep application
code focused on arguments, framing, configuration and output; call cohesive
existing owners instead of duplicating their algorithms.
