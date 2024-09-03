use std::collections::HashMap;

use gprocess_proto::gprocess::api;

use crate::process_manager::ProcessManager;

pub async fn handle(
    request: &api::WaitRequest,
    processes: ProcessManager,
) -> anyhow::Result<api::response::Command> {
    let mut w = processes.wait(request.pid).await?;
    todo!()
    /*
    match processes.get_mut(&request.pid) {
        Some(process) => {
            let status = process
                .child
                .wait()
                .expect("Failed to wait for child process");
            api::Response {
                request_id,
                command: Some(api::response::Command::Wait(api::WaitResponse {
                    status: status.code().unwrap_or(-1),
                })),
            }
        }
        None => api::Response {
            request_id,
            command: Some(api::response::Command::Error(api::Error {
                message: "Process not found".to_string(),
            })),
        },
    }

     */
}
