# genie-worksheets-rs

Pure-Rust port of [stanford-oval/genie-worksheets](https://github.com/stanford-oval/genie-worksheets) — knowledge-guided conversational task agents using declarative worksheets, typed slot validation, confirmation policies, and stateful multi-turn execution.

Zero Python runtime required.

## Workspace Crates

| Crate | Purpose |
| :--- | :--- |
| `worksheets-core` | `WorksheetSpec`, `FieldSpec`, `FieldType`, `WorksheetInstance`, `AgentPolicy`, `SemanticParser`, `GenieRuntime` |
| `worksheets-cli` | `genie-worksheets` CLI (`demo`) |

## Quickstart

```bash
cargo test --workspace
cargo run -p worksheets-cli -- demo
```
