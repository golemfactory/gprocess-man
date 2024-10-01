use anyhow::Result;
use gprocess_proto::gprocess::api::{PsRequest, PsResponse};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &PsRequest,
    processes: &ProcessManager,
) -> Result<PsResponse> {
    let pid = processes.ps().await;
    Ok(PsResponse { pid })
}
