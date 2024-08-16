use bevy::prelude::*;
#[derive(Debug, Component, Default)]
pub struct Player {
    pub speed: f32,
    pub health: f32,
    pub max_health: f32,
    pub damage: f32,
    pub is_alive: bool,
    pub transform: Transform,
}