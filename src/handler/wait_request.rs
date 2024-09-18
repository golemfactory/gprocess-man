use anyhow::Result;
use gprocess_proto::gprocess::api::{WaitRequest, WaitResponse};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &WaitRequest,
    processes: &ProcessManager,
) -> Result<WaitResponse> {
    let status = processes.wait(request.pid).await?;
    Ok(WaitResponse { status })
}
