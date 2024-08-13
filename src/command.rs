use gprocess_proto::gprocess::api;
use tokio::sync::oneshot;

#[derive(Debug)]
pub enum QueueCommand {
    Reaper,
    Command(u32, api::request::Command, oneshot::Sender<api::Response>),
}
