use std::{f32::consts::PI, time::Duration};

use crate::{
    components::{MenuElement, MiniMapPlayer, PlayButton, PlayerBody, Projectile, WaitingText},
    events::{PlayerMoveEvent, ShootEvent},
    map_plugin::*,
    states::GameState,
    Animations, ClientEntity, GameAlreadyStarted, Live, WaitingEntity,
};

pub const VIEW_MODEL_RENDER_LAYER: usize = 1;
const DEFAULT_RENDER_LAYER: usize = 0;
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
use mp_fps::{ClientMessage, PlayerAttributes, ProjectileProperties, P};
use renet::{DefaultChannel, RenetClient};

use crate::{
    components::{MyPlayer, PlayerEntity, WorldModelCamera},
    events::{LobbySyncEvent, PlayerDespawnEvent, PlayerSpawnEvent},
    resources::{HasCollision, ProposedPlayerPosition},
    MyClientId,
};

pub fn send_message_system(
    mut client: ResMut<RenetClient>,
    query: Query<(&MyPlayer, &Transform)>,
    // player_attributes: Res<PlayerAttributes>,
) {
    // let _ = player_attributes;
    let (_, transform) = query.single();
    let mut r = [0.0; 4];
    transform.rotation.write_to_slice(&mut r);
    let player_sync = PlayerAttributes {
        position: transform.translation.into(),
        rotation: r,
        pitch: 0.,
        // pitch: p.0,
    };
    let message =
        bincode::serialize(&ClientMessage::PlayerAttributes(player_sync.clone())).unwrap();
    client.send_message(DefaultChannel::Unreliable, message);
}

pub fn receive_message_system(
    mut client: ResMut<RenetClient>,
    mut spawn_events: EventWriter<PlayerSpawnEvent>,
    mut despawn_events: EventWriter<PlayerDespawnEvent>,
    mut lobby_sync_events: EventWriter<LobbySyncEvent>,
    mut shoot_events: EventWriter<ShootEvent>,
    map_client_entity: Res<ClientEntity>,
    mut next_state: ResMut<NextState<GameState>>,
    mut commands: Commands,
    mut game_already_started: ResMut<GameAlreadyStarted>,
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
            mp_fps::ServerMessage::Shoot(client_id, projectile_projectile) => {
                shoot_events.send(ShootEvent(client_id, projectile_projectile));
            }
            mp_fps::ServerMessage::DaNiggaDie(client_id) => {
                info!("Player {} died!", client_id);
                let player_entity = map_client_entity.0.get(&client_id).unwrap();
                commands.entity(player_entity.clone()).despawn();
            }
            mp_fps::ServerMessage::YouWon => {
                info!("You won!");
                next_state.set(GameState::IWon);
            }
            mp_fps::ServerMessage::GameStarted => {
                info!("Game started!");
                if game_already_started.0 {
                    return; // Don't send message again if game has already started
                }

                game_already_started.0 = true;
                next_state.set(GameState::GameStarted)
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
    mut player_move_event: EventWriter<PlayerMoveEvent>,
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
            player_move_event.send(PlayerMoveEvent(player_transform.translation.into()));
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
    ));
}

pub fn spawn_text(mut commands: Commands, mut waiting_entity: ResMut<WaitingEntity>) {
    let We = commands
        .spawn((
            WaitingText,
            NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,
                    bottom: Val::Px(12.0),
                    left: Val::Px(12.0),
                    ..default()
                },
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section(
                concat!("Waiting for player.\n",),
                TextStyle {
                    font_size: 25.0,
                    ..default()
                },
            ));
        })
        .id();
    println!("ldsdkskdks");
    waiting_entity.0 = Some(We);
    println!("this is the entity : {}", We);
    // commands.entity(We).despawn()
}

pub fn move_player(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, (With<MyPlayer>, Without<WorldModelCamera>)>,
    // mut player_attributes: ResMut<PlayerAttributes>,
    // mut camera_transform: Query<&mut Transform, (Without<MyPlayer>, With<WorldModelCamera>)>
) {
    let mut transform = player.single_mut();
    // let mut camera_transform = camera_transform.single_mut();
    for motion in mouse_motion.read() {
        // let mut transform_to_send = Transform::from_translation(transform.translation);
        // let mut r = [0.0; 4];
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // transform_to_send.rotate_y(yaw);
        // transform_to_send.rotate_local_x(-1. * pitch);
        // transform_to_send.rotate(Quat::from_rotation_y(PI));
        // transform_to_send.rotation.write_to_slice(&mut r);
        // player_attributes.position = transform_to_send.translation.into();
        // player_attributes.rotation = r;
        // player_attributes.pitch = pitch;
        // info!(" pitch: {}", pitch);
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
        // camera_transform.rotate_y(yaw);
        // camera_transform.rotate_local_x(    pitch);
        // p.0 = pitch;
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
        proposed_position.0.y = 0.9 / 2.;
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
    ass: Res<AssetServer>,
) {
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let player_body = meshes.add(Cuboid::new(0.37, 1.4, 0.37));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    // commands.spawn((
    //     WorldModelCamera,
    //     Camera3dBundle {
    //         projection: PerspectiveProjection {
    //             fov: 70.0_f32.to_radians(),
    //             ..default()
    //         }
    //         .into(),
    //         ..default()
    //     },
    //     RenderLayers::from_layers(&[VIEW_MODEL_RENDER_LAYER, DEFAULT_RENDER_LAYER]),
    // ));

    commands
        .spawn((
            MyPlayer,
            Collider {
                size: Vec3::new(1.0, 1.0, 1.0),
            }, // Add collider here
            SpatialBundle {
                transform: Transform::from_xyz((23. / 2.) + 1.5, 0.9 / 2., (14.0 / 2.) + 1.5),
                ..default()
            },
        ))
        .with_children(|parent| {
            //this is the camera of the global view

            parent.spawn((
                WorldModelCamera,
                Camera3dBundle {
                    projection: PerspectiveProjection {
                        fov: 90.0_f32.to_radians(),
                        ..default()
                    }
                    .into(),
                    transform: Transform::from_xyz(-0.12, 0.5, 0.),
                    ..default()
                },
                RenderLayers::from_layers(&[VIEW_MODEL_RENDER_LAYER, DEFAULT_RENDER_LAYER]),
            ));

            // Spawn view model camera.
            // parent.spawn((
            //     Camera3dBundle {
            //         camera: Camera {
            //             // Bump the order to render on top of the world model.
            //             order: 1,
            //             ..default()
            //         },
            //         projection: PerspectiveProjection {
            //             fov: 70.0_f32.to_radians(),
            //             ..default()
            //         }
            //         .into(),
            //         ..default()
            //     },
            //     // Only render objects belonging to the view model.
            //     RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            // ));

            // Spawn the player's right arm.
            // parent.spawn((
            //     MaterialMeshBundle {
            //         mesh: arm,
            //         material: arm_material.clone(),
            //         transform: Transform::from_xyz(0.08, 0.5, -0.25),
            //         ..default()
            //     },
            //     // Ensure the arm is only rendered by the view model camera.
            //     RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
            //     // The arm is free-floating, so shadows would look weird.
            //     NotShadowCaster,
            // ));
            let riffle = ass.load("m4_carbine_rifle.glb#Scene0");
            parent.spawn((
                SceneBundle {
                    scene: riffle,
                    transform: Transform {
                        scale: Vec3 {
                            x: 0.15,
                            y: 0.15,
                            z: 0.15,
                        },
                        ..Transform::from_xyz(0.08, 0.5, -0.2)
                    },
                    ..default()
                },
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
            parent.spawn((
                PlayerBody,
                MaterialMeshBundle {
                    mesh: player_body,
                    material: arm_material,
                    ..default()
                },
                // Ensure the arm is only rendered by the view model camera.
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                Collider {
                    size: Vec3::new(0.37, 1.4, 0.37),
                },
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
        });
}
pub fn collision_detection_system(
    mut commands: Commands,
    player_query: Query<(&Transform, &Collider), (With<MyPlayer>, Without<Projectile>)>,
    projectile_query: Query<(Entity, &mut Transform, &Projectile), With<Projectile>>,
    mut live: ResMut<Live>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let (player_transform, player_collider) = player_query.single();
    let player_position = player_transform.translation;

    for (projectile_entity, projectile_transform, projectile) in projectile_query.iter() {
        let projectile_position = projectile_transform.translation;
        let projectile_direction = projectile.direction;
        let projectile_length = 2.0; // Longueur du projectile (définie dans spawn_projectile)

        // Calculer la distance entre le joueur et le point le plus proche du projectile
        let closest_point = closest_point_on_line(
            player_position,
            projectile_position,
            projectile_position + projectile_direction * projectile_length,
        );
        let distance = (closest_point - player_position).length();

        // Vérifier si la distance est inférieure au rayon du joueur plus la moitié de la longueur du projectile
        if distance
            <= (player_collider
                .size
                .x
                .max(player_collider.size.y)
                .max(player_collider.size.z)
                + projectile_length / 2.0)
        {
            // Collision détectée !
            println!("Collision détectée !");

            live.0 -= 1; // On diminue la vie du joueur
            if live.0 <= 0 {
                next_state.set(GameState::Game0ver);
            }

            // Supprimez le projectile
            commands.entity(projectile_entity).despawn();
        }
    }
}

// Fonction pour trouver le point le plus proche sur une ligne
fn closest_point_on_line(point: Vec3, line_start: Vec3, line_end: Vec3) -> Vec3 {
    let line_direction = (line_end - line_start).normalize();
    let projection = (point - line_start).dot(line_direction);
    let closest_point = line_start + projection * line_direction;
    closest_point
}
pub fn handle_camera(
    player_transform: Query<&Transform, (With<MyPlayer>, Without<WorldModelCamera>)>,
    mut camera_transform: Query<&mut Transform, (Without<MyPlayer>, With<WorldModelCamera>)>,
) {
    if let Ok(player_transform) = player_transform.get_single() {
        if let Ok(mut camera_transform) = camera_transform.get_single_mut() {
            let backup_distance = camera_transform.back().as_vec3() * 1.5;
            camera_transform.translation = player_transform.translation + backup_distance;
        }
    }
}

pub fn handle_player_spawn_event_system(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    mut graphs: ResMut<Assets<AnimationGraph>>,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
    mut spawn_events: EventReader<PlayerSpawnEvent>,
    mut map_client_entities: ResMut<ClientEntity>,
    // mut waiting_entity: ResMut<WaitingEntity>,
     waiting_text_entity_query: Query<Entity , With<WaitingText>>
) {
    let mut graph = AnimationGraph::new();
    let animations = graph
        .add_clips(
            [GltfAssetLabel::Animation(0).from_asset("soldier_aiming_idle.glb")]
                .into_iter()
                .map(|path| asset_server.load(path)),
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
    for event in spawn_events.read() {
        info!("Handling player spawn event: {:?}", event.0);
        let client_id = event.0;

        let scene_handle = asset_server.load("soldier_aiming_idle.glb#Scene0");
        // let transform = Transform::from_rotation(Quat::from_rotation_y(180.0));

        let entity = commands
            .spawn((
                SceneBundle {
                    scene: scene_handle,
                    transform: Transform {
                        scale: Vec3 {
                            x: 0.7,
                            y: 0.7,
                            z: 0.7,
                        },
                        rotation: Quat::from_rotation_y(180.0),
                        ..default()
                    },
                    ..default()
                },
                PlayerEntity(client_id),
            ))
            .id();
        map_client_entities.0.insert(client_id, entity);
    }
    if let Ok(entity) = waiting_text_entity_query.get_single() {
        commands.entity(entity).despawn_recursive();
      

        println!("got something : {}", entity);
        // waiting_entity.0 = None;
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
                // let x_rotation = Quat::from_rotation_x(-2. * player_sync.pitch);
                transform.translation = new_position.into();

                transform.rotation = Quat::from_array(new_rotation);
                transform.rotate_y(PI);

                // transform.rotation *= x_rotation;
                // transform.
                // transform.rotate(Quat::from_rotation_x(-player_sync.pitch));
                // transform.rotate_ = -transform.rotate_local;
                // transform.rotate_local_x(player_sync.pitch);
                transform.translation.y = 0.;

                found = true;
            }
        }

        if !found {
            info!("Spawning player {}: {:?}", client_id, player_sync.position);
            spawn_events.send(PlayerSpawnEvent(*client_id));
        }
    }
}
pub fn handle_shoot_event_system(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut shoot_events: EventReader<ShootEvent>,
    my_clinet_id: Res<MyClientId>,
) {
    for event in shoot_events.read() {
        let client_id = event.0;
        let projectile_properties = event.1.clone();
        if client_id == my_clinet_id.0 {
            continue;
        }
        spawn_projectile_by_projectile_properties(
            &mut commands,
            &mut meshes,
            &mut materials,
            projectile_properties.clone(),
        );
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
    mut client: ResMut<RenetClient>,
) {
    for (interaction, mut color) in &mut interaction_query {
        match *interaction {
            Interaction::Pressed => {
                *color = Color::srgb(0.35, 0.75, 0.35).into();
                let message = bincode::serialize(&ClientMessage::PlayerStartTheGame).unwrap();

                client.send_message(DefaultChannel::ReliableOrdered, message);

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

pub fn spawn_game_over_image(mut commands: Commands, asset_server: Res<AssetServer>) {
    let game_over_image: Handle<Image> = asset_server.load("game_over.png");
    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            image: UiImage::new(game_over_image),
            ..default()
        },
        // GameOverElement,
    ));
}
pub fn spawn_i_won_image(mut commands: Commands, ass: Res<AssetServer>) {
    let i_won_image: Handle<Image> = ass.load("congrat.png");
    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                position_type: PositionType::Absolute,
                margin: UiRect::all(Val::Auto),
                ..default()
            },
            image: UiImage::new(i_won_image),
            ..default()
        },
        // IWonElement,
    ));
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
pub fn spawn_projectile(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<MyPlayer>>,
    mut client: ResMut<RenetClient>,
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
    let mut kiki = player_transform.clone();
    kiki.translation.x += 0.08;
    kiki.translation.y += 0.5;
    kiki.translation.z += -0.25;
    let direction = kiki.forward().as_vec3();
    let translation = kiki.translation + direction * 0.5;
    let rotation = kiki.rotation * Quat::from_rotation_x(std::f32::consts::FRAC_PI_2);
    commands.spawn((
        Projectile {
            direction: direction.clone(),
            speed: projectile_speed,
        },
        MaterialMeshBundle {
            mesh: projectile,
            material: projectile_material,
            transform: Transform {
                translation: translation.clone(),
                rotation: rotation.clone(),
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    let projectile_properties = ProjectileProperties {
        position: translation.into(),
        rotation: rotation.to_array(),
        direction: direction.into(),
    };
    let message = bincode::serialize(&ClientMessage::Shoot(projectile_properties)).unwrap();
    client.send_message(DefaultChannel::ReliableOrdered, message);
}
pub fn spawn_projectile_by_projectile_properties(
    commands: &mut Commands,
    meshes: &mut ResMut<Assets<Mesh>>,
    materials: &mut ResMut<Assets<StandardMaterial>>,
    projectile_properties: ProjectileProperties,
) {
    let dir = Vec3::new(
        projectile_properties.direction[0],
        projectile_properties.direction[1],
        projectile_properties.direction[2],
    );
    let translation = Vec3::new(
        projectile_properties.position[0],
        projectile_properties.position[1],
        projectile_properties.position[2],
    );
    let rotation = Quat::from_array(projectile_properties.rotation);
    let projectile = Projectile {
        direction: dir,
        speed: 20.,
    };
    let projectile_mesh = meshes.add(Cylinder::new(0.05, 2.0));
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
        projectile,
        MaterialMeshBundle {
            mesh: projectile_mesh,
            material: projectile_material,
            transform: Transform {
                translation,
                rotation,
                ..Default::default()
            },
            ..Default::default()
        },
    ));
    // spawn_projectile(commands, meshes, materials, projectile);
}
pub fn shoot_system(
    mouse_button_input: Res<ButtonInput<MouseButton>>,
    commands: Commands,
    meshes: ResMut<Assets<Mesh>>,
    materials: ResMut<Assets<StandardMaterial>>,
    player_query: Query<&Transform, With<MyPlayer>>,
    client: ResMut<RenetClient>,
) {
    if mouse_button_input.just_pressed(MouseButton::Left) {
        // Tirer un projectile
        println!("Pew!");
        spawn_projectile(commands, meshes, materials, player_query, client);
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
fn check_collision_for_projectile(cube_pos: Vec3, cube_size: Vec3, cylinder_pos: Vec3) -> bool {
    let cylinder_radius = 0.05;
    let cylinder_height = 2.0;
    // Vérification de la collision sur l'axe Y (hauteur)
    let y_collision = (cube_pos.y - cube_size.y / 2.0 <= cylinder_pos.y + cylinder_height / 2.0)
        && (cube_pos.y + cube_size.y / 2.0 >= cylinder_pos.y - cylinder_height / 2.0);

    if !y_collision {
        return false;
    }

    // Vérification de la collision sur le plan XZ
    let dx = (cube_pos.x - cylinder_pos.x).abs();
    let dz = (cube_pos.z - cylinder_pos.z).abs();

    if dx > (cube_size.x / 2.0 + cylinder_radius) || dz > (cube_size.z / 2.0 + cylinder_radius) {
        return false;
    }

    if dx <= cube_size.x / 2.0 || dz <= cube_size.z / 2.0 {
        return true;
    }

    let corner_distance_sq = (dx - cube_size.x / 2.0).powi(2) + (dz - cube_size.z / 2.0).powi(2);

    corner_distance_sq <= cylinder_radius.powi(2)
}

pub fn collision_detection_system_for_cube_and_projectile(
    mut commands: Commands,
    player_query: Query<(Entity, &mut Transform, &MyPlayer), Without<Projectile>>,
    projectile_query: Query<(Entity, &Transform, &Projectile), Without<PlayerBody>>,
    mut live: ResMut<Live>,
    mut next_state: ResMut<NextState<GameState>>,
    mut client: ResMut<RenetClient>,
) {
    let (player_entity, player_transform, player) = player_query.single();

    for (projectile_entity, projectile_transform, projectile) in projectile_query.iter() {
        // player_transform.translation.y = 1.4;
        let mut player_tranform = player_transform.clone();
        player_tranform.translation.y -= 0.5;
        if check_collision_for_projectile(
            player_transform.translation,
            Vec3::new(0.3, 1.4, 0.3),
            projectile_transform.translation,
        ) {
            println!("Collision détectée !");
            live.0 -= 1; // On diminue la vie du joueur
            if live.0 <= 0 {
                next_state.set(GameState::Game0ver);
                let message = bincode::serialize(&ClientMessage::ImDead).unwrap();
                client.send_message(DefaultChannel::ReliableOrdered, message);
            }
            // Ici, vous pouvez ajouter la logique pour gérer la collision
            // Par exemple, supprimer le projectile et infliger des dégâts au joueur
            commands.entity(projectile_entity).despawn();

            // Vous pouvez également ajouter un événement de collision si nécessaire
        }
    }
}

pub fn player_animation(
    mut commands: Commands,
    animations: Res<Animations>,
    mut players: Query<(Entity, &mut AnimationPlayer), Added<AnimationPlayer>>,
) {
    for (entity, mut player) in &mut players {
        let mut transitions = AnimationTransitions::new();

        transitions
            .play(&mut player, animations.animations[0], Duration::ZERO)
            .repeat();

        commands
            .entity(entity)
            .insert(animations.graph.clone())
            .insert(transitions);
    }
}

pub fn despawn_player(entity: Entity, commands: &mut Commands) {
    //despawn_player
    commands.entity(entity).despawn();
}
