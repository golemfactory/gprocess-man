use anyhow::{anyhow, bail, Context};
use api::response::Command;
use gprocess_proto::gprocess::api;
use std::{collections::HashMap, io::Read, os::fd::AsRawFd};

use crate::process_manager::ProcessManager;
use crate::utils::MAX_PACKET_SIZE;

pub async fn handle(
    request: &api::ReadRequest,
    processes: ProcessManager,
) -> anyhow::Result<api::response::Command> {
    if request.len == 0 {
        bail!("Invalid length");
    }

    if request.len > MAX_PACKET_SIZE as u32 {
        bail!("Length too large");
    }

    let mut r = processes.get_reader(request.pid, request.stream).await?;
    let mut buf = vec![0; request.len as usize];
    let r_size = r.read(&mut buf).await.context("read error")?;

    buf.truncate(r_size);

    let read_response = api::ReadResponse {
        len: r_size as u32,
        data: buf,
    };

    Ok(Command::Read(read_response))
}

#[cfg(test)]
mod test {
    use std::sync::{Arc, Mutex};
    use std::time::Duration;

    #[tokio::test]
    async fn test_unwrap() {
        let m = Arc::new(Mutex::new(()));

        let h = {
            let m = m.clone();
            tokio::spawn(async move {
                let _g = m.lock().unwrap();
                panic!("test");
            })
        };

        tokio::time::sleep(Duration::from_secs(1)).await;
        let _g = m.lock().unwrap();
        if let Err(e) = h.await {
            eprintln!("err={:?}", e);
        }
    }
}
