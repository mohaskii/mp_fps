use crate::{
    components::{MenuElement, MiniMapPlayer, PlayButton},
    map_plugin::*,
    states::GameState,
};
use bevy::color::palettes::tailwind;
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
    sprite::MaterialMesh2dBundle,
    // render::{mesh::Mesh},
    transform::components::Transform,
};
use mp_fps::{ClientMessage, PlayerAttributes, PlayerRotationValue};
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
    let mut r = [0.0; 4];
    transform.rotation.write_to_slice(&mut r);
    let player_sync = PlayerAttributes {
        position: transform.translation.into(),
        rotation: r,
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
    mut minimap_player_query: Query<&mut Transform, (With<MiniMapPlayer>, Without<MyPlayer>)>,
    proposed_position: Res<ProposedPlayerPosition>,
    has_collision: Res<HasCollision>,
) {
    let mut minimap_player_transform = minimap_player_query.get_single_mut().unwrap();
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
            minimap_player_transform.translation = Vec3::new(
                (player_transform.translation.x.floor() - (23. / 2.) + 0.5) * 10.,
                (player_transform.translation.z.floor() - (14. / 2.) + 0.5) * 10.,
                0.,
            );
        }
    }
}

pub fn spawn_lights(mut commands: Commands) {
    commands.spawn((
        PointLightBundle {
            point_light: PointLight {
                color: Color::from(tailwind::ROSE_300),
                shadows_enabled: true,
                ..default()
            },
            transform: Transform::from_xyz(-2.0, 4.0, -0.75),
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
    mut client: ResMut<RenetClient>,
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<MyPlayer>>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
        

        let message = bincode::serialize(&ClientMessage::PlayerRotation(PlayerRotationValue {
            rotation: [pitch, yaw],
        }))
        .unwrap();
        client.send_message(DefaultChannel::ReliableOrdered, message);
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
    let collision_factor = 0.7; // Réduction de 20% de la distance de collision
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    commands
        .spawn((
            MyPlayer,
            Collider {
                size: Vec3::new(1.0, 1.0, 1.0),
            }, // Ajoutez le collider ici
            SpatialBundle {
                transform: Transform::from_xyz((23. / 2.) + 1.5, 2.0, (14.0 / 2.) + 1.5),
                ..default()
            },
        ))
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
                // RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            ));

            // Spawn the player's right arm.
            parent.spawn((
                MaterialMeshBundle {
                    mesh: arm,
                    material: arm_material,
                    transform: Transform::from_xyz(0.2, -0.1, -0.25),
                    ..default()
                },
                // Ensure the arm is only rendered by the view model camera.
                // RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
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
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));
    for event in spawn_events.read() {
        info!("Handling player spawn event: {:?}", event.0);
        let client_id = event.0;

        commands
            .spawn((
                // Ajoutez le collider ici
                SpatialBundle {
                    transform: Transform::from_xyz((23. / 2.) + 1.5, 2.0, (14.0 / 2.) + 1.5),
                    ..default()
                },
                PlayerEntity(client_id),
            ))
            .with_children(|parent| {
                // Spawn view model camera.

                // Spawn the player's right arm.
                parent.spawn((
                    MaterialMeshBundle {
                        mesh: arm.clone(),
                        material: arm_material.clone(),
                        transform: Transform::from_xyz(0.2, -0.1, -0.25),
                        ..default()
                    },
                    // Ensure the arm is only rendered by the view model camera.
                    // RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                    // The arm is free-floating, so shadows would look weird.
                    NotShadowCaster,
                ));
            });
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
                let new_rotation = player_sync.rotation;
                transform.translation = new_position.into();
                transform.rotation = Quat::from_array(new_rotation);
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
pub fn despawn_menu(mut commands: Commands, menu_elements: Query<Entity, With<MenuElement>>) {
    for entity in menu_elements.iter() {
        commands.entity(entity).despawn_recursive();
    }
}
pub fn button_system(
    mut next_state: ResMut<NextState<GameState>>,
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor),
        (Changed<Interaction>, With<PlayButton>),
    >,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.35, 0.75, 0.35).into();
                next_state.set(GameState::Playing);
            }
            Interaction::Hovered => {
                *color = Color::srgb(0.25, 0.25, 0.25).into();
            }
            Interaction::None => {
                *color = Color::srgb(0.15, 0.15, 0.15).into();
            }
        }
    }
}
pub fn spawn_menu(mut commands: Commands, asset_server: Res<AssetServer>) {
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 3,
            ..default()
        },
        ..default()
    });
    let background_image: Handle<Image> = asset_server.load("game_image.png");
    commands
        .spawn((
            ImageBundle {
                style: Style {
                    width: Val::Percent(100.0),
                    height: Val::Percent(100.0),
                    justify_content: JustifyContent::Center,
                    align_items: AlignItems::Center,
                    ..default()
                },
                image: UiImage::new(background_image),
                ..default()
            },
            MenuElement,
        ))
        .with_children(|parent| {
            parent
                .spawn((
                    ButtonBundle {
                        style: Style {
                            width: Val::Px(150.0),
                            height: Val::Px(65.0),
                            justify_content: JustifyContent::Center,
                            align_items: AlignItems::Center,
                            ..default()
                        },
                        background_color: Color::srgb(0.15, 0.15, 0.15).into(),
                        ..default()
                    },
                    PlayButton,
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        "Play",
                        TextStyle {
                            font_size: 40.0,
                            color: Color::srgb(0.9, 0.9, 0.9),
                            ..default()
                        },
                    ));
                });
        });
}
pub fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut mini_map_materials: ResMut<Assets<ColorMaterial>>,
) {
    // Définir la carte du labyrinthe
    commands.spawn(Camera2dBundle {
        camera: Camera {
            order: 2,
            ..default()
        },
        ..default()
    });
    let maze = vec![
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ],
        vec![
            1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1,
        ],
        vec![
            1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1,
        ],
        vec![
            1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
        ],
    ];

    // Calculer la taille du sol en fonction de la taille de la matrice du labyrinthe
    let maze_width = maze[0].len() as f32;
    let maze_height = maze.len() as f32;

    // Créer le sol
    let floor = meshes.add(Plane3d::new(
        Vec3::Y,
        Vec2::new(maze_width / 2., maze_height / 2.),
    ));
    let floor_material = materials.add(Color::WHITE);

    commands.spawn(PbrBundle {
        mesh: floor,
        material: floor_material,
        transform: Transform::from_xyz(maze_width, 0.0, maze_height),
        ..default()
    });
    let cube_size = 1.0;
    let cube_half_size = cube_size / 2.0;

    // Créer les cubes pour le labyrinthe
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::BLACK);
    let mini_map_cube_material = mini_map_materials.add(Color::srgb(0.02, 0.4, 0.4));

    // Créer l'entité parent pour le mini-map
    let mini_map_parent = commands.spawn(SpatialBundle::default()).id();

    let p = commands
        .spawn((
            MaterialMesh2dBundle {
                mesh: meshes.add(Rectangle::default()).into(),
                transform: Transform::from_xyz(
                    (1. + cube_half_size) * 10.,
                    (1. + cube_half_size) * 10.,
                    0.0,
                )
                .with_scale(Vec3::splat(10.)),
                material: mini_map_materials.add(Color::WHITE),
                ..default()
            },
            MiniMapPlayer,
        ))
        .id();
    commands.entity(mini_map_parent).push_children(&[p]);

    for (z, row) in maze.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                commands.spawn((
                    PbrBundle {
                        mesh: cube.clone(),
                        material: cube_material.clone(),
                        transform: Transform::from_xyz(
                            (x as f32 + cube_half_size) + maze_width / 2.,
                            0.5,
                            (z as f32 + cube_half_size) + maze_height / 2.,
                        ),
                        ..default()
                    },
                    Wall,
                    Collider {
                        size: Vec3::new(1.0, 1.0, 1.0),
                    },
                ));

                let mini_map_cube = commands
                    .spawn((MaterialMesh2dBundle {
                        mesh: meshes.add(Rectangle::default()).into(),
                        transform: Transform::from_xyz(
                            (x as f32 + cube_half_size) * 10.0,
                            (z as f32 + cube_half_size) * 10.0,
                            0.0,
                        )
                        .with_scale(Vec3::splat(10.0)),
                        material: mini_map_cube_material.clone(),
                        ..default()
                    },))
                    .id();

                // Ajouter le carré à l'entité parent du mini-map
                commands
                    .entity(mini_map_parent)
                    .push_children(&[mini_map_cube]);
            }
        }
    }

    // Positionner le mini-map dans un coin de l'écran
    commands.entity(mini_map_parent).insert(Transform {
        translation: Vec3::new(-700.0, 200.0, 0.0), // Ajustez cette valeur pour positionner dans le coin souhaité
        ..default()
    });
}
