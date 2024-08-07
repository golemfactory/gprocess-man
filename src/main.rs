use clap::{arg, command, value_parser};
use shadow_rs::shadow;
use tracing::info;
use tracing_subscriber::prelude::*;

shadow!(build);

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

#[tokio::main]
async fn main() {
    init_tracing();

    let matches = command!()
        .arg(
            arg!(-l --listen <INTERFACE> "Interface to listen on")
                .required(false)
                .value_parser(value_parser!(String)),
        )
        .get_matches();

    print_version();

    let interface = matches
        .get_one::<String>("listen")
        .unwrap_or(&"0.0.0.0:1234".to_string())
        .to_string();

    info!("Listening on {}", interface);
}
