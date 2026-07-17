use std::env;
use std::io::{BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::backend::{BackendError, GarejiBackend};
use crate::types::{
    ActiveWorkItemOutput, CheckpointStatusOutput, ListProjectsOutput, ProjectContextOutput,
    RecordProgressOutput,
};

const CORE_BRIDGE_PROTOCOL_VERSION: &str = "gareji.core-bridge.v0";
const MAX_BRIDGE_MESSAGE_BYTES: usize = 1_048_576;

/// Production Adapter translating MCP operations to one reusable local Core child process.
pub struct CoreBackend {
    bridge: Arc<dyn CoreBridgeTransport>,
}

impl CoreBackend {
    /// Configure the Adapter from `GAREJI_CORE_BIN` and optional `GAREJI_CORE_DB`.
    #[must_use]
    pub fn from_environment() -> Self {
        let command = env::var_os("GAREJI_CORE_BIN")
            .map_or_else(|| PathBuf::from("gareji-core"), PathBuf::from);
        let database = env::var_os("GAREJI_CORE_DB").map(PathBuf::from);
        Self::new(command, database)
    }

    /// Configure an explicit Core executable and optional database override.
    #[must_use]
    pub fn new(command: PathBuf, database: Option<PathBuf>) -> Self {
        Self {
            bridge: Arc::new(ProcessBridge::new(command, database)),
        }
    }

    #[cfg(test)]
    fn with_transport(bridge: Arc<dyn CoreBridgeTransport>) -> Self {
        Self { bridge }
    }

    fn call<T: DeserializeOwned>(&self, operation: CoreBridgeOperation) -> Result<T, BackendError> {
        let result = self.bridge.call(operation)?;
        serde_json::from_value(result).map_err(|_| BackendError::CoreUnavailable)
    }
}

impl GarejiBackend for CoreBackend {
    fn list_projects(&self) -> Result<ListProjectsOutput, BackendError> {
        self.call(CoreBridgeOperation::ListProjects)
    }

    fn get_project_context(
        &self,
        project_id: &str,
        work_item_id: Option<&str>,
    ) -> Result<ProjectContextOutput, BackendError> {
        self.call(CoreBridgeOperation::GetProjectContext {
            project_id: project_id.to_owned(),
            work_item_id: work_item_id.map(str::to_owned),
        })
    }

    fn set_active_work_item(
        &self,
        project_id: &str,
        work_item_id: &str,
    ) -> Result<ActiveWorkItemOutput, BackendError> {
        self.call(CoreBridgeOperation::SetActiveWorkItem {
            project_id: project_id.to_owned(),
            work_item_id: work_item_id.to_owned(),
        })
    }

    fn record_progress(&self, checkpoint: &Value) -> Result<RecordProgressOutput, BackendError> {
        self.call(CoreBridgeOperation::RecordProgress {
            checkpoint: checkpoint.clone(),
        })
    }

    fn get_checkpoint_status(
        &self,
        checkpoint_id: &str,
    ) -> Result<CheckpointStatusOutput, BackendError> {
        self.call(CoreBridgeOperation::GetCheckpointStatus {
            checkpoint_id: checkpoint_id.to_owned(),
        })
    }
}

trait CoreBridgeTransport: Send + Sync {
    fn call(&self, operation: CoreBridgeOperation) -> Result<Value, BackendError>;
}

struct ProcessBridge {
    command: PathBuf,
    database: Option<PathBuf>,
    process: Mutex<Option<BridgeProcess>>,
    request_counter: AtomicU64,
}

impl ProcessBridge {
    fn new(command: PathBuf, database: Option<PathBuf>) -> Self {
        Self {
            command,
            database,
            process: Mutex::new(None),
            request_counter: AtomicU64::new(1),
        }
    }

    fn spawn(&self) -> Result<BridgeProcess, BackendError> {
        let mut command = Command::new(&self.command);
        if let Some(database) = &self.database {
            command.arg("--database").arg(database);
        }
        let mut child = command
            .arg("bridge")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()
            .map_err(|_| BackendError::CoreUnavailable)?;
        let stdin = child.stdin.take().ok_or(BackendError::CoreUnavailable)?;
        let stdout = child.stdout.take().ok_or(BackendError::CoreUnavailable)?;
        Ok(BridgeProcess {
            child,
            stdin: BufWriter::new(stdin),
            stdout: BufReader::new(stdout),
        })
    }
}

impl CoreBridgeTransport for ProcessBridge {
    fn call(&self, operation: CoreBridgeOperation) -> Result<Value, BackendError> {
        let counter = self.request_counter.fetch_add(1, Ordering::Relaxed);
        let request_id = format!("mcp-{}-{counter}", std::process::id());
        let request = CoreBridgeRequest {
            protocol_version: CORE_BRIDGE_PROTOCOL_VERSION.to_owned(),
            request_id: request_id.clone(),
            operation,
        };
        let mut process = self.process.lock().map_err(|_| BackendError::Internal)?;
        if process.is_none() {
            *process = Some(self.spawn()?);
        }
        let response = process
            .as_mut()
            .ok_or(BackendError::CoreUnavailable)?
            .exchange(&request);
        if response.is_err() {
            process.take();
        }
        let mapped = match response? {
            CoreBridgeResponse::Ok {
                protocol_version,
                request_id: response_id,
                result,
            } if protocol_version == CORE_BRIDGE_PROTOCOL_VERSION && response_id == request_id => {
                Ok(result)
            }
            CoreBridgeResponse::Error {
                protocol_version,
                request_id: response_id,
                error,
            } if protocol_version == CORE_BRIDGE_PROTOCOL_VERSION && response_id == request_id => {
                Err(map_core_error(error.code))
            }
            _ => Err(BackendError::CoreUnavailable),
        };
        if matches!(mapped, Err(BackendError::CoreUnavailable)) {
            process.take();
        }
        mapped
    }
}

struct BridgeProcess {
    child: Child,
    stdin: BufWriter<ChildStdin>,
    stdout: BufReader<ChildStdout>,
}

impl BridgeProcess {
    fn exchange(
        &mut self,
        request: &CoreBridgeRequest,
    ) -> Result<CoreBridgeResponse, BackendError> {
        let payload = serde_json::to_vec(request).map_err(|_| BackendError::Internal)?;
        if payload.len() > MAX_BRIDGE_MESSAGE_BYTES {
            return Err(BackendError::InvalidRequest);
        }
        self.stdin
            .write_all(&payload)
            .and_then(|()| self.stdin.write_all(b"\n"))
            .and_then(|()| self.stdin.flush())
            .map_err(|_| BackendError::CoreUnavailable)?;
        let mut response = Vec::new();
        if !read_bounded_line(&mut self.stdout, &mut response)? {
            return Err(BackendError::CoreUnavailable);
        }
        serde_json::from_slice(&response).map_err(|_| BackendError::CoreUnavailable)
    }
}

impl Drop for BridgeProcess {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn read_bounded_line(
    reader: &mut impl BufRead,
    output: &mut Vec<u8>,
) -> Result<bool, BackendError> {
    output.clear();
    loop {
        let available = reader
            .fill_buf()
            .map_err(|_| BackendError::CoreUnavailable)?;
        if available.is_empty() {
            return Ok(!output.is_empty());
        }
        let newline = available.iter().position(|byte| *byte == b'\n');
        let consumed = newline.map_or(available.len(), |position| position + 1);
        let payload_len = newline.unwrap_or(consumed);
        if output.len().saturating_add(payload_len) > MAX_BRIDGE_MESSAGE_BYTES {
            return Err(BackendError::CoreUnavailable);
        }
        output.extend_from_slice(&available[..payload_len]);
        reader.consume(consumed);
        if newline.is_some() {
            return Ok(true);
        }
    }
}

#[derive(Debug, Serialize)]
struct CoreBridgeRequest {
    protocol_version: String,
    request_id: String,
    #[serde(flatten)]
    operation: CoreBridgeOperation,
}

#[derive(Debug, Serialize)]
#[serde(tag = "operation", content = "payload", rename_all = "snake_case")]
enum CoreBridgeOperation {
    ListProjects,
    GetProjectContext {
        project_id: String,
        work_item_id: Option<String>,
    },
    SetActiveWorkItem {
        project_id: String,
        work_item_id: String,
    },
    RecordProgress {
        checkpoint: Value,
    },
    GetCheckpointStatus {
        checkpoint_id: String,
    },
}

#[derive(Debug, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
enum CoreBridgeResponse {
    Ok {
        protocol_version: String,
        request_id: String,
        result: Value,
    },
    Error {
        protocol_version: String,
        request_id: String,
        error: CoreBridgeError,
    },
}

#[derive(Debug, Deserialize)]
struct CoreBridgeError {
    code: CoreErrorCode,
    #[allow(dead_code)]
    message: String,
}

#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CoreErrorCode {
    InvalidRequest,
    PermissionDenied,
    ProjectNotFound,
    WorkItemNotFound,
    WorkItemNotEligible,
    BoardUnavailable,
    WorkspaceNotConnected,
    CheckpointNotFound,
    Conflict,
    InternalError,
}

const fn map_core_error(code: CoreErrorCode) -> BackendError {
    match code {
        CoreErrorCode::InvalidRequest => BackendError::InvalidRequest,
        CoreErrorCode::PermissionDenied => BackendError::PermissionDenied,
        CoreErrorCode::ProjectNotFound => BackendError::ProjectNotFound,
        CoreErrorCode::WorkItemNotFound => BackendError::WorkItemNotFound,
        CoreErrorCode::WorkItemNotEligible => BackendError::WorkItemNotEligible,
        CoreErrorCode::BoardUnavailable => BackendError::BoardUnavailable,
        CoreErrorCode::WorkspaceNotConnected => BackendError::WorkspaceNotConnected,
        CoreErrorCode::CheckpointNotFound => BackendError::CheckpointNotFound,
        CoreErrorCode::Conflict => BackendError::Conflict,
        CoreErrorCode::InternalError => BackendError::Internal,
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;
    use std::io::Cursor;

    use serde_json::json;

    use super::*;

    struct MemoryBridge {
        responses: Mutex<VecDeque<Result<Value, BackendError>>>,
    }

    impl CoreBridgeTransport for MemoryBridge {
        fn call(&self, _operation: CoreBridgeOperation) -> Result<Value, BackendError> {
            self.responses
                .lock()
                .map_err(|_| BackendError::Internal)?
                .pop_front()
                .ok_or(BackendError::Internal)?
        }
    }

    #[test]
    fn core_backend_decodes_live_project_results_through_its_transport_seam() {
        let backend = CoreBackend::with_transport(Arc::new(MemoryBridge {
            responses: Mutex::new(VecDeque::from([Ok(json!({
                "projects": [{
                    "id": "core",
                    "name": "Gareji Core",
                    "execution_workspace": "core-local",
                    "context_sources": ["zettelkasten"],
                    "grants": ["read_context"]
                }]
            }))])),
        }));

        let result = backend.list_projects().unwrap();
        assert_eq!(result.projects[0].id, "core");
    }

    #[test]
    fn missing_core_binary_is_reported_as_core_unavailable() {
        let backend = CoreBackend::new(
            PathBuf::from("gareji-core-binary-that-does-not-exist"),
            None,
        );
        assert!(matches!(
            backend.list_projects(),
            Err(BackendError::CoreUnavailable)
        ));
    }

    #[test]
    fn core_response_reader_rejects_oversized_messages() {
        let mut input = Cursor::new(vec![b'x'; MAX_BRIDGE_MESSAGE_BYTES + 1]);
        let mut output = Vec::new();
        assert!(matches!(
            read_bounded_line(&mut input, &mut output),
            Err(BackendError::CoreUnavailable)
        ));
    }
}
