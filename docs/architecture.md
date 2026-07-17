# Gareji MCP architecture

## Purpose

Gareji MCP is a transport Adapter at the client-to-Gareji seam. Its small tool surface gives Codex useful project context and progress operations while keeping the difficult behavior in deep Core and Board Modules.

## Flow

```text
Codex MCP client
  -> Gareji MCP tool translation
  -> reusable local Core child-process Adapter
  -> Core Registry, permission, and Progress Recorder Interfaces
  -> reusable Board child-process Adapter for active-work assessment
  -> durable receipt or sourced result
```

MCP does not write directly to SQLite, Markdown, JSON knowledge stores, or Board files. It does not decide task transitions or projection retries.

## Initial transport

v0 uses a local STDIO MCP server started by a command. It lazily starts one `gareji-core bridge` child process and reuses it for all tool calls. This fits the local-first product, amortizes Core startup, requires no always-on network listener, and is supported by Codex clients. Streamable HTTP is deferred until Gareji Cloud or another real remote client exists.

The server initialization `instructions` field will summarize cross-tool ordering, permission limits, and rate guidance. Its first 512 characters must be self-contained so a client can make safe tool choices from the truncated guidance.

## Client trust

Users install and enable Gareji MCP explicitly. A sample project-scoped `.codex/config.toml` is documentation only until copied into a trusted project. Gareji does not silently write global Codex configuration.

The trusted Codex `Stop` Hook is a separate capture Adapter. It invokes the same Progress Recorder but is not hidden inside an MCP tool call. Hook installation, hash review, and re-review after changes remain visible user actions.

## Future Codex Plugin

After the standalone MCP server and Hook work independently, a Codex Plugin may bundle:

- the MCP server declaration;
- the trusted Stop Hook;
- sample Skills and agent profiles;
- installation and permission guidance.

Bundling changes distribution convenience, not authority. Core still owns generic rules and Board still owns Work item state.

## Implementation status

The single-binary Rust MCP server uses the production `CoreBackend` Adapter by default. Core owns project registration, grants, active-work references, Checkpoint validation, SQLite durability, and delivery state. Board now validates Work item existence, project relationship, and active-work eligibility before Core changes the selection. The Fixture Adapter is disposable test support and owns no production state.

The next integration gate is Board reconciliation of accepted Progress Checkpoints into visible Work item timelines and state recommendations.
