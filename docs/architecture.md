# Gareji MCP architecture

## Purpose

Gareji MCP is a transport Adapter at the client-to-Gareji seam. Its small tool surface gives Codex useful project context and progress operations while keeping the difficult behavior in deep Core and Board Modules.

## Flow

```text
Codex MCP client
  -> Gareji MCP tool translation
  -> project permission check through Core
  -> Core or Board Interface
  -> durable receipt or sourced result
```

MCP does not write directly to SQLite, Markdown, JSON knowledge stores, or Board files. It does not decide task transitions or projection retries.

## Initial transport

v0 uses a local STDIO MCP server started by a command. This fits the local-first product, requires no always-on network listener, and is supported by Codex clients. Streamable HTTP is deferred until Gareji Cloud or another real remote client exists.

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

## Implementation gate

Core provides Capability Gate and SQLite-backed Progress Recorder foundations. Gareji MCP now has a single-binary Rust STDIO server and an in-memory Fixture Adapter proving the five-tool transport Interface. The Fixture Adapter is disposable and owns no production state. The next gate is a Core/Board Adapter plus an MCP client conformance roundtrip using the same `GarejiBackend` Interface.
