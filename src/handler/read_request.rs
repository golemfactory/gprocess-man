use anyhow::{bail, Context, Result};
use gprocess_proto::gprocess::api::{
    response::Command, ReadRequest, ReadResponse,
};

use crate::{process_manager::ProcessManager, utils::MAX_PACKET_SIZE};

pub async fn handle(
    request: &ReadRequest,
    processes: ProcessManager,
) -> Result<Command> {
    let len = request.len.try_into()?;

    if len == 0 {
        bail!("Invalid length");
    }

    if len > MAX_PACKET_SIZE {
        bail!("Length too large");
    }

    let mut r = processes.get_reader(request.pid, request.stream).await?;
    let mut data = vec![0; len];
    let r_size = r.read(&mut data).await.context("read error")?;
    data.truncate(r_size);

    Ok(Command::Read(ReadResponse { data }))
}
