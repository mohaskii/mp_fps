use bevy::ecs::event::Event;
use mp_fps::ProjectileProperties;
use renet::ClientId;

#[derive(Event)]
pub struct ShootEvent(pub ClientId, pub ProjectileProperties);
#[derive(Event)]
pub struct PlayerMoveEvent(pub ClientId, pub [f32; 3]);

// #[derive(Event)]
// pub struct PlayerDespawnEvent(pub ClientId);

// #[derive(Event)]
// pub struct PlayerMoveEvent(pub ClientId, pub [f32; 3]);

// #[derive(Event)]
// pub struct LobbySyncEvent(pub std::collections::HashMap<ClientId, PlayerAttributes>);
