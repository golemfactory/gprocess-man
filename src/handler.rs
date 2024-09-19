use anyhow::{Context, Result};
use gprocess_proto::gprocess::api::{
    request::Command as Request,
    response::Command as Response,
};

use crate::process_manager::ProcessManager;

mod close_request;
mod ps_request;
mod read_request;
mod signal_request;
mod start_request;
mod wait_request;
mod write_request;

pub async fn handle_request_command(
    request: &Request,
    processes: &ProcessManager,
) -> Result<Response> {
    match request {
        Request::Start(request) => {
            start_request::handle(request, processes)
                .await
                .map(Response::Start)
                .context("failed to start process")
        }
        Request::Signal(request) => {
            signal_request::handle(request, processes)
                .await
                .map(Response::Signal)
                .with_context(|| format!("failed to signal process: {}", request.pid))
        }
        Request::Wait(request) => {
            wait_request::handle(request, processes)
                .await
                .map(Response::Wait)
                .with_context(|| format!("failed to wait for process: {}", request.pid))
        }
        Request::Read(request) => {
            read_request::handle(request, processes)
                .await
                .map(Response::Read)
                .with_context(|| format!("failed to read from process: {}", request.pid))
        }
        Request::Write(request) => {
            write_request::handle(request, processes)
                .await
                .map(Response::Write)
                .with_context(|| format!("failed to write to process: {}", request.pid))
        }
        Request::Ps(request) => {
            ps_request::handle(request, processes)
                .await
                .map(Response::Ps)
                .context("failed to list processes")
        }
        Request::Close(request) => {
            close_request::handle(request, processes)
                .await
                .map(Response::Close)
                .with_context(|| format!("failed to close process: {}", request.pid))
        }
    }
}
