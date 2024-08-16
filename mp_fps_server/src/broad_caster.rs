use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use serde_json::Value;
use tokio::io;
use tokio::net::UdpSocket;
//Message code
pub const JOIN: u8 = 0;
pub const LIVE_LOSS: u8 = 1;
pub const TRANSFORM_UPDATE: u8 = 2;

pub struct Server {
    pub players: Players,
    pub socket: UdpSocket,
}
pub struct Player {
    pub name: String,
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub damage: f32,
    pub is_alive: bool,
}
pub struct Players(Vec<Player>);
impl Server {
    pub async fn new(addr: &str) -> io::Result<Server> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Server {
            players: Players(Vec::new()),
            socket,
        })
    }
}

fn raw_message_to_hashmap(message: &[u8]) -> Result<Value, Error> {
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
