use std::{collections::HashMap, io::Write, os::fd::AsRawFd};

use api::response::Command;
use gprocess_proto::gprocess::api;

use crate::ChildInfo;

pub async fn handle(
    request_id: u32,
    request: &api::WriteRequest,
    processes: &mut HashMap<u64, ChildInfo>,
) -> api::Response {
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

    let fd = request.stream;
    let stdin_fd = process.stdin.as_ref().map(|x| x.as_raw_fd()).unwrap_or(-1);

    if stdin_fd == -1 {
        return api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: "Invalid stdin fd".to_string(),
            })),
        };
    }

    if fd != stdin_fd {
        return api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: "Invalid stdin fd".to_string(),
            })),
        };
    }

    let len = process
        .stdin
        .as_mut()
        .expect("Failed to get stdin stream")
        .write(request.data.as_slice())
        .expect("Failed to write to stdin stream") as u32;

    api::Response {
        request_id,
        command: Some(Command::Write(api::WriteResponse { len })),
    }
}
