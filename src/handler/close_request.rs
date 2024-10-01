use anyhow::Result;
use gprocess_proto::gprocess::api::{CloseRequest, CloseResponse};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &CloseRequest,
    processes: &ProcessManager,
) -> Result<CloseResponse> {
    processes.remove(request.pid).await;
    Ok(CloseResponse {})
}
