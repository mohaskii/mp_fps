use bevy::prelude::*;
use bevy::{
    input::mouse::MouseMotion,
    render::view::RenderLayers,
};

use super::components::*;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;

pub fn spawn_free_cam(mut commands: Commands) {
    commands
        .spawn((SpatialBundle {
            transform: Transform::from_xyz(0.0, 10.0, 0.0),
            ..default()
        },))
        .with_children(|parent| {
            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));
            // Spawn view model camera.
            parent.spawn((
                Camera3dBundle {
                    camera: Camera {
                        // Bump the order to render on top of the world model.
                        order: 1,
                        ..default()
                    },
                    projection: PerspectiveProjection {
                        fov: 70.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));
        });
}

pub fn move_camera(
    mut mouse_motion: EventReader<MouseMotion>,
    mut cam: Query<&mut Transform, With<WorldModelCamera>>,
) {
    let mut transform = cam.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
    }
}

pub fn free_cam_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    mut cam_query: Query<&mut Transform, With<WorldModelCamera>>,
) {
    if let Ok(mut transform) = cam_query.get_single_mut() {
        // Définir la vitesse de déplacement
        let speed = 10.0;
        let delta = time.delta_seconds();

        // Calculer le vecteur de déplacement
        let mut movement = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            movement += transform.forward().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            movement += transform.back().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement += transform.left().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            movement += transform.right().as_vec3();
        }

        // Normaliser le vecteur de mouvement pour éviter une vitesse plus rapide en diagonale
        if movement != Vec3::ZERO {
            movement = movement.normalize();
        }

        // Appliquer le mouvement en tenant compte de la vitesse et du temps delta
        transform.translation += movement * speed * delta;

        // Optionnel : Limiter le mouvement vertical
        // transform.translation.y = 1.0; // Maintient une hauteur constante
        // println!("\n\n{:?}\n\n", transform.translation)
        // Afficher la nouvelle position (pour le débogage)
    }
}
