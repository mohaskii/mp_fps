use std::time::Duration;

use super::components::*;
use bevy::prelude::*;
// Player system
pub fn player_system(mut query: Query<&mut Player>) {
    for mut player in query.iter_mut() {
        // Update player logic here
    }
}

pub fn spawn_player(mut commands: Commands, ass: Res<AssetServer>,    mut graphs: ResMut<Assets<AnimationGraph>>,) {
// Build the animation graph
let mut graph = AnimationGraph::new();
let animations = graph
    .add_clips(
        [
            // GltfAssetLabel::Animation(2).from_asset("player1.glb"),
            // GltfAssetLabel::Animation(1).from_asset("player1.glb"),
            GltfAssetLabel::Animation(0).from_asset("player1.glb"),
        ]
        .into_iter()
        .map(|path| ass.load(path)),
        1.0,
        graph.root,
    )
    .collect();

// Insert a resource with the current scene information
let graph = graphs.add(graph);
commands.insert_resource(Animations {
    animations,
    graph: graph.clone(),
});
let model = ass.load("player1.glb#Scene0");
    // Spawn player entity
    commands.spawn(
        (SceneBundle {
            scene: model,
            transform: Transform::from_xyz(12.0, 0.0, 14.0),
            ..default()
        }),
    );
}

pub fn player_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        // Make sure to start the animation via the `AnimationTransitions`
        // component. The `AnimationTransitions` component wants to manage all
        // the animations and will get confused if the animations are started
        // directly via the `AnimationPlayer`.
        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}
