use std::collections::HashMap;
type Transform = [f32; 3];
type Quaternion = [f32; 4];
use bevy::prelude::Resource;
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
    PlayerAttributes(PlayerAttributes),
}
#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct ProjectileProperties {
    pub position: Transform,
    pub rotation: Quaternion,
    pub direction: Transform,
    
}


#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {

    LobbySync(HashMap<ClientId, PlayerAttributes>),
    Shoot(ClientId, ProjectileProperties),
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
}
