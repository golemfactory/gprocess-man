use std::process::Stdio;

use gprocess_proto::gprocess::api;
use nix::sys::signal::Signal;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::build;

pub const MAX_PACKET_SIZE: usize = 1024 * 1024;

pub fn init_tracing() {
    let filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .or_else(|_| tracing_subscriber::EnvFilter::try_new("trace"))
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

pub fn print_version() {
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

pub fn int_to_stream_type(stream: Option<i32>) -> api::Stream {
    match stream {
        Some(stream) => api::Stream::try_from(stream).expect("Invalid stream type"),
        None => api::Stream::Inherit,
    }
}

pub fn stream_type_to_stdio(stdin: api::Stream) -> Stdio {
    match stdin {
        api::Stream::Inherit => Stdio::inherit(),
        api::Stream::Null => Stdio::null(),
        api::Stream::Pipe => Stdio::piped(),
    }
}

pub fn int_to_signal(signal: i32) -> Signal {
    match signal {
        1 => Signal::SIGHUP,
        2 => Signal::SIGINT,
        3 => Signal::SIGQUIT,
        4 => Signal::SIGILL,
        5 => Signal::SIGTRAP,
        6 => Signal::SIGABRT,
        7 => Signal::SIGBUS,
        8 => Signal::SIGFPE,
        9 => Signal::SIGKILL,
        10 => Signal::SIGUSR1,
        11 => Signal::SIGSEGV,
        12 => Signal::SIGUSR2,
        13 => Signal::SIGPIPE,
        14 => Signal::SIGALRM,
        15 => Signal::SIGTERM,
        17 => Signal::SIGCHLD,
        18 => Signal::SIGCONT,
        19 => Signal::SIGSTOP,
        20 => Signal::SIGTSTP,
        21 => Signal::SIGTTIN,
        22 => Signal::SIGTTOU,
        23 => Signal::SIGURG,
        24 => Signal::SIGXCPU,
        25 => Signal::SIGXFSZ,
        26 => Signal::SIGVTALRM,
        27 => Signal::SIGPROF,
        28 => Signal::SIGWINCH,
        29 => Signal::SIGIO,
        31 => Signal::SIGSYS,
        _ => Signal::SIGKILL,
    }
}

// pub async fn shutdown_signal() {
//     let ctrl_c = async {
//         signal::ctrl_c()
//             .await
//             .expect("Failed to install Ctrl+C handler");
//     };

//     #[cfg(unix)]
//     let terminate = async {
//         signal::unix::signal(signal::unix::SignalKind::terminate())
//             .expect("Failed to install signal handler")
//             .recv()
//             .await;
//     };

//     #[cfg(not(unix))]
//     let terminate = std::future::pending::<()>();

//     tokio::select! {
//         _ = ctrl_c => {},
//         _ = terminate => {},
//     }
// }
