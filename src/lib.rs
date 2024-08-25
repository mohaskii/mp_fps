use std::collections::HashMap;
type Transform = [f32; 3];
type Quaternion = [f32; 4];
use renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerAttributes {
    pub position: Transform,
    pub  rotation: Quaternion,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    PlayerMove(Transform),
    Shoot(ProjectileProperties),    
    PlayerRotation(PlayerRotationValue),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectileProperties {
    pub position: Transform,
    pub velocity: f32,
    pub rotation: [f32; 4],
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerRotationValue {
    pub rotation: [f32; 2]
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    PlayerRotation(HashMap<ClientId, PlayerRotationValue>),
    LobbySync(HashMap<ClientId, PlayerAttributes>),
    Shot(ClientId),
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
}
