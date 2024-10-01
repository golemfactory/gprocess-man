use anyhow::Result;
use gprocess_proto::gprocess::api::{WaitRequest, WaitResponse};
use tokio::sync::TryLockError;

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &WaitRequest,
    processes: &ProcessManager,
) -> Result<WaitResponse> {
    match processes.wait(request.pid).await {
        Ok(status) => {
            Ok(WaitResponse { status: Some(status), already_waits: None })
        },
        Err(e) if e.is::<TryLockError>() => {
            Ok(WaitResponse { status: None, already_waits: Some(true) })
        },
        Err(e) => Err(e),
    }
}
