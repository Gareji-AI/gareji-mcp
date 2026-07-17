use std::sync::Arc;

use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::{Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_handler, tool_router};
use serde_json::json;

use crate::backend::{BackendError, GarejiBackend};
use crate::types::{
    ActiveWorkItemOutput, ActiveWorkItemRequest, CheckpointStatusOutput, CheckpointStatusRequest,
    ListProjectsOutput, ProjectContextOutput, ProjectContextRequest, RecordProgressOutput,
    RecordProgressRequest,
};

#[derive(Clone)]
pub struct GarejiMcpServer {
    backend: Arc<dyn GarejiBackend>,
}

#[tool_router]
impl GarejiMcpServer {
    #[must_use]
    pub fn new(backend: Arc<dyn GarejiBackend>) -> Self {
        Self { backend }
    }

    #[tool(
        description = "List only Gareji projects visible to this client, including granted operations and connected context sources."
    )]
    fn list_projects(&self) -> Result<Json<ListProjectsOutput>, McpError> {
        self.backend.list_projects().map(Json).map_err(map_error)
    }

    #[tool(
        description = "Read bounded project context from every enabled source while preserving source identity. Call this before recording work."
    )]
    fn get_project_context(
        &self,
        Parameters(request): Parameters<ProjectContextRequest>,
    ) -> Result<Json<ProjectContextOutput>, McpError> {
        self.backend
            .get_project_context(&request.project_id, request.work_item_id.as_deref())
            .map(Json)
            .map_err(map_error)
    }

    #[tool(
        description = "Select the explicit active Work item for one connected project. This does not transition Work item state."
    )]
    fn set_active_work_item(
        &self,
        Parameters(request): Parameters<ActiveWorkItemRequest>,
    ) -> Result<Json<ActiveWorkItemOutput>, McpError> {
        self.backend
            .set_active_work_item(&request.project_id, &request.work_item_id)
            .map(Json)
            .map_err(map_error)
    }

    #[tool(
        description = "Durably submit one structured Progress Checkpoint. A receipt is separate from per-destination delivery success."
    )]
    fn record_progress(
        &self,
        Parameters(request): Parameters<RecordProgressRequest>,
    ) -> Result<Json<RecordProgressOutput>, McpError> {
        self.backend
            .record_progress(&request.checkpoint)
            .map(Json)
            .map_err(map_error)
    }

    #[tool(
        description = "Inspect one immutable Progress Checkpoint and its independent destination delivery states."
    )]
    fn get_checkpoint_status(
        &self,
        Parameters(request): Parameters<CheckpointStatusRequest>,
    ) -> Result<Json<CheckpointStatusOutput>, McpError> {
        self.backend
            .get_checkpoint_status(&request.checkpoint_id)
            .map(Json)
            .map_err(map_error)
    }
}

#[tool_handler]
impl ServerHandler for GarejiMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo::new(ServerCapabilities::builder().enable_tools().build())
            .with_server_info(Implementation::from_build_env())
            .with_instructions(
                "Read project context before recording work. Use explicit project and Work item IDs. Preserve source attribution. A Checkpoint receipt confirms durable local intake, not delivery to every destination. This server cannot execute shell commands, mutate Git, publish, retrieve secrets, or complete Work items."
                    .to_owned(),
            )
    }
}

fn map_error(error: BackendError) -> McpError {
    McpError::invalid_params(error.to_string(), Some(json!({ "category": error.code() })))
}

#[cfg(test)]
mod tests {
    use crate::FixtureBackend;
    use rmcp::{ClientHandler, ServiceExt};

    use super::*;

    #[test]
    fn server_exposes_exactly_the_five_v0_tools() {
        let _server = GarejiMcpServer::new(Arc::new(FixtureBackend::default()));
        let mut tools: Vec<_> = GarejiMcpServer::tool_router()
            .list_all()
            .into_iter()
            .map(|tool| tool.name.to_string())
            .collect();
        tools.sort();
        assert_eq!(
            tools,
            [
                "get_checkpoint_status",
                "get_project_context",
                "list_projects",
                "record_progress",
                "set_active_work_item",
            ]
        );
    }

    #[derive(Clone, Default)]
    struct TestClient;

    impl ClientHandler for TestClient {}

    #[tokio::test]
    async fn mcp_client_completes_handshake_and_lists_the_v0_tools() {
        let server = GarejiMcpServer::new(Arc::new(FixtureBackend::default()));
        let (server_transport, client_transport) = tokio::io::duplex(16 * 1024);
        let server_task = tokio::spawn(async move {
            let service = server.serve(server_transport).await.unwrap();
            let _ = service.waiting().await;
        });

        let client = TestClient.serve(client_transport).await.unwrap();
        let tools = client.list_tools(Option::default()).await.unwrap();
        assert_eq!(tools.tools.len(), 5);
        assert!(
            tools
                .tools
                .iter()
                .any(|tool| tool.name == "record_progress")
        );

        client.cancel().await.unwrap();
        server_task.await.unwrap();
    }
}
