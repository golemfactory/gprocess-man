use std::{
    collections::HashMap,
    ffi::OsStr,
    os::{
        fd::AsRawFd,
        unix::{ffi::OsStrExt, process::CommandExt},
    },
    process::Command,
};

use gprocess_proto::gprocess::api;

use crate::{
    utils::{int_to_stream_type, stream_type_to_stdio},
    ChildInfo,
};

pub async fn handle(
    request_id: u32,
    request: &api::StartRequest,
    processes: &mut HashMap<u64, ChildInfo>,
) -> api::Response {
    let mut command = Command::new(request.program.clone());

    for arg in request.args.iter() {
        command.arg(OsStr::from_bytes(arg.as_slice()));
    }

    if let Some(uid) = request.uid {
        command.uid(uid);
    }

    if let Some(gid) = request.gid {
        command.gid(gid);
    }

    if let Some(cwd) = &request.work_dir {
        command.current_dir(OsStr::from_bytes(cwd.as_slice()));
    }

    if let Some(env_clear) = request.env_clear {
        if env_clear {
            command.env_clear();
        }
    }

    for env in request.env.iter() {
        let key = String::from_utf8(env.name.clone())
            .expect("Failed to convert environment variable name to string");
        match env.value.clone() {
            Some(value) => {
                let value = String::from_utf8(value)
                    .expect("Failed to convert environment variable value to string");
                command.env(key, value);
            }
            None => {
                command.env_remove(key);
            }
        };
    }

    let stdin = int_to_stream_type(request.stdin);
    let stdout = int_to_stream_type(request.stdout);
    let stderr = int_to_stream_type(request.stderr);
    command.stdin(stream_type_to_stdio(stdin));
    command.stdout(stream_type_to_stdio(stdout));
    command.stderr(stream_type_to_stdio(stderr));

    let mut spawned = command.spawn().expect("Failed to start process");

    let pid = spawned.id();
    let stdin = spawned.stdin.take();
    let stdout = spawned.stdout.take();
    let stderr = spawned.stderr.take();

    let stdin_fd = stdin.as_ref().map(|x| x.as_raw_fd());
    let stdout_fd = stdout.as_ref().map(|x| x.as_raw_fd());
    let stderr_fd = stderr.as_ref().map(|x| x.as_raw_fd());

    let child_info = ChildInfo {
        child: spawned,
        stdin,
        stdout,
        stderr,
    };

    processes.insert(pid as u64, child_info);

    let start_response = api::StartResponse {
        pid: pid as u64,
        stdin: stdin_fd,
        stdout: stdout_fd,
        stderr: stderr_fd,
    };

    api::Response {
        request_id,
        command: Some(api::response::Command::Start(start_response)),
    }
}
