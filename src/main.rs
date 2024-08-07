use axum::Router;
use clap::{arg, command, value_parser};
use shadow_rs::shadow;
use std::ffi::OsStr;
use std::net::SocketAddr;
use std::path::PathBuf;
use tokio::signal;
use tracing::info;
use tracing_subscriber::prelude::*;

shadow!(build);

mod app_state;
mod routes;

fn init_tracing() {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::Registry::default()
        .with(filter_layer)
        .with(
            tracing_subscriber::fmt::layer()
                .with_target(false)
                .without_time(),
        )
        .init();
}

fn print_version() {
    info!(
        "{} {} ({}, {}, git: {}, {}, {}{})",
        build::PROJECT_NAME,
        build::PKG_VERSION,
        if shadow_rs::is_debug() {
            "debug"
        } else {
            "release"
        },
        build::BUILD_OS,
        build::SHORT_COMMIT,
        if build::BRANCH.is_empty() {
            "no branch"
        } else {
            build::BRANCH
        },
        if build::TAG.is_empty() {
            "no tag"
        } else {
            build::TAG
        },
        if !build::GIT_CLEAN { ", dirty" } else { "" }
    );
}

async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() {
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

    let interface = matches.get_one::<SocketAddr>("listen").unwrap();

    info!("Listening on {:?}", interface);

    let listener = tokio::net::TcpListener::bind(interface).await.unwrap();

    let state = app_state::AppState {};

    let app = Router::new()
        .nest("/api/v1", routes::api::v1::get_routes())
        .with_state(state.clone());

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await
        .expect("Failed to start server");

    info!("Shutting down");
}
