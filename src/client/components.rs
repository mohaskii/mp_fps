use bevy::ecs::component::Component;
use bevy::prelude::*;
use renet::ClientId;

#[derive(Component)]
pub struct PlayerEntity(pub ClientId);

#[derive(Component)]
pub struct MyPlayer;

#[derive(Debug, Component)]
pub struct WorldModelCamera;

#[derive(Component, Default)]
pub struct HasCollision(pub bool);
#[derive(Component)]
pub struct MenuElement;
#[derive(Component)]
pub struct PlayButton;
#[derive(Component)]
pub struct MiniMapPlayer;
#[derive(Component, Clone)]
pub struct Projectile {
    pub direction: Vec3,
    pub speed: f32,
}
#[derive(Component)]
pub struct  PlayerBody ;
#[derive(Component)]
pub struct WaitingText ;