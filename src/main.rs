use std::{collections::HashMap, net::SocketAddr, process::Child};

use clap::{arg, command, value_parser};
use network::process_connection;
use shadow_rs::shadow;
use tokio::sync::mpsc::{self};
use tracing::{info, trace};

use command::QueueCommand;
use handler::{handle_request_command, reap_processes};
use utils::{init_tracing, print_version};

shadow!(build);

mod command;
mod handler;
mod network;
mod utils;

struct ChildInfo {
    child: Child,
    stdin: Option<std::process::ChildStdin>,
    stdout: Option<std::process::ChildStdout>,
    stderr: Option<std::process::ChildStderr>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    init_tracing();

    let matches = command!()
        .arg(
            arg!(-l --listen <INTERFACE> "Interface to listen on")
                .required(false)
                .default_value("0.0.0.0:1234")
                .value_parser(value_parser!(SocketAddr)),
        )
        .get_matches();

    print_version();

    let interface = matches
        .get_one::<SocketAddr>("listen")
        .expect("Failed to parse interface");

    let listener = tokio::net::TcpListener::bind(interface).await?;

    info!("Listening on {:?}", interface);

    let mut processes: HashMap<u64, ChildInfo> = HashMap::new();

    let (queue_tx, mut queue_rx) = mpsc::channel::<QueueCommand>(32);

    tokio::spawn(async move {
        // process commands queue
        while let Some(command) = queue_rx.recv().await {
            match command {
                QueueCommand::Reaper => {
                    reap_processes(&mut processes).await;
                }
                QueueCommand::Command(id, request, response_tx) => {
                    let response = handle_request_command(id, &request, &mut processes).await;
                    response_tx.send(response).unwrap();
                }
            }
        }
    });

    let reaper_queue_tx = queue_tx.clone();

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
            reaper_queue_tx.send(QueueCommand::Reaper).await.unwrap();
        }
    });

    loop {
        trace!("Waiting for connection");
        let (mut stream, addr) = listener.accept().await?;

        trace!("Accepted connection from {:?}", addr);

        let queue_tx = queue_tx.clone();

        tokio::spawn(async move {
            trace!("Processing connection from {:?}", addr);

            let rc = process_connection(&mut stream, queue_tx).await;

            if let Err(e) = rc {
                tracing::error!("Error processing connection from {:?}: {}", addr, e);
            }
        });
    }

    // info!("Shutting down");
}
