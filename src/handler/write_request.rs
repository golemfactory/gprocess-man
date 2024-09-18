use anyhow::Result;
use gprocess_proto::gprocess::api::{WriteRequest, WriteResponse};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &WriteRequest,
    processes: &ProcessManager,
) -> Result<WriteResponse> {
    let mut w = processes.get_writer(request.pid, request.stream).await?;
    let len = w.write(&request.data).await?.try_into()?;

    Ok(WriteResponse { len })
}
