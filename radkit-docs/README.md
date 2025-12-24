# radkit Companion Documentation

This repository contains authoritative documentation and reference examples for the `radkit` Rust SDK.
It is explicitly structured to help AI coding assistants (Cursor, GitHub Copilot, Claude) understand the library's architecture and avoid common pitfalls.

## 🤖 For AI Context
If you are using an LLM to write `radkit` code, **add the following file to your context window immediately**:

> **[`llms.txt`](./llms.txt)**

This file contains condensed type signatures and critical constraints (e.g., Immutable Builders, Feature Flags).

## 📚 Documentation

- **[AGENTS.md](./AGENTS.md)**: Deep dive into the A2A (Agent-to-Agent) Protocol, State Machine, and Lifecycle.
- **[Examples](./examples/)**: "Golden" reference implementations.

## 🧩 Feature Matrix

`radkit` is split into two distinct modes. You must choose the right one.

| Mode | Feature Flag | Key Types | Use Case |
|------|-------------|-----------|----------|
| **Core** | `default` | `LlmFunction`, `LlmWorker`, `Thread` | CLI tools, scripts, embedding AI into apps. |
| **Runtime** | `runtime` | `Agent`, `Runtime`, `SkillHandler` | Standalone A2A Agents, Long-running servers. |

## ⚡ Quick Examples

### 1. Stateless Function (Core)
See [`examples/core_minimal.rs`](./examples/core_minimal.rs)
```rust
let func = LlmFunction::<Output>::new(llm);
let result = func.run(thread).await?;
```

### 2. Full Agent Server (Runtime)
See [`examples/runtime_full.rs`](./examples/runtime_full.rs)
```rust
let runtime = Runtime::builder(agent_def, llm).build();
runtime.serve("0.0.0.0:3000").await?;
```

### 3. Tool Usage
See [`examples/tool_usage.rs`](./examples/tool_usage.rs)
```rust
#[tool]
async fn my_tool(args: Args) -> ToolResult { ... }
```
