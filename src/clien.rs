

use tokio::sync::mpsc;
use serde_json::{json, Value};
type MessageCode = u8;
//Message code
pub const JOIN: MessageCode = 0;
pub const LIVE_LOSS: MessageCode = 1;
pub const TRANSFORM_UPDATE: MessageCode = 2;
pub const SHOOT: MessageCode = 3;
pub const DEATH: MessageCode = 4;
pub const RESPONSE: MessageCode = 5;

#[derive(Debug, serde::Serialize)]
pub struct Message {
    pub message_code: MessageCode,
    pub content: Value,
}

use tokio::task::JoinHandle;
struct NetworkSender(mpsc::Sender<Message>);
struct NetworkReceiver(mpsc::Receiver<Message>);