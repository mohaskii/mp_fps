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
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProjectileProperties {
    pub position: Transform,
    pub rotation: Quaternion,
    pub direction: Transform,
    
}


#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbySync(HashMap<ClientId, PlayerAttributes>),
    Shoot(ProjectileProperties),
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
}
