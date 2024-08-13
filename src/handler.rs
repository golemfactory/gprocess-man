use std::collections::HashMap;

use gprocess_proto::gprocess::api;
use tracing::{debug, error, info};

use crate::ChildInfo;

mod read_request;
mod signal_request;
mod start_request;
mod wait_request;
mod write_request;

pub async fn reap_processes(processes: &mut HashMap<u64, ChildInfo>) {
    let mut to_remove = Vec::new();

    for (pid, process) in processes.iter_mut() {
        match process.child.try_wait() {
            Ok(Some(status)) => {
                info!("Process {} exited with: {}", pid, status);
                to_remove.push(*pid);
            }
            Ok(None) => {
                debug!("Process {} still running", pid);
            }
            Err(e) => {
                error!("Error attempting to wait: {}", e);
            }
        }
    }

    for pid in to_remove {
        processes.remove(&pid);
    }
}

pub async fn handle_request_command(
    request_id: u32,
    request: &api::request::Command,
    processes: &mut HashMap<u64, ChildInfo>,
) -> api::Response {
    use api::request::Command;

    match request {
        Command::Start(request) => start_request::handle(request_id, request, processes).await,
        Command::Signal(request) => signal_request::handle(request_id, request, processes).await,
        Command::Wait(request) => wait_request::handle(request_id, request, processes).await,
        Command::Read(request) => read_request::handle(request_id, request, processes).await,
        Command::Write(request) => write_request::handle(request_id, request, processes).await,
    }
}
