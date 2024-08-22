use crate::map_plugin::*;
use bevy::color::palettes::css::{RED, WHITE};
use bevy::color::palettes::tailwind::{self, BLUE_500};
use bevy::input::mouse::MouseMotion;
use bevy::pbr::NotShadowCaster;
use bevy::render::view::RenderLayers;
use bevy::window::{CursorGrabMode, PrimaryWindow};
use bevy::{
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        event::{EventReader, EventWriter},
        system::{Commands, Query, Res, ResMut},
    },
    input::keyboard::{KeyCode, KeyboardInput},
    log::info,
    math::{
        primitives::{Cuboid, Plane3d},
        Vec3,
    },
    pbr::{MaterialMeshBundle, StandardMaterial},
    prelude::*,
    render::mesh::Mesh,
    transform::components::Transform,
};
use mp_fps::PlayerAttributes;
use renet::{DefaultChannel, RenetClient};

use crate::{
    components::{MyPlayer, PlayerEntity, WorldModelCamera},
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    resources::{HasCollision, ProposedPlayerPosition},
    MyClientId,
};

const VIEW_MODEL_RENDER_LAYER: usize = 1;
const DEFAULT_RENDER_LAYER: usize = 0;

pub fn send_message_system(mut client: ResMut<RenetClient>, query: Query<(&MyPlayer, &Transform)>) {
    let (_, transform) = query.single();
    let player_sync = PlayerAttributes {
        position: transform.translation.into(),
    };
    let message = bincode::serialize(&player_sync).unwrap();
    client.send_message(DefaultChannel::Unreliable, message);
}

pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
) {
    while let Some(message) = client.receive_message(DefaultChannel::ReliableOrdered) {
        let server_message = bincode::deserialize(&message).unwrap();

        match server_message {
            mp_fps::ServerMessage::PlayerJoin(client_id) => {
                info!("Client connected: {}", client_id);
                spawn_events.send(PlayerSpawnEvent(client_id));
            }
            mp_fps::ServerMessage::PlayerLeave(client_id) => {
                info!("Client disconnected: {}", client_id);
                despawn_events.send(PlayerDespawnEvent(client_id));
            }
            _ => {
                info!("Unhandled message: {:?}", server_message);
            }
        }
    }

    while let Some(message) = client.receive_message(DefaultChannel::Unreliable) {
        let message = bincode::deserialize(&message).unwrap();

        match message {
            mp_fps::ServerMessage::LobbySync(map) => {
                lobby_sync_events.send(LobbySyncEvent(map));
            }
            _ => {
                info!("Unhandled message: {:?}", message);
            }
        }
    }
}

pub fn update_player_movement_system(
    mut keyboard_events: EventReader<KeyboardInput>,
    mut query: Query<(&mut Transform, &MyPlayer)>,
) {
    let (mut transform, _) = query.single_mut();

    for event in keyboard_events.read() {
        let mut delta_position = Vec3::new(0.0, 0.0, 0.0);

        match event.key_code {
            KeyCode::KeyW => delta_position.z += 0.1,
            KeyCode::KeyS => delta_position.z -= 0.1,
            KeyCode::KeyA => delta_position.x -= 0.1,
            KeyCode::KeyD => delta_position.x += 0.1,
            _ => {}
        }

        let new_position = transform.translation + delta_position;
        transform.translation = new_position;
    }
}

pub fn apply_movement(
    mut query: Query<&mut Transform, With<MyPlayer>>,
    proposed_position: Res<ProposedPlayerPosition>,
    has_collision: Res<HasCollision>,
) {
    if let Ok(mut player_transform) = query.get_single_mut() {
        if has_collision.0 {
            // Calculer la direction de la collision
            let push_direction = (player_transform.translation - proposed_position.0).normalize();

            // Définir une distance de repoussement
            let push_distance = 0.001; // Ajustez cette valeur selon le besoin

            // Repousser le joueur légèrement pour sortir de la collision
            player_transform.translation += push_direction * push_distance;
        } else {
            // Appliquer le mouvement proposé s'il n'y a pas de collision
            player_transform.translation = proposed_position.0;
        }
        println!("player position: {:?}", player_transform.translation);
    }
}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: WHITE.into(),
                intensity:100_000.0,
                shadows_enabled: true,
                radius: 100.0,
                ..default()
            },
            transform: Transform::from_xyz(18.5, 2.5, 15.5),
            ..default()
        },
        
        // The light source illuminates both the world model and the view model.
        RenderLayers::from_layers(&[DEFAULT_RENDER_LAYER, VIEW_MODEL_RENDER_LAYER]),
    ));
}

pub fn spawn_text(mut commands: Commands) {
    commands
        .spawn(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            ..default()
        })
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                concat!(
                    "Move the camera with your mouse.\n",
                    "Press arrow up to decrease the FOV of the world model.\n",
                    "Press arrow down to increase the FOV of the world model."
                ),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        });
}

pub fn move_player(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<MyPlayer>>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);

        println!("{}", transform.forward().y + pitch);

        // clamp the pitch between -0.9 & 0.9 of the y axis
        if transform.forward().y + pitch <= 0.9 && transform.forward().y + pitch >= -0.9 {
            transform.rotate_local_x(pitch);
        }
    }
}

pub fn player_position_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player_query: Query<&Transform, With<MyPlayer>>,
    mut proposed_position: ResMut<ProposedPlayerPosition>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let speed = 3.0;
        let delta = time.delta_seconds();

        let mut movement = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::KeyW) {
            movement += player_transform.forward().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyS) {
            movement += player_transform.back().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyA) {
            movement += player_transform.left().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::KeyD) {
            movement += player_transform.right().as_vec3();
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize();
        }

        proposed_position.0 = player_transform.translation + movement * speed * delta;
        proposed_position.0.y = 1.0;
    }
}

fn check_collision(
    player_position: Vec3,
    player_size: Vec3,
    wall_position: Vec3,
    wall_size: Vec3,
) -> bool {
    let collision_factor = 0.6; // Réduction de 20% de la distance de collision
    let min_distance = (player_size + wall_size) * 0.5 * collision_factor;
    let actual_distance = player_position - wall_position;
    actual_distance.abs().cmple(min_distance).all()
}

pub fn check_collision_system(
    mut query: ParamSet<(
        Query<&Collider, With<MyPlayer>>,
        Query<(&Transform, &Collider), With<Wall>>,
    )>,
    mut has_collision: ResMut<HasCollision>,
    proposed_position: Res<ProposedPlayerPosition>,
) {
    let plyer_collider = query.p0().single().clone();
    let wall_query = query.p1();
    for (wall_transform, wall_collider) in wall_query.iter() {
        if check_collision(
            proposed_position.0,
            plyer_collider.size,
            wall_transform.translation,
            wall_collider.size,
        ) {
            has_collision.0 = true;
            return;
        }
    }
    has_collision.0 = false;
}

pub fn setup_system(
    mut commands: Commands,
    ass: Res<AssetServer>,
) {
    commands
        .spawn((
            MyPlayer,
            Collider {
                size: Vec3::new(1.0, 1.0, 1.0),
            }, // Ajoutez le collider ici
            SpatialBundle {
                transform: Transform::from_xyz((18. / 2.) + 1.5, 2.0, (14.0 / 2.) + 1.5),
                ..default()
            },
        ))
        .with_children(|parent| {
            let riffle = ass.load("m4_carbine_rifle.glb#Scene0");
            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 60.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    ..default()
                },
            ));
            // Spawn view model camera.
            parent.spawn((
                // Camera3dBundle {
                //     camera: Camera {
                //         // Bump the order to render on top of the world model.
                //         order: 1,
                //         ..default()
                //     },
                //     projection: PerspectiveProjection {
                //         fov: 70.0_f32.to_radians(),
                //         ..default()
                //     }
                //     .into(),
                //     ..default()
                // },
                // Only render objects belonging to the view model.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));

            // Spawn the player's right arm.
            parent.spawn((
                SceneBundle {
                    scene: riffle,
                    // material: arm_material,
                    transform: Transform {
                        scale: Vec3 {
                            x: 0.15,
                            y: 0.15,
                            z: 0.15,
                        },
                        ..Transform::from_xyz(0.1, -0.1, -0.25)
                    },
                    ..default()
                },
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
        });
}

pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
) {
    for event in spawn_events.read() {
        info!("Handling player spawn event: {:?}", event.0);
        let client_id = event.0;

        commands.spawn((
            MaterialMeshBundle {
                material: materials.add(StandardMaterial {
                    base_color: Color::rgb(1.0, 0.0, 0.0),
                    ..default()
                }),
                mesh: meshes.add(Cuboid::default()),
                ..default()
            },
            PlayerEntity(client_id),
        ));
    }
}

pub fn handle_lobby_sync_event_system(
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut sync_events: EventReader<LobbySyncEvent>,
    mut query: Query<(&PlayerEntity, &mut Transform)>,
    my_clinet_id: Res<MyClientId>,
) {
    let event_option = sync_events.read().last();
    if event_option.is_none() {
        return;
    }
    let event = event_option.unwrap();

    for (client_id, player_sync) in event.0.iter() {
        if *client_id == my_clinet_id.0 {
            continue;
        }

        let mut found = false;
        for (player_entity, mut transform) in query.iter_mut() {
            if *client_id == player_entity.0 {
                let new_position = player_sync.position;
                transform.translation = new_position.into();
                found = true;
            }
        }

        if !found {
            info!("Spawning player {}: {:?}", client_id, player_sync.position);
            spawn_events.send(PlayerSpawnEvent(*client_id));
        }
    }
}

pub fn spawn_crosshair(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let crosshair: Handle<Image> = asset_server.load("crosshair.png");
    // commands.spawn(UiCameraBundle::default());

    // Crée un élément simple (un carré blanc) pour représenter le viseur
    let kiki = ImageBundle {
        style: Style {
            align_self: AlignSelf::Center,
            position_type: PositionType::Absolute,
            margin: UiRect::all(Val::Auto),
            width: Val::Px(100.0), // Taille du viseur

            height: Val::Px(100.0),
            // Taille du viseur
            ..Default::default()
        },
        // image_size:UiImageSize::new(Val::Px(50.0), Val::Px(50.0)),
        image: UiImage::new(crosshair),

        // Couleur du viseur
        ..Default::default()
    };
    // kiki.lao;
    commands.spawn(kiki);
}

pub fn cursor_grab(mut q_windows: Query<&mut Window, With<PrimaryWindow>>) {
    let mut primary_window = q_windows.single_mut();

    // if you want to use the cursor, but not let it leave the window,
    // use `Confined` mode:
    primary_window.cursor.grab_mode = CursorGrabMode::Confined;

    // for a game that doesn't use the cursor (like a shooter):
    // use `Locked` mode to keep the cursor in one place
    primary_window.cursor.grab_mode = CursorGrabMode::Locked;

    // also hide the cursor
    primary_window.cursor.visible = false;
}
