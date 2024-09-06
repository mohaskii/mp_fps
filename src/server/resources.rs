use std::collections::HashMap;

use bevy::ecs::system::Resource;
use mp_fps::PlayerAttributes;
use renet::ClientId;

#[derive(Resource, Clone)]
pub struct PlayerLobby(pub HashMap<ClientId, PlayerAttributes>);
#[derive(Resource, Default)]
pub  struct ProjectileBuffer(pub Vec<mp_fps::ProjectileProperties>);

#[derive(Resource)]
pub struct  PlayerInTheGame(pub u32);

