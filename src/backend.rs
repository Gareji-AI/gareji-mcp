use std::collections::BTreeMap;
use std::sync::Mutex;

use serde_json::Value;
use thiserror::Error;

use crate::types::{
    ActiveWorkItemOutput, CheckpointStatusOutput, DeliveryView, ListProjectsOutput,
    ProjectContextOutput, ProjectView, RecordProgressOutput, SourcedContext,
};

/// Core/Board seam consumed by the transport Module and its conformance tests.
pub trait GarejiBackend: Send + Sync + 'static {
    /// List projects visible to the current local client.
    ///
    /// # Errors
    ///
    /// Returns a bounded category when the authority cannot be queried.
    fn list_projects(&self) -> Result<ListProjectsOutput, BackendError>;
    /// Read sourced context for one visible project.
    ///
    /// # Errors
    ///
    /// Returns `ProjectNotFound` for an unknown or inaccessible project.
    fn get_project_context(
        &self,
        project_id: &str,
        work_item_id: Option<&str>,
    ) -> Result<ProjectContextOutput, BackendError>;
    /// Select one explicit active Work item without transitioning its state.
    ///
    /// # Errors
    ///
    /// Returns a bounded validation, lookup, or local-operation category.
    fn set_active_work_item(
        &self,
        project_id: &str,
        work_item_id: &str,
    ) -> Result<ActiveWorkItemOutput, BackendError>;
    /// Submit one immutable Progress Checkpoint.
    ///
    /// # Errors
    ///
    /// Returns `Conflict` when an existing identity has different content.
    fn record_progress(&self, checkpoint: &Value) -> Result<RecordProgressOutput, BackendError>;
    /// Inspect one Checkpoint and its independent delivery states.
    ///
    /// # Errors
    ///
    /// Returns `CheckpointNotFound` when the identity is unknown.
    fn get_checkpoint_status(
        &self,
        checkpoint_id: &str,
    ) -> Result<CheckpointStatusOutput, BackendError>;
}

/// Disposable Adapter proving the MCP contract before Core wiring exists.
#[derive(Default)]
pub struct FixtureBackend {
    active_work_item: Mutex<Option<String>>,
    checkpoints: Mutex<BTreeMap<String, Value>>,
}

impl FixtureBackend {
    fn require_project(project_id: &str) -> Result<(), BackendError> {
        if project_id == "gareji-board" {
            Ok(())
        } else {
            Err(BackendError::ProjectNotFound)
        }
    }

    fn deliveries() -> Vec<DeliveryView> {
        vec![
            DeliveryView {
                destination_id: "local-json".to_owned(),
                status: "synced".to_owned(),
                attempts: 1,
            },
            DeliveryView {
                destination_id: "zettelkasten".to_owned(),
                status: "pending".to_owned(),
                attempts: 0,
            },
        ]
    }
}

impl GarejiBackend for FixtureBackend {
    fn list_projects(&self) -> Result<ListProjectsOutput, BackendError> {
        Ok(ListProjectsOutput {
            projects: vec![ProjectView {
                id: "gareji-board".to_owned(),
                name: "Gareji Board".to_owned(),
                execution_workspace: "fixture://gareji-board".to_owned(),
                context_sources: vec!["local-json".to_owned(), "zettelkasten".to_owned()],
                grants: vec![
                    "read_context".to_owned(),
                    "select_active_work".to_owned(),
                    "write_progress".to_owned(),
                ],
            }],
        })
    }

    fn get_project_context(
        &self,
        project_id: &str,
        work_item_id: Option<&str>,
    ) -> Result<ProjectContextOutput, BackendError> {
        Self::require_project(project_id)?;
        Ok(ProjectContextOutput {
            project_id: project_id.to_owned(),
            work_item_id: work_item_id.map(str::to_owned),
            sources: vec![SourcedContext {
                workspace_id: "demo-knowledge".to_owned(),
                source_id: "project-brief".to_owned(),
                freshness: "fixture".to_owned(),
                content: Some(
                    "Build an understandable local multi-project control surface.".to_owned(),
                ),
                evidence_ref: None,
            }],
        })
    }

    fn set_active_work_item(
        &self,
        project_id: &str,
        work_item_id: &str,
    ) -> Result<ActiveWorkItemOutput, BackendError> {
        Self::require_project(project_id)?;
        if work_item_id.trim().is_empty() {
            return Err(BackendError::InvalidRequest);
        }
        let mut active = self
            .active_work_item
            .lock()
            .map_err(|_| BackendError::Internal)?;
        let previous_work_item_id = active.replace(work_item_id.to_owned());
        Ok(ActiveWorkItemOutput {
            project_id: project_id.to_owned(),
            previous_work_item_id,
            current_work_item_id: work_item_id.to_owned(),
        })
    }

    fn record_progress(&self, checkpoint: &Value) -> Result<RecordProgressOutput, BackendError> {
        let checkpoint_id = checkpoint
            .get("checkpoint_id")
            .and_then(Value::as_str)
            .filter(|id| !id.trim().is_empty())
            .ok_or(BackendError::InvalidRequest)?;
        let mut checkpoints = self
            .checkpoints
            .lock()
            .map_err(|_| BackendError::Internal)?;
        let duplicate = if let Some(stored) = checkpoints.get(checkpoint_id) {
            if stored != checkpoint {
                return Err(BackendError::Conflict);
            }
            true
        } else {
            checkpoints.insert(checkpoint_id.to_owned(), checkpoint.clone());
            false
        };
        Ok(RecordProgressOutput {
            checkpoint_id: checkpoint_id.to_owned(),
            duplicate,
            deliveries: Self::deliveries(),
        })
    }

    fn get_checkpoint_status(
        &self,
        checkpoint_id: &str,
    ) -> Result<CheckpointStatusOutput, BackendError> {
        let checkpoints = self
            .checkpoints
            .lock()
            .map_err(|_| BackendError::Internal)?;
        let checkpoint = checkpoints
            .get(checkpoint_id)
            .cloned()
            .ok_or(BackendError::CheckpointNotFound)?;
        Ok(CheckpointStatusOutput {
            checkpoint_id: checkpoint_id.to_owned(),
            checkpoint,
            deliveries: Self::deliveries(),
        })
    }
}

#[derive(Clone, Copy, Debug, Error)]
pub enum BackendError {
    #[error("request did not satisfy the Gareji contract")]
    InvalidRequest,
    #[error("project does not grant this operation")]
    PermissionDenied,
    #[error("project was not found or is not visible")]
    ProjectNotFound,
    #[error("checkpoint was not found")]
    CheckpointNotFound,
    #[error("Work item was not found")]
    WorkItemNotFound,
    #[error("Work item state is not eligible for active work")]
    WorkItemNotEligible,
    #[error("Gareji Board is unavailable")]
    BoardUnavailable,
    #[error("execution workspace is not connected")]
    WorkspaceNotConnected,
    #[error("Gareji Core is unavailable")]
    CoreUnavailable,
    #[error("checkpoint identity conflicts with stored content")]
    Conflict,
    #[error("local Gareji operation failed")]
    Internal,
}

impl BackendError {
    pub const fn code(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::PermissionDenied => "permission_denied",
            Self::ProjectNotFound => "project_not_found",
            Self::WorkItemNotFound => "work_item_not_found",
            Self::WorkItemNotEligible => "work_item_not_eligible",
            Self::BoardUnavailable => "board_unavailable",
            Self::WorkspaceNotConnected => "workspace_not_connected",
            Self::CheckpointNotFound => "checkpoint_not_found",
            Self::CoreUnavailable => "core_unavailable",
            Self::Conflict => "conflict",
            Self::Internal => "internal_error",
        }
    }
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::*;

    #[test]
    fn fixture_recording_is_idempotent_and_detects_conflicts() {
        let backend = FixtureBackend::default();
        let checkpoint = json!({"checkpoint_id": "cp-1", "summary": "first"});

        assert!(!backend.record_progress(&checkpoint).unwrap().duplicate);
        assert!(backend.record_progress(&checkpoint).unwrap().duplicate);
        assert!(matches!(
            backend.record_progress(&json!({"checkpoint_id": "cp-1", "summary": "changed"})),
            Err(BackendError::Conflict)
        ));
    }
}
