# ADR 0003: Reusable Core child process

Status: accepted.

The production `CoreBackend` Adapter starts `gareji-core bridge` lazily and reuses that child process across MCP tool calls. The child owns Project Registry and Progress Recorder access. MCP sends only versioned bounded operations and never reads or writes Core SQLite tables.

`GAREJI_CORE_BIN` may select the installed Core executable and `GAREJI_CORE_DB` may select an explicit local database. Missing or invalid Core installations return the stable `core_unavailable` category without preventing the MCP server handshake.

The in-memory Fixture Adapter remains only for transport conformance tests. A small internal transport Seam allows those tests to replace the child process without exposing process details through the public `GarejiBackend` Interface.

This is not a background daemon. Closing MCP closes the child pipes and ends the local Core process. A future daemon or Cloud Adapter can replace this transport at the same Seam when a real multi-client requirement exists.

Core may in turn keep one `gareji-board bridge` child for Board-owned active-work assessment. This preserves the same local process composition: MCP owns transport, Core owns operational state, and Board owns Work item state.
