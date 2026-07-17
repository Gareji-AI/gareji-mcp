# ADR 0002: Use Rust and the official MCP SDK

Status: accepted.

Gareji MCP ships as one Rust binary using the official `rmcp` SDK and local STDIO transport. The transport Module translates the five v0 tools through a small `GarejiBackend` Interface; it does not issue SQL, run Codex, or duplicate Core policy.

The first implementation includes an in-memory Fixture Adapter so tool schemas, server instructions, errors, idempotency, and client compatibility can be tested independently of Core. The Fixture Adapter is test support, is not used by the production binary, and does not make the MCP repository authoritative for projects or Checkpoints.

Python, Node.js, and an HTTP listener are not runtime dependencies. Streamable HTTP remains deferred until there is a real remote client.
