use std::collections::HashMap;

use bevy::{ecs::{entity::Entity, system::Resource}, math::Vec3};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

#[derive(Resource)]
pub struct PlayerEntities(pub HashMap<ClientId, Entity>);

#[derive(Resource, Default)]
pub struct ProposedPlayerPosition(pub Vec3);

#[derive(Resource, Default)]
pub struct HasCollision(pub bool);