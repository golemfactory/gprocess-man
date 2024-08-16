use anyhow::{bail, Result};
use gprocess_proto::gprocess::api;
use prost::Message;
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{
        tcp::{OwnedReadHalf, OwnedWriteHalf},
        TcpStream,
    },
    sync::{
        mpsc::{self, Sender},
        oneshot,
    },
    task::{self, LocalSet},
};
use tracing::{debug, error, trace};

use crate::{command::QueueCommand, utils::MAX_PACKET_SIZE};

pub async fn read_request(stream: &mut OwnedReadHalf) -> Result<Option<api::Request>> {
    let size = stream.read_u32().await;

    if size.is_err() {
        // Connection closed?!: Connection reset by peer (os error 54)
        error!("Error reading packet size: {}", size.err().unwrap());
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

pub async fn write_response(stream: &mut OwnedWriteHalf, response: &api::Response) -> Result<()> {
    stream.write_u32(response.encoded_len() as u32).await?;
    stream.write_all(&response.encode_to_vec()).await?;

    Ok(())
}

pub async fn process_connection(stream: TcpStream, queue_tx: Sender<QueueCommand>) -> Result<()> {
    let (mut reader, mut writer) = stream.into_split();

    let (write_tx, mut write_rx) = mpsc::channel::<api::Response>(32);

    let writer = async move {
        while let Some(response) = write_rx.recv().await {
            debug!("Writing response: {:?}", response);
            if let Err(e) = write_response(&mut writer, &response).await {
                error!("Error writing response: {}", e);
            }
        }
        trace!("Write channel closed");
        Ok(())
    };

    let reader = async move {
        loop {
            let request = match read_request(&mut reader).await? {
                Some(request) => request,
                None => {
                    return anyhow::Ok(());
                }
            };

            match request.command {
                Some(command) => {
                    queue_tx
                        .send(QueueCommand::Command(
                            request.request_id,
                            command,
                            write_tx.clone(),
                        ))
                        .await?;
                }
                None => {
                    error!("Missing command");
                    continue;
                }
            }
        }
    };

    tokio::try_join!(reader, writer)?;

    trace!("Connection closed");
    anyhow::Ok(())
}
