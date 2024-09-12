use anyhow::{bail, Result};
use gprocess_proto::gprocess::api::{
    response::Command, SignalRequest, SignalResponse,
};

use crate::{process_manager::ProcessManager, utils::int_to_signal};

pub async fn handle(
    request: &SignalRequest,
    processes: ProcessManager,
) -> Result<Command> {
    if !processes.process_exists(request.pid).await {
        bail!("pid not found: {}", request.pid);
    }

    let rc = nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(request.pid.try_into()?),
        Some(int_to_signal(request.signal)),
    );

    Ok(Command::Signal(SignalResponse {}))
}
