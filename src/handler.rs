use std::collections::HashMap;
use anyhow::Context;
use gprocess_proto::gprocess::api;
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

    Ok(match request {
        Command::Start(request) => start_request::handle(request_id, &request, processes).await?,
        Command::Signal(request) => signal_request::handle(request_id, &request, processes).await,
        Command::Wait(request) => wait_request::handle(request_id, &request, processes).await,
        Command::Read(request) => read_request::handle(request_id, &request, processes).await.context("failed to process read")?,
        Command::Write(request) => write_request::handle(request_id, request, processes).await?,
    })
}
