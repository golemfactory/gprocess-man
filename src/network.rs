use anyhow::{bail, Result};
use gprocess_proto::gprocess::api;
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpStream,
    sync::{mpsc::Sender, oneshot},
};
use tracing::error;

use crate::{command::QueueCommand, utils::MAX_PACKET_SIZE};

pub async fn read_request(stream: &mut TcpStream) -> Result<Option<api::Request>> {
    let size = stream.read_u32().await;

    if size.is_err() {
        // Connection closed?!: Connection reset by peer (os error 54)
        return Ok(None);
    };

    let size = size.unwrap() as usize;

    if size > MAX_PACKET_SIZE {
        bail!("Packet too large {}", size);
    }

    if size == 0 {
        bail!("Empty packet");
    }

    let mut buf = vec![0; size];
    let read = tokio::select! {
        r = stream.read_exact(&mut buf) => {
            r?
        },
        _ = tokio::time::sleep(tokio::time::Duration::from_secs(3)) => {
            bail!("Timeout reading packet")
        }
    };

    if read == 0 {
        // Connection closed
        return Ok(None);
    }

    Ok(Some(api::Request::decode(buf.as_slice())?))
}

pub async fn write_response(stream: &mut TcpStream, response: &api::Response) -> Result<()> {
    stream.write_u32(response.encoded_len() as u32).await?;
    stream.write_all(&response.encode_to_vec()).await?;

    Ok(())
}

pub async fn process_connection(
    stream: &mut TcpStream,
    queue_tx: Sender<QueueCommand>,
) -> Result<()> {
    loop {
        let request = match read_request(stream).await? {
            Some(request) => request,
            None => {
                return Ok(());
            }
        };

        match request.command {
            Some(command) => {
                let (response_tx, response_rx) = oneshot::channel::<api::Response>();

                queue_tx
                    .send(QueueCommand::Command(
                        request.request_id,
                        command,
                        response_tx,
                    ))
                    .await?;

                let response = response_rx.await?;
                write_response(stream, &response).await?;
            }
            None => {
                error!("Missing command");
                continue;
            }
        }
    }
}
