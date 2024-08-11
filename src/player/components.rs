use bevy::prelude::*;

// Player component
#[derive(Component)]
pub struct Player {
    pub username: String,
    pub health: u32,
    pub position: Vec3,
}


#[derive(Resource)]
pub struct Animations {
    pub animations: Vec<AnimationNodeIndex>,
    pub graph: Handle<AnimationGraph>,
}
