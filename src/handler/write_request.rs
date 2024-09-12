use anyhow::{Context, Result};
use gprocess_proto::gprocess::api::{
    response::Command, WriteRequest, WriteResponse,
};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: WriteRequest,
    processes: ProcessManager,
) -> Result<Command> {
    let mut w = processes.get_writer(request.pid, request.stream).await?;
    let len = w.write(&request.data).await.context("write error")?.try_into()?;

    Ok(Command::Write(WriteResponse { len }))
}
