use anyhow::Context;
use gprocess_proto::gprocess::api;
use std::collections::HashMap;
use tracing::{debug, error, info};

use crate::process_manager::ProcessManager;

mod read_request;
mod signal_request;
mod start_request;
mod wait_request;
mod write_request;

pub async fn handle_request_command(
    request_id: u32,
    request: api::request::Command,
    processes: ProcessManager,
) -> anyhow::Result<api::Response> {
    use api::request::Command;

    let command = match request {
        Command::Start(request) => start_request::handle(&request, processes)
            .await
            .context("failed to start process")?,
        Command::Signal(request) => signal_request::handle(&request, processes)
            .await
            .context("failed to signal process")?,
        Command::Wait(request) => wait_request::handle(&request, processes)
            .await
            .context("failed to wait for process")?,
        Command::Read(request) => read_request::handle(&request, processes)
            .await
            .context("failed to process read")?,
        Command::Write(request) => write_request::handle(request, processes)
            .await
            .context("failed to write to process")?,
    };

    Ok(api::Response {
        request_id,
        command: Some(command),
    })
}
