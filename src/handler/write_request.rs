use anyhow::Context;
use anyhow::Result;
use api::response::Command;
use gprocess_proto::gprocess::api;
use std::{collections::HashMap, io::Write, os::fd::AsRawFd};

use crate::process_manager::ProcessManager;

pub async fn handle(
    request_id: u32,
    request: api::WriteRequest,
    processes: ProcessManager,
) -> Result<api::Response> {
    let mut w = processes.get_writer(request.pid, request.stream)?;
    let data = request.data;
    let len = tokio::task::spawn_blocking(move || w.write(data.as_slice()))
        .await
        .context("write error")?? as u32;

    Ok(api::Response {
        request_id,
        command: Some(Command::Write(api::WriteResponse { len })),
    })
}
