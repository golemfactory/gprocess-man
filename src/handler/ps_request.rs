use anyhow::Result;
use gprocess_proto::gprocess::api::{
    response::Command, PsRequest, PsResponse,
};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &PsRequest,
    processes: ProcessManager,
) -> Result<Command> {
    let pid = processes.ps().await;
    Ok(Command::Ps(PsResponse { pid }))
}
