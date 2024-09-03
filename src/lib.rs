use std::collections::HashMap;
type Transform = [f32; 3];
type Quaternion = [f32; 4];
use bevy::prelude::Resource;
use renet::ClientId;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, Clone, Default,Resource)]
pub struct PlayerAttributes {
    pub position: Transform,
    pub rotation: Quaternion,
    pub pitch: f32,
}

#[derive(Serialize, Deserialize, Debug)]
pub enum ClientMessage {
    PlayerMove(Transform),
    Shoot(ProjectileProperties),
    PlayerAttributes(PlayerAttributes),
    PlayerDie
}
#[derive(Serialize, Deserialize, Debug, Clone, Resource)]
pub struct ProjectileProperties {
    pub position: Transform,
    pub rotation: Quaternion,
    pub direction: Transform,
}
#[derive(Resource, Default)]
pub struct P(pub f32);

#[derive(Serialize, Deserialize, Debug)]
pub enum ServerMessage {
    LobbySync(HashMap<ClientId, PlayerAttributes>),
    Shoot(ClientId, ProjectileProperties),
    PlayerJoin(ClientId),
    PlayerLeave(ClientId),
}
