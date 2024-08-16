use std::collections::HashMap;

use gprocess_proto::gprocess::api;

use crate::process_manager::ProcessManager;
use crate::utils::int_to_signal;

pub async fn handle(
    request: &api::SignalRequest,
    processes: ProcessManager,
) -> anyhow::Result<api::response::Command> {
    todo!()

    /*use api::response::Command;

    let process = match processes.get_mut(&request.pid) {
        Some(process) => process,
        None => {
            return api::Response {
                request_id,
                command: Some(Command::Error(api::Error {
                    message: "Process not found".to_string(),
                })),
            };
        }
    };

    let rc = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(process.child.id() as i32),
        Some(int_to_signal(request.signal)),
    );

    match rc {
        Ok(_) => api::Response {
            request_id,
            command: Some(Command::Signal(api::SignalResponse {})),
        },
        Err(e) => api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: format!("Failed to send signal: {}", e),
            })),
        },
    }

     */
}
