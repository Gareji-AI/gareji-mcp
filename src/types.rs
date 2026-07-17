use rmcp::schemars;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Clone, Debug, Deserialize, schemars::JsonSchema)]
pub struct ProjectContextRequest {
    pub project_id: String,
    pub work_item_id: Option<String>,
}

#[derive(Clone, Debug, Deserialize, schemars::JsonSchema)]
pub struct ActiveWorkItemRequest {
    pub project_id: String,
    pub work_item_id: String,
}

#[derive(Clone, Debug, Deserialize, schemars::JsonSchema)]
pub struct RecordProgressRequest {
    /// A `gareji.progress-checkpoint.v0` object.
    pub checkpoint: Value,
}

#[derive(Clone, Debug, Deserialize, schemars::JsonSchema)]
pub struct CheckpointStatusRequest {
    pub checkpoint_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ProjectView {
    pub id: String,
    pub name: String,
    pub execution_workspace: String,
    pub context_sources: Vec<String>,
    pub grants: Vec<String>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ListProjectsOutput {
    pub projects: Vec<ProjectView>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct SourcedContext {
    pub workspace_id: String,
    pub source_id: String,
    pub freshness: String,
    pub content: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct ProjectContextOutput {
    pub project_id: String,
    pub work_item_id: Option<String>,
    pub sources: Vec<SourcedContext>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
#[allow(clippy::struct_field_names)]
pub struct ActiveWorkItemOutput {
    pub project_id: String,
    pub previous_work_item_id: Option<String>,
    pub current_work_item_id: String,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct DeliveryView {
    pub destination_id: String,
    pub status: String,
    pub attempts: u32,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, schemars::JsonSchema)]
pub struct RecordProgressOutput {
    pub checkpoint_id: String,
    pub duplicate: bool,
    pub deliveries: Vec<DeliveryView>,
}

#[derive(Clone, Debug, PartialEq, Serialize, schemars::JsonSchema)]
pub struct CheckpointStatusOutput {
    pub checkpoint_id: String,
    pub checkpoint: Value,
    pub deliveries: Vec<DeliveryView>,
}
