use std::collections::HashMap;

use bevy::ecs::system::Resource;
use mp_fps::PlayerAttributes;
use renet::ClientId;

#[derive(Resource, Clone)]
pub struct PlayerLobby(pub HashMap<ClientId, PlayerAttributes>);

