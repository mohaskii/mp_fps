use bevy::prelude::*;

use crate::Player;
#[derive(Component)]
pub struct Projectile {
    direction: Vec3,
    speed: f32,
}

fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
) {
    let player_transform = player_query.get_single();
    match player_transform {
        Ok(_) => {}
        Err(e) => {
            println!("error:{:?}", e);
            return;
        }
    }
    let player_transform = player_transform.unwrap();
    let projectile_speed = 20.0;

    // La direction du projectile sera la direction du bras du joueur
    let direction = player_transform.forward().as_vec3();

    // Créer un projectile avec une forme de cylindre pour représenter le laser
    let projectile = meshes.add(Cylinder::new(0.05, 2.0));
    // let projectile_material = materials.add(Color::srgb(1.0, 0.0, 0.0)); // Rouge pour l'effet laser
    let projectile_material = materials.add(StandardMaterial {
            base_color: Color::srgb(1.0, 0.0, 0.0), // Rouge
            emissive: LinearRgba::rgb(1.0, 0.0, 0.0),
            diffuse_transmission: 1.0,
            // reflectance: .0,
            // Rouge émissif
            ..Default::default()
        });
    commands.spawn((
        Projectile {
            direction,
            speed: projectile_speed,
        },
        MaterialMeshBundle {
            mesh: projectile,
            material: projectile_material,
            transform: Transform {
                translation: player_transform.translation + direction * 0.5,
                rotation: player_transform.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
}

pub fn shoot_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<Player>>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Tirer un projectile
        println!("Pew!");
        spawn_projectile(commands, meshes, materials, player_query);
    }
}
pub fn projectile_movement_system(
    mut commands: Commands,
    time: Res<Time>,
    mut projectile_query: Query<(Entity, &mut Transform, &Projectile)>,
) {
    for (entity, mut transform, projectile) in projectile_query.iter_mut() {
        // Déplacer le projectile
        transform.translation += projectile.direction * projectile.speed * time.delta_seconds();

        // Optionnel : Détruire le projectile après un certain temps ou s'il dépasse une certaine distance
        if transform.translation.length() > 100.0 {
            commands.entity(entity).despawn();
        }
    }
}
