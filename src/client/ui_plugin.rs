use bevy::prelude::*;

use crate::{button_system, spawn_menu, states::GameState};
pub struct UiPlugin;
impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .add_systems(Startup, spawn_menu)
            .add_systems(Update, button_system);
    }
}