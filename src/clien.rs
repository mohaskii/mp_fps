

use tokio::sync::mpsc;
use serde_json::{json, Value};
use bevy:: prelude::*;
type MessageCode = u8;
//Message code
pub const JOIN: MessageCode = 0;
pub const LIVE_LOSS: MessageCode = 1;
pub const TRANSFORM_UPDATE: MessageCode = 2;
pub const SHOOT: MessageCode = 3;
pub const DEATH: MessageCode = 4;
pub const RESPONSE: MessageCode = 5;

pub struct ClientPlugin;

impl Plugin for ClientPlugin {
    fn build(&self, app: &mut App) {
       todo!();
    }
}


#[derive(Debug, serde::Serialize)]
pub struct Message {
    pub message_code: MessageCode,
    pub content: Value,
}

use tokio::task::JoinHandle;
#[derive(Resource,)]
struct NetworkSender(mpsc::Sender<Message>);
#[derive(Resource)]
struct NetworkReceiver(mpsc::Receiver<Message>);
fn network_update(
    mut receiver: ResMut<NetworkReceiver>,
    mut query: Query<(&Player, &mut Transform)>,
) {
    while let Ok(msg) = receiver.0.try_recv() {
        // Processer les messages reçus du réseau
    }
}

// Fonction pour envoyer les mises à jour du joueur
fn send_player_update(
    sender: Res<NetworkSender>,
    query: Query<(&Player, &Transform)>,
) {
    // Envoyer les mises à jour du joueur au réseau
}