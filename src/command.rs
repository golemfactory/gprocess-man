use gprocess_proto::gprocess::api;
use tokio::sync::{mpsc, oneshot};

#[derive(Debug)]
pub enum QueueCommand {
    Reaper,
    Command(u32, api::request::Command, mpsc::Sender<api::Response>),
}
