use bevy::app::{App, Plugin, Startup};

pub mod utils;

pub struct DebugPlugin;

impl Plugin for DebugPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, utils::print_scene_tree);
    }
}