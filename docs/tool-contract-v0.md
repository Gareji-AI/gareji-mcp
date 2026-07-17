# Gareji MCP tool contract v0

Status: proposed for the first implementation.

## Server instructions

The server tells clients to read context before recording work, use explicit project and Work item IDs, preserve source attribution, and treat a Checkpoint receipt separately from per-destination delivery. It states that the server cannot execute shell commands, mutate Git, publish, retrieve secrets, or complete Work items directly.

## Tools

### `list_projects()`

Returns projects visible to the client, their stable IDs, connected Execution workspaces, available context sources, and granted operations. It does not scan unrelated directories.

Side effect: none.

### `get_project_context(project_id, work_item_id?)`

Returns bounded Sourced context from every enabled read connection. Each result preserves `workspace_id`, `source_id`, freshness, and content or evidence reference. Conflicting sources remain distinguishable.

Required grant: `read_context`.

Side effect: none.

### `set_active_work_item(project_id, work_item_id)`

Stores the explicit active Work item for the project's connected Execution workspace and returns the previous and current selection. It rejects terminal, unrelated, or inaccessible Work items.

Required grant: `select_active_work`.

Side effect: local active selection only; no Work item transition.

### `record_progress(checkpoint)`

Submits one structured Progress Checkpoint to Core. Core validates, redacts, deduplicates, writes durably, and creates independent deliveries for every enabled write connection before returning a Checkpoint receipt.

Required grant: `write_progress`.

Side effect: append-only local Checkpoint and delivery records. The `recommended_state` field does not apply a Board transition.

### `get_checkpoint_status(checkpoint_id)`

Returns the immutable Checkpoint identity and per-destination `pending`, `synced`, `conflict`, or `failed` status. It preserves partial success and retry diagnostics without exposing raw secrets or logs.

Required grant: `write_progress` or a future explicit `read_progress` grant.

Side effect: none.

## Stable error categories

- `invalid_request`
- `permission_denied`
- `project_not_found`
- `work_item_not_found`
- `workspace_not_connected`
- `checkpoint_not_found`
- `core_unavailable`
- `conflict`
- `internal_error`

Errors are bounded and actionable. They do not echo credentials, raw content from an inaccessible source, full diffs, transcripts, or Core sidecars.

## Explicit exclusions

v0 has no `complete_work_item`, `run_shell`, `write_file`, `delete_file`, `commit`, `push`, `publish`, `get_secret`, generic query, or pass-through Core command. New tools require a concrete user workflow, a permission definition, a stable authority owner, and threat-model review.
