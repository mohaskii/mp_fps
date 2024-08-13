
use bevy::{prelude::*, utils::HashMap};

// Player component
#[derive(Component)]
pub struct Player {
    pub username: String,
    pub health: u32,
}


#[derive(Resource, Debug)]
pub struct SceneEntitiesByName(pub HashMap<String, Entity>);

#[derive(Resource, Debug)]
pub struct Animations{
    pub animations: HashMap<String, AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}


#[derive(Component, Debug)]
pub struct SceneName(pub String);