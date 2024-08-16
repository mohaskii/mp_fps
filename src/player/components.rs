
use bevy::{prelude::*, utils::HashMap};

// Player component
#[derive(Component)]
pub struct Player {
    pub username: String,
    pub health: u32,
}


#[derive(Resource, Debug,Default)]
pub struct SceneEntitiesByName(pub HashMap<String, Entity>);
// impl Default for SceneEntitiesByName {
//     fn default() -> Self {
//         SceneEntitiesByName(HashMap::default())
//     }
    
// }

#[derive(Resource, Debug,Default)]
pub struct Animations{
    pub animations: HashMap<String, AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}


#[derive(Component, Debug)]
pub struct SceneName(pub String);