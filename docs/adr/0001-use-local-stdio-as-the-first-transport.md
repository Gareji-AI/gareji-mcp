# Use local STDIO as the first MCP transport

Gareji MCP v0 runs as a user-enabled local STDIO server and delegates all durable behavior to Core and Board Interfaces. This avoids an always-on network listener for the local-first workflow, matches current Codex client support, and keeps Streamable HTTP deferred until a real Cloud or remote-client requirement justifies the additional authentication and deployment surface.
