use std::collections::HashMap;
use std::io::{Error, ErrorKind};

use serde_json::{json, Value};
use std::net::SocketAddr;
use tokio::io;
use tokio::net::UdpSocket;
type MessageCode = u8;
//Message code
pub const JOIN: MessageCode = 0;
pub const LIVE_LOSS: MessageCode = 1;
pub const TRANSFORM_UPDATE: MessageCode = 2;
pub const SHOOT: MessageCode = 3;
pub const DEATH: MessageCode = 4;
pub const RESPONSE: MessageCode = 5;

pub struct Server {
    pub players: Players,
    pub socket: UdpSocket,
}
#[derive(Debug, serde::Serialize)]
pub struct Message {
    pub message_code: MessageCode,
    pub content: Value,
}

pub struct Player {
    pub address: SocketAddr,
    pub name: String,
    pub is_alive: bool,
}
pub struct Players(Vec<Player>);
impl Players {
    fn get(&mut self, name: &str) -> Option<&mut Player> {
        for player in self.0.iter_mut() {
            if player.name == name {
                return Some(player);
            }
        }
        None
    }
}
impl Server {
    pub async fn new(addr: &str) -> io::Result<Server> {
        let socket = UdpSocket::bind(addr).await?;
        Ok(Server {
            players: Players(Vec::new()),
            socket,
        })
    }
     async fn broacast(&self, json_message: Value) -> io::Result<()> {
        let message = serde_json::to_string(&json_message).unwrap();
        let message = message.as_bytes();
        for player in &self.players.0 {
            self.socket.send_to(message, player.address).await?;
        }
        Ok(())
    }
    pub async fn broadcast_except(&self, json_message: Value, addr: SocketAddr) -> io::Result<()> {
        let message = serde_json::to_string(&json_message).unwrap();
        let message = message.as_bytes();
        for player in &self.players.0 {
            if player.address != addr {
                self.socket.send_to(message, player.address).await?;
            }
        }
        Ok(())
    }
    pub async fn send_message(&self, json_message: Value, addr: SocketAddr) -> io::Result<()> {
        let message = serde_json::to_string(&json_message).unwrap();
        let message = message.as_bytes();
        match self.socket.send_to(message, addr).await {
            Ok(_) => Ok(()),
            Err(e) => Err(e),
        }
    }

    pub async fn handle_join_message(&mut self, name: &str, addr: SocketAddr) -> io::Result<()> {
        let player = Player {
            address: addr,
            name: name.to_string(),
            is_alive: true,
        };
        self.players.0.push(player);
        let bd_message = Message {
            message_code: JOIN,
            content: json!({
                "name": name,
            }),
        };
        let res_message = Message {
            message_code: RESPONSE,
            content: json!({
                "message": "success",
                "players": self.players.0.iter().map(|player| player.name.clone()).collect::<Vec<String>>(),
            }),
        };
        self.send_message(serde_json::to_value(res_message).unwrap(), addr)
            .await?;

        let bd_message = serde_json::to_value(bd_message).unwrap();
        self.broadcast_except(bd_message, addr).await
    }
    pub async fn handle_messages(&mut self) -> io::Result<()> {
        let mut buf = vec![0; 1024];
        let (len, addr) = self.socket.recv_from(&mut buf).await?;
        let message = raw_message_to_json(&buf[..len])?;
        let message_type = message["message_code"].as_u64().unwrap() as u8;
        match message_type {
            JOIN => {
                self.handle_join_message(message["content"]["name"].as_str().unwrap(), addr)
                    .await?;
            }
            LIVE_LOSS => {
                // let name = message["name"].as_str().unwrap();
                // let damage = message["damage"].as_f64().unwrap() as f32;
                // for player in &mut self.players.0 {
                //     if player.name == name {
                //         player.damage += damage;
                //         if player.damage >= 100.0 {
                //             player.is_alive = false;
                //         }
                //     }
                // }
            }
            TRANSFORM_UPDATE => {
                // let name = message["name"].as_str().unwrap();
                // let player = self.players.get(name).unwrap();
                // let x = message["x"].as_f64().unwrap() as f32;
                // let y = message["y"].as_f64().unwrap() as f32;
                // let z = message["z"].as_f64().unwrap() as f32;
                // player.x = x;
                // player.y = y;
                // player.z = z;
            }
            _ => {}
        }
        Ok(())
    }
    pub async fn run(&mut self) -> io::Result<()> {
        loop {
            self.handle_messages().await?;
        }
    }
}

fn raw_message_to_json(message: &[u8]) -> Result<Value, Error> {
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
