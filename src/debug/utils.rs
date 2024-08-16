use bevy::prelude::*;

use crate::{
    animations::systems::SpawnScenesState,
    player::components::{Player, SceneName},
};

pub fn walk_tree(
    all_entities_with_children: &Query<&Children>,
    names: &Query<&Name>,
    entity: &Entity,
    depth: u32,
) {
    let mut padding = String::from("");
    for _ in 0..depth {
        padding.push_str("-")
    }

    if let Ok(name) = names.get(*entity) {
        println!("{padding}{:#?} ({:?})", name, entity)
    } else {
        println!("{padding}unnamed entity ({:?})", entity)
    }

    if let Ok(children_of_current_entity) = all_entities_with_children.get(*entity) {
        for child_entity in children_of_current_entity {
            walk_tree(all_entities_with_children, names, child_entity, depth + 1)
        }
    }
}

pub fn print_scene_tree(
    scene_query: Query<(Entity, &SceneName), With<SceneName>>,
    all_entities_with_children: Query<&Children>,
    names: Query<&Name>,
    mut next_state: ResMut<NextState<SpawnScenesState>>,
) {
    println!("printing scene tree");
    for (scene_entity, _) in &scene_query {
        walk_tree(&all_entities_with_children, &names, &scene_entity, 0)
    }
    println!("printing scene tree done");
    next_state.set(SpawnScenesState::Done);
}

//print the position of an entity
pub fn print_position(
    entities: Query<Entity, With<Player>>,
    transform_query: Query<&Transform>,
    names: Query<&Name>,
) {
    for entity in entities.iter() {
        if let Ok(transform) = transform_query.get(entity) {
            if let Ok(name) = names.get(entity) {
                println!("{:?} is at {:?}", name, transform.translation)
            } else {
                println!("{:?} is at {:?}", entity, transform.translation)
            }
        } else {
            if let Ok(name) = names.get(entity) {
                println!("{:?} has no position", name)
            } else {
                println!("{:?} has no position", entity)
            }
        }
    }
}
