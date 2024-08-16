#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(dead_code)]

use std::net::SocketAddr;

use clap::{arg, command, value_parser};
use network::process_connection;
use shadow_rs::shadow;
use tokio::sync::mpsc::{self};
use tracing::{info, trace};

use command::QueueCommand;
use gprocess_proto::gprocess::api;
use gprocess_proto::gprocess::api::Response;
use handler::handle_request_command;
use utils::{init_tracing, print_version};

shadow!(build);

mod command;
mod handler;
mod network;
mod process_manager;
mod utils;

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

    let processes = process_manager::create();

    let (queue_tx, mut queue_rx) = mpsc::channel::<QueueCommand>(32);

    tokio::spawn(async move {
        // process commands queue
        while let Some(command) = queue_rx.recv().await {
            match command {
                QueueCommand::Reaper => {
                    //
                }
                QueueCommand::Command(id, request, response_tx) => {
                    let processes = processes.clone();
                    let h = tokio::spawn(async move {
                        let response = handle_request_command(id, request, processes);
                        let rc = tokio::select! {
                            r = response => {
                                r
                            },
                            response_tx = response_tx.closed() => {
                                tracing::error!("Response channel closed");
                                return;
                            }
                        };

                        let response = match rc {
                            Ok(response) => response,
                            Err(e) => {
                                tracing::error!("late response: {}", e);
                                api::Response {
                                    request_id: id,
                                    command: None,
                                }
                            }
                        };

                        if let Err(e) = response_tx.send(response).await {
                            tracing::error!("Error sending response: {}", e);
                        }
                    });
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
        let (stream, addr) = listener.accept().await?;

        trace!("Accepted connection from {:?}", addr);

        let queue_tx = queue_tx.clone();

        tokio::spawn(async move {
            trace!("Processing connection from {:?}", addr);

            let rc = process_connection(stream, queue_tx).await;

            if let Err(e) = rc {
                tracing::error!("Error processing connection from {:?}: {}", addr, e);
            }
        });
    }

    // info!("Shutting down");
}
