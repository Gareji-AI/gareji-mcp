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

Private local design scaffold. No implementation language, remote repository, release, or public license has been finalized. The standalone MCP Interface will be proven before it is bundled with a Codex Plugin, Stop Hook, or sample Skills.
