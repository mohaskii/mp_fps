use std::collections::HashMap;

use bevy::{
    asset::Handle,
    ecs::{entity::Entity, system::Resource},
    math::Vec3,
    prelude::{AnimationGraph, AnimationNodeIndex},
};
use renet::ClientId;

#[derive(Resource)]
pub struct MyClientId(pub ClientId);

#[derive(Resource)]
pub struct PlayerEntities(pub HashMap<ClientId, Entity>);

#[derive(Resource, Default)]
pub struct ProposedPlayerPosition(pub Vec3);

#[derive(Resource, Default)]
pub struct HasCollision(pub bool);
#[derive(Resource, Default)]
pub struct ProjectileBuffer(pub Vec<mp_fps::ProjectileProperties>);
#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}
#[derive(Resource)]
pub struct Live(pub i32);
#[derive(Resource)]
pub struct ClientEntity(pub HashMap<ClientId, Entity>);
#[derive(Resource, Default)]
pub struct  WaitingEntity (pub Option<Entity>,);
#[derive(Resource, Default)]
pub struct  GameAlreadyStarted(pub bool);