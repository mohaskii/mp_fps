use bevy::prelude::*;

use crate::{despawn_menu, spawn_world_model, states::GameState};
pub struct MapPlugin;

#[derive(Component)]
pub struct Wall;

#[derive(Component, Clone)]
pub struct Collider {
    pub size: Vec3,
}
impl Plugin for MapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            OnEnter(GameState::Playing),
            (spawn_world_model, despawn_menu),
        );
    }
}
