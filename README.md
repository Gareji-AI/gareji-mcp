# Gareji MCP

Gareji MCP is the thin, user-enabled Codex transport for Gareji projects, sourced context, active Work item selection, Progress Checkpoint recording, and delivery inspection. It delegates validation, permissions, storage, redaction, idempotency, and projection behavior to Gareji Core and Gareji Board instead of becoming another state owner.

## v0 tools

```text
list_projects()
get_project_context(project_id, work_item_id?)
set_active_work_item(project_id, work_item_id)
record_progress(checkpoint)
get_checkpoint_status(checkpoint_id)
```

The v0 Interface exposes no arbitrary file write, deletion, shell execution, Git commit, Git push, publication, secret access, or direct Work item completion.

## Codex integration

The first transport is a local STDIO MCP server. Installation and activation are explicit user actions. Project-scoped Codex configuration and lifecycle Hooks load only for trusted projects, and changed Hooks require review again. The example config in [examples/codex-config.toml](examples/codex-config.toml) is inert until a user copies or installs it deliberately.

See [the architecture](docs/architecture.md), [the v0 tool contract](docs/tool-contract-v0.md), and [the domain glossary](CONTEXT.md).

## Status

The Rust STDIO server exposes the five v0 tools through a production `CoreBackend` Adapter. It lazily starts one `gareji-core bridge` child process, reuses it across calls, and leaves Registry, grant, validation, and Progress Recorder behavior inside Core. The in-memory Fixture Adapter remains only for conformance tests.

Install `gareji-board`, `gareji-core`, and `gareji-mcp`, then run `gareji-mcp`. Set `GAREJI_CORE_BIN` or `GAREJI_BOARD_BIN` only when an executable is not on `PATH`; database overrides are available through `GAREJI_CORE_DB` and `GAREJI_BOARD_DB`. A missing Core is reported as `core_unavailable`, while active-work selection reports a missing Board as `board_unavailable` without disabling unrelated tools.

```bash
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
```
