use std::time::Duration;

use crate::{animations::systems::SpawnScenesState, asset_loader_plugin::MyAssets};

use super::components::*;
use bevy::{prelude::*, utils::HashMap};

#[derive(Debug, Component)]
pub struct PlayerModelCamera;

pub fn spawn_scenes(
    mut commands: Commands,
    asset_pack: Res<MyAssets>,
    assets_gltf: Res<Assets<Gltf>>,
    ass: Res<AssetServer>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    mut next_state: ResMut<NextState<SpawnScenesState>>,
) {
    let mut animations: HashMap<String, AnimationNodeIndex> = HashMap::new();
    let mut scene_entities_by_name = HashMap::new();
    let mut clips = Vec::new();
    let mut graph = AnimationGraph::new();

    // println!("111111111\n\n{:?}\n\n11111111", asset_pack.gltf_files);

    // let mut x = 0.0;
    // SPAWN SCENES
    for (name, gltf_handle) in &asset_pack.gltf_files {
        if let Some(gltf) = assets_gltf.get(gltf_handle) {
            println!("********\n\n{:?}\n\n********", gltf.named_scenes);

            println!("SPAWING");
            let transform = Transform::from_xyz(14.5, 0.0, 9.0);
            let mut entity_commands = commands.spawn((
                SceneBundle {
                    scene: gltf
                        .named_scenes
                        .get("Scene")
                        .expect("scene to exist")
                        .clone(),
                    transform,
                    ..Default::default()
                },
                SceneName(name.clone()),
                Player {
                    username: name.clone(),
                    health: 100,
                },
            ));
            entity_commands.with_children(|parent| {
                parent.spawn((
                    PlayerModelCamera,
                    //3d camera bundle
                    Camera3dBundle {
                        transform: Transform {
                            rotation: Quat::from_rotation_y(180.0_f32.to_radians()),
                            ..Transform::from_xyz(0.0, 0.8, 0.3)
                        },
                        projection: Projection::Perspective(PerspectiveProjection {
                            fov: 90.0_f32.to_radians(),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                    // GlobalTransform::IDENTITY, // Add this line to set the initial global transform
                ));
            });
            let entity = entity_commands.id();
            scene_entities_by_name.insert(name.clone(), entity);
            // }
            let mut sorted_animation = gltf.named_animations.iter().collect::<Vec<_>>();
            sorted_animation.sort_by(|a, b| a.0.cmp(&b.0));
            for (i, (a_name, _)) in sorted_animation.iter().enumerate() {
                let animation_index = i + 1;
                let animation_name = a_name.to_string();
                animations.insert(animation_name, AnimationNodeIndex::new(animation_index));

                clips.push(GltfAssetLabel::Animation(i).from_asset(name.clone()));
            }
            clips
                .clone()
                .into_iter()
                .map(|path| ass.load(path))
                .for_each(|clip| {
                    graph.add_clip(clip, 1.0, graph.root);
                });
            let graph = graphs.add(graph.clone());

            commands.insert_resource(Animations {
                animations: animations.clone(),
                graph: graph,
            });
        }
    }

    println!("********\n\n{:?}\n\n********", scene_entities_by_name);

    commands.insert_resource(SceneEntitiesByName(scene_entities_by_name));

    next_state.set(SpawnScenesState::Spawned)
}

pub fn run_animations(
    animations: Res<Animations>,
    scene_entities_by_name: Res<SceneEntitiesByName>,
    mut players: Query<(Entity, &mut AnimationPlayer)>,
    mut commands: Commands,
) {
    println!(
        "####\n\n{:?}\n\t\t-------\n{:?}\n\n####",
        scene_entities_by_name, animations
    );
    let main_skeleton_scene_entity = scene_entities_by_name
        .0
        .get("player1.glb")
        .expect("the scene to be registered");

    println!("entity ####\n\n{:?}\n\n####", main_skeleton_scene_entity);
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions.play(
            &mut player,
            animations
                .animations
                .get("Armature.001|mixamo.com|Layer0.003")
                .expect("to have an animation by this name")
                .clone(),
            Duration::ZERO,
        ).repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
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
