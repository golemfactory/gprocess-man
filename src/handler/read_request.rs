use anyhow::{ensure, Result};
use gprocess_proto::gprocess::api::{ReadRequest, ReadResponse};

use crate::{process_manager::ProcessManager, utils::MAX_PACKET_SIZE};

pub async fn handle(
    request: &ReadRequest,
    processes: &ProcessManager,
) -> Result<ReadResponse> {
    let len = request.len.try_into()?;
    ensure!(len > 0, "invalid length");
    ensure!(len <= MAX_PACKET_SIZE, "length too large");

    let mut r = processes.get_reader(request.pid, request.stream).await?;
    let mut data = vec![0; len];
    let r_size = r.read(&mut data).await?;
    data.truncate(r_size);

    Ok(ReadResponse { data })
}
