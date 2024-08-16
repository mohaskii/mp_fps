
use std::collections::HashMap;

use bevy::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub enum SpawnScenesState {
    #[default]
    Spawning,
    Spawned,
    Done,
}


// #[derive(Resource, Debug)]
// pub struct SceneEntitiesByName(pub HashMap<String, Entity>);

// #[derive(Resource, Debug)]
// pub struct Animations(pub HashMap<String, Handle<AnimationClip>>);