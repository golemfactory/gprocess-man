use anyhow::{ensure, Result};
use gprocess_proto::gprocess::api::{SignalRequest, SignalResponse};

use crate::{process_manager::ProcessManager, utils::int_to_signal};

pub async fn handle(
    request: &SignalRequest,
    processes: &ProcessManager,
) -> Result<SignalResponse> {
    ensure!(
        processes.process_exists(request.pid).await,
        "pid not found: {}", request.pid,
    );

    nix::sys::signal::kill(
        nix::unistd::Pid::from_raw(request.pid.try_into()?),
        int_to_signal(request.signal),
    )?;

    Ok(SignalResponse {})
}
