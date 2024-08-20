
use std::io::{Error, ErrorKind};

use tokio::sync::mpsc;
use tokio::runtime::Runtime;
pub use bevy::prelude::*;
use serde_json::{json, Value};
use bevy::prelude::*;

use crate::Player;
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
        todo!()
    }
}
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Message {
    pub message_code: MessageCode,
    pub content: Value,
}
impl  Message {
    pub fn frome_bytes(message: &[u8]) -> Result<Message , Error> {
        // Convertir les bytes en une chaîne de caractères
        let message_str = match std::str::from_utf8(message) {
            Ok(v) => v,
            Err(e) => return Err(Error::new(ErrorKind::InvalidData, e)), // En cas d'erreur de conversion
        };

        // Convertir la chaîne JSON en une valeur JSON (Value)
        match serde_json::from_str(message_str) {
            Ok(v) => Ok(v),
            Err(e) => return Err(Error::from(e)), // En cas d'erreur de parsing JSON
        }

        // Convertir le Value en HashMap<String, String>
    }
}
#[derive(Resource,)]
pub struct NetworkSender(pub mpsc::Sender<Message>);
#[derive(Resource,)]
pub struct NetworkReceiver( pub mpsc::Receiver<Message>);

fn setup(mut commands: Commands) {
    // Ajoutez ici les entités du jeu et les configurations supplémentaires
}

// Fonction pour mettre à jour le réseau
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