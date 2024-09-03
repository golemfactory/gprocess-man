use std::{collections::HashMap, io::Read, os::fd::AsRawFd};

use api::response::Command;
use gprocess_proto::gprocess::api;

use crate::{utils::MAX_PACKET_SIZE, ChildInfo};

pub async fn handle(
    request_id: u32,
    request: &api::ReadRequest,
    processes: &mut HashMap<u64, ChildInfo>,
) -> api::Response {
    if request.len == 0 {
        return api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: "Invalid length".to_string(),
            })),
        };
    }

    if request.len > MAX_PACKET_SIZE as u32 {
        return api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: "Length too large".to_string(),
            })),
        };
    }

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
    let stdout_fd = process.stdout.as_ref().map(|x| x.as_raw_fd()).unwrap_or(-1);
    let stderr_fd = process.stderr.as_ref().map(|x| x.as_raw_fd()).unwrap_or(-1);

    let mut buf = vec![0; request.len as usize];
    let read;

    if fd == stdout_fd {
        read = process
            .stdout
            .as_mut()
            .expect("Failed to get stdout stream")
            .read(&mut buf)
            .expect("Failed to read from stdout stream");
    } else if fd == stderr_fd {
        read = process
            .stderr
            .as_mut()
            .expect("Failed to get stderr stream")
            .read(&mut buf)
            .expect("Failed to read from stderr stream");
    } else {
        return api::Response {
            request_id,
            command: Some(Command::Error(api::Error {
                message: "Invalid fd".to_string(),
            })),
        };
    }

    buf.truncate(read);

    let read_response = api::ReadResponse {
        len: read as u32,
        data: buf,
    };

    api::Response {
        request_id,
        command: Some(Command::Read(read_response)),
    }
}
