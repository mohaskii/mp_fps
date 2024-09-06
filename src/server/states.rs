use bevy::prelude::*;
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum ServerState {
    #[default]

    // Waiting,    
    Playing,
    EndOfTheGame,
}