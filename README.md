# blackboard

Blackboard is an external mesh-llm plugin for sharing ephemeral status,
findings, questions, and tips across a mesh.

## Install

```bash
mesh-llm plugins install blackboard
```

Released plugin archives include the blackboard Agent Skill under `skills/`.
After installing the plugin, install that skill into detected agents:

```bash
mesh-llm skills install
```

Agent launch commands such as `mesh-llm goose`, `mesh-llm pi`,
`mesh-llm opencode`, and `mesh-llm claude` also install available plugin skills
for that agent before starting the session.

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
command = "blackboard"
```

Once configured, mesh-llm routes the plugin CLI command through `cli.run`:

```bash
mesh-llm blackboard
mesh-llm blackboard --search "QUESTION"
mesh-llm blackboard "STATUS: testing external plugins"
mesh-llm blackboard install-skill
```

`install-skill` is kept for older mesh-llm versions that only read
`~/.agents/skills`. Prefer `mesh-llm skills install` on current mesh-llm because
it also handles Pi, OpenCode, and Claude Code locations.

For MCP, use mesh-llm's generic plugin MCP bridge:

```bash
mesh-llm plugin mcp
```
