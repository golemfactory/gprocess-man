use anyhow::Context;
use gprocess_proto::gprocess::api;
use std::{
    collections::HashMap,
    ffi::OsStr,
    os::{
        fd::AsRawFd,
        unix::{ffi::OsStrExt, process::CommandExt},
    },
    process::Command,
};

use crate::process_manager::ProcessManager;
use crate::utils::{int_to_stream_type, stream_type_to_stdio};

pub async fn handle(
    request: &api::StartRequest,
    processes: ProcessManager,
) -> anyhow::Result<api::response::Command> {
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
        let key = OsStr::from_bytes(&env.name);

        match env.value.clone() {
            Some(value) => {
                command.env(key, OsStr::from_bytes(&value));
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

    let mut spawned = command
        .spawn()
        .with_context(|| format!("failed to run {:?}", request.program))?;

    let stdin = spawned.stdin.as_ref().map(|_| 0);
    let stdout = spawned.stdout.as_ref().map(|_| 1);
    let stderr = spawned.stderr.as_ref().map(|_| 2);

    let pid = processes.add_process(spawned)?;

    let start_response = api::StartResponse {
        pid,
        stdin,
        stdout,
        stderr,
    };

    Ok(api::response::Command::Start(start_response))
}
