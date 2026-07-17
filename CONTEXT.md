# Gareji MCP

Gareji MCP connects an explicitly enabled agent client to existing Gareji behavior without becoming a second state or policy owner.

## Language

**Gareji MCP**:
The transport product that exposes a bounded set of Gareji operations to an enabled MCP client.
_Avoid_: Gareji Core, database, agent runtime

**MCP tool**:
One permission-scoped operation exposed by Gareji MCP and implemented through an existing Core or Board Interface.
_Avoid_: shell command, Plugin

**Project grant**:
The user's explicit permission for one MCP client to read context, select active work, or record progress for one Board project.
_Avoid_: project trust, operating-system permission

**Sourced context**:
Context returned with its Knowledge workspace and source identity preserved so an agent can distinguish several connected sources.
_Avoid_: merged truth, model memory

**Active Work item**:
The explicit Work item associated with direct work in one connected Execution workspace.
_Avoid_: selected Runner candidate, inferred branch match

**Checkpoint receipt**:
The durable acknowledgement returned after Core accepts one immutable Progress Checkpoint, before every external projection necessarily succeeds.
_Avoid_: completed Work item, delivery receipt

**Delivery status**:
The per-destination projection result for a Progress Checkpoint, including partial success across several Knowledge connections.
_Avoid_: Checkpoint receipt, Work item state
