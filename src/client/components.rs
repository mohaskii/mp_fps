use bevy::ecs::component::Component;
use renet::ClientId;

#[derive(Component)]
pub struct PlayerEntity(pub ClientId);

#[derive(Component)]
pub struct MyPlayer;

#[derive(Debug, Component)]
pub struct WorldModelCamera;

#[derive(Component, Default)]
pub struct HasCollision(pub bool);