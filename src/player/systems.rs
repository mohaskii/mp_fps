use std::{collections::HashMap, time::Duration};

use crate::{animations::systems::SpawnScenesState, asset_loader_plugin::MyAssets};

use super::components::*;
use bevy::prelude::*;
// Player system
// pub fn player_system(mut query: Query<&mut Player>) {
//     for mut player in query.iter_mut() {
//         // Update player logic here
//     }
// }

// pub fn spawn_player(
//     mut commands: Commands,
//     ass: Res<AssetServer>,
//     mut graphs: ResMut<Assets<AnimationGraph>>,
// ) {
//     let mut animations = HashMap::new();
//     let mut scene_entities_by_name = HashMap::new();

//     // Build the animation graph
//     let mut graph = AnimationGraph::new();
//     let animations = graph
//         .add_clips(
//             [
//                 GltfAssetLabel::Animation(2).from_asset("player1.glb"),
//                 GltfAssetLabel::Animation(1).from_asset("player1.glb"),
//                 GltfAssetLabel::Animation(0).from_asset("player1.glb"),
//             ]
//             .into_iter()
//             .map(|path| ass.load(path)),
//             1.0,
//             graph.root,
//         )
//         .collect();

//     // Insert a resource with the current scene information
//     let graph = graphs.add(graph);
//     commands.insert_resource(Animations {
//         animations,
//         graph: graph.clone(),
//     });
//     let model = ass.load("player1.glb#Scene0");
//     // Spawn player entity
//     commands.spawn(
//         (Player{
//             username:"my_player".to_string(),
//             health:100
//         },SceneBundle {
//             scene: model,
//             transform: Transform::from_xyz(12.0, 0.0, 14.0),
//             ..default()
//         }),
//     );
// }



pub fn spawn_scenes(
    mut commands: Commands,
    asset_pack: Res<MyAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    mut next_state: ResMut<NextState<SpawnScenesState>>,
) {
    let mut animations: HashMap<String, Handle<AnimationClip>, _> = HashMap::new();
    let mut scene_entities_by_name = HashMap::new();

    // let mut x = 0.0;
    // SPAWN SCENES
    for (name, gltf_handle) in &asset_pack.gltf_files {
        if let Some(gltf) = assets_gltf.get(gltf_handle) {
            println!("SPAWING");
            let mut transform = Transform::from_xyz(0.0, 0.0, 0.0);

            if name == "sword.glb" {
                transform.scale = Vec3::splat(0.1)
            }

            let entity_commands = commands.spawn((
                SceneBundle {
                    scene: gltf.named_scenes["Scene"].clone(),
                    transform,
                    ..Default::default()
                },
                SceneName(name.clone()),
            ));

            let entity = entity_commands.id();
            scene_entities_by_name.insert(name.clone(), entity);

            for named_animation in gltf.named_animations.iter() {
                println!("inserting animation: {}", named_animation.0);
                animations.insert(
                    named_animation.0.to_string(),
                    gltf.named_animations[named_animation.0].clone(),
                );
            }
        }
        // x += 2.0;
    }

    let mut graph = AnimationGraph::new();
    let mut clips = Vec::new();
    for path in asset_pack.gltf_files.keys() {
              clips.push(GltfAssetLabel::Animation(0).from_asset(path)) ;
            }
    let animations = graph
        .add_clips(
            
        
            clips.into_iter()
            .map(|path| ass.load(path)),
            1.0,
            graph.root,
        )
        .collect();

    commands.insert_resource(Animations(animations));
    commands.insert_resource(SceneEntitiesByName(scene_entities_by_name));

    next_state.set(SpawnScenesState::Spawned)
}
// pub fn player_animation(
//     mut commands: Commands,
//     animations: Res<Animations>,
//     mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
// ) {
//     for (entity, mut player) in &mut players {
//         let mut transitions = AnimationTransitions::new();

//         // Make sure to start the animation via the `AnimationTransitions`
//         // component. The `AnimationTransitions` component wants to manage all
//         // the animations and will get confused if the animations are started
//         // directly via the `AnimationPlayer`.
//         transitions
//             .play(&mut player, animations.0, Duration::ZERO)
//             .repeat();

//         commands
//             .entity(entity)
//             .insert(animations.graph.clone())
//             .insert(transitions);
//     }
// }


pub fn run_animations(
    mut animation_player_query: Query<&mut AnimationPlayer>,
    scene_and_animation_player_link_query: Query<
        (&SceneName, &AnimationEntityLink),
        Added<AnimationEntityLink>,
    >,
    animations: Res<Animations>,
    scene_entities_by_name: Res<SceneEntitiesByName>,
) {
    let main_skeleton_scene_entity = scene_entities_by_name
        .0
        .get("main_skeleton.glb")
        .expect("the scene to be registered");

    let (_, animation_player_entity_link) = scene_and_animation_player_link_query
        .get(*main_skeleton_scene_entity)
        .expect("the scene to exist");

    let mut animation_player = animation_player_query
        .get_mut(animation_player_entity_link.0)
        .expect("to have an animation player on the main skeleton");

        let transition = AnimationTransitions::new();
        animation_player.play(animation)
        transition
            .play(
                &mut animation_player,
                animations
                    .0
                    .get("Idle")
                    .expect("to have an animation by this name")
                    .clone_weak(),
                    Duration::ZERO,
            )
            .repeat()
            .set_speed(0.5);    
    animation_player
        .play(
            animations
                .0
                .get("Sword_Slash")
                .expect("to have an animation by this name")
                .clone_weak(),
        )
        .repeat()
        .set_speed(0.5);
}

#[derive(Component, Debug)]
pub struct AnimationEntityLink(pub Entity);

pub fn get_top_parent(
    mut curr_entity: Entity,
    all_entities_with_parents_query: &Query<&Parent>,
) -> Entity {
    //Loop up all the way to the top parent
    loop {
        if let Ok(ref_to_parent) = all_entities_with_parents_query.get(curr_entity) {
            curr_entity = ref_to_parent.get();
        } else {
            break;
        }
    }
    curr_entity
}

pub fn link_animations(
    animation_players_query: Query<Entity, Added<AnimationPlayer>>,
    all_entities_with_parents_query: Query<&Parent>,
    animations_entity_link_query: Query<&AnimationEntityLink>,
    mut commands: Commands,
    mut next_state: ResMut<NextState<SpawnScenesState>>,
) {
    // Get all the Animation players which can be deep and hidden in the heirachy
    for entity_with_animation_player in animation_players_query.iter() {
        let top_entity = get_top_parent(
            entity_with_animation_player,
            &all_entities_with_parents_query,
        );

        // If the top parent has an animation config ref then link the player to the config
        if animations_entity_link_query.get(top_entity).is_ok() {
            warn!("Problem with multiple animation players for the same top parent");
        } else {
            println!(
                "linking entity {:#?} to animation_player entity {:#?}",
                top_entity, entity_with_animation_player
            );
            commands
                .entity(top_entity)
                .insert(AnimationEntityLink(entity_with_animation_player.clone()));
        }
    }

    next_state.set(SpawnScenesState::Done)
}