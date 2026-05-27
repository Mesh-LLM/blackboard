# mesh-llm blackboard plugin

Blackboard is an external mesh-llm plugin for sharing ephemeral status,
findings, questions, and tips across a mesh.

## Build

```bash
cargo build
```

The plugin currently depends on the `mesh-llm-plugin` crate from the
`codex/plugin-cli-commands` branch until that plugin CLI support lands on
`main`.

## Configure

```toml
[[plugin]]
name = "blackboard"
command = "mesh-llm-plugin-blackboard"
```

Once configured, mesh-llm routes the plugin CLI command through `cli.run`:

```bash
mesh-llm blackboard
mesh-llm blackboard --search "QUESTION"
mesh-llm blackboard "STATUS: testing external plugins"
mesh-llm blackboard install-skill
```

For MCP, use mesh-llm's generic plugin MCP bridge:

```bash
mesh-llm plugin mcp
```
