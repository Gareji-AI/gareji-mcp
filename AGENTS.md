# Gareji MCP contributor guide

## Scope

Gareji MCP is a thin transport. It owns MCP protocol translation, tool descriptions, client-facing validation errors, and connection lifecycle. Gareji Core owns generic permissions, durable Progress Checkpoint intake, idempotency, redaction, local storage, and delivery. Gareji Board owns Work item state and applies recommended transitions.

Do not create an MCP-only database, duplicate Core policy, infer task assignment silently, or expose arbitrary shell, file mutation, Git, publication, secret, or direct completion tools.

## Codex trust

- Installation and enablement are explicit user actions.
- Project configuration is loaded only in a trusted project.
- A future Stop Hook must use the same Recorder and remain independently reviewable.
- Do not place an active `.codex/config.toml` or Hook in this repository until an install flow exists; examples remain inert.

## Change discipline

- Read `CONTEXT.md`, `docs/architecture.md`, and `docs/tool-contract-v0.md` first.
- One commit represents one independently reviewable and revertible meaning.
- Keep contract examples and conformance tests with the tool behavior they prove.
- Use an English Conventional Commit subject.
- Never include credentials, personal paths, customer data, private strategy, or raw transcripts.
