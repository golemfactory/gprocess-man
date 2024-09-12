use anyhow::Result;
use gprocess_proto::gprocess::api::{
    response::Command, WaitRequest, WaitResponse,
};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &WaitRequest,
    processes: ProcessManager,
) -> Result<Command> {
    let mut status = processes.wait(request.pid).await?;
    Ok(Command::Wait(WaitResponse { status }))
}
