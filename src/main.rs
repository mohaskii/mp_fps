use bevy::color::palettes::tailwind;
use bevy::input::mouse::MouseMotion;
use bevy::pbr::NotShadowCaster;
use bevy::prelude::*;
use bevy::render::view::RenderLayers;
use bevy::window::{CursorGrabMode, PrimaryWindow};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .init_resource::<ProposedPlayerPosition>()
        .init_resource::<HasCollision>() // Ajoutez cette ligne
        .add_systems(
            Startup,
            (
                spawn_view_model,
                spawn_world_model,
                spawn_lights,
                spawn_text,
                spawn_crosshair,
                cursor_grab,
            ),
        )
        .add_systems(
            Update,
            (
                move_player,
                player_position_control,
                check_collision_system,
                apply_movement,
            )
                .chain(),
        ) // Modifiez cette ligne
        .run();
}
#[derive(Component)]
struct Wall;

#[derive(Component, Clone)]
struct Collider {
    size: Vec3,
}

#[derive(Debug, Component)]
struct Player;

#[derive(Debug, Component)]
struct WorldModelCamera;
#[derive(Resource, Default)]
struct ProposedPlayerPosition(Vec3);
#[derive(Resource, Component, Default)]
struct HasCollision(bool);
/// Used implicitly by all entities without a `RenderLayers` component.
/// Our world model camera and all objects other than the player are on this layer.
/// The light source belongs to both layers.
const DEFAULT_RENDER_LAYER: usize = 0;

/// Used by the view model camera and the player's arm.
/// The light source belongs to both layers.
const VIEW_MODEL_RENDER_LAYER: usize = 1;
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

fn spawn_view_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let arm = meshes.add(Cuboid::new(0.1, 0.1, 0.5));
    let arm_material = materials.add(Color::from(tailwind::TEAL_200));

    commands
        .spawn((
            Player,
            Collider {
                size: Vec3::new(1.0, 1.0, 1.0),
            }, // Ajoutez le collider ici
            SpatialBundle {
                transform: Transform::from_xyz(0.0, 10.0, 0.0),
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
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
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
                RenderLayers::layer(VIEW_MODEL_RENDER_LAYER),
                // The arm is free-floating, so shadows would look weird.
                NotShadowCaster,
            ));
        });
}

fn spawn_world_model(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Définir la carte du labyrinthe
    let maze = vec![
        vec![0, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![0, 0, 1, 1, 1, 1, 1, 1, 0, 1],
        vec![0, 0, 1, 0, 0, 0, 0, 1, 0, 1],
        vec![0, 0, 1, 0, 1, 1, 0, 1, 0, 1],
        vec![0, 0, 1, 0, 1, 1, 0, 1, 0, 1],
        vec![0, 0, 1, 0, 0, 0, 0, 1, 0, 1],
        vec![0, 0, 1, 1, 1, 1, 1, 1, 0, 1],
        vec![0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
    ];

    // Calculer la taille du sol en fonction de la taille de la matrice du labyrinthe
    let maze_width = maze[0].len() as f32;
    let maze_height = maze.len() as f32;

    // Créer le sol
    let floor = meshes.add(Plane3d::new(Vec3::Y, Vec2::new(maze_width, maze_height)));
    let floor_material = materials.add(Color::WHITE);

    commands.spawn(PbrBundle {
        mesh: floor,
        material: floor_material,
        transform: Transform::from_xyz(maze_width / 2.0, 0.0, maze_height / 2.0),
        ..default()
    });

    // Créer les cubes pour le labyrinthe
    let cube = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let cube_material = materials.add(Color::BLACK);

    for (z, row) in maze.iter().enumerate() {
        for (x, &cell) in row.iter().enumerate() {
            if cell == 1 {
                commands.spawn((
                    PbrBundle {
                        mesh: cube.clone(),
                        material: cube_material.clone(),
                        transform: Transform::from_xyz(x as f32 + 0.5, 0.5, z as f32 + 0.5),
                        ..default()
                    },
                    Wall,
                    Collider {
                        size: Vec3::new(1.0, 1.0, 1.0),
                    }, // Ajoutez le collider ici
                ));
            }
        }
    }
}

fn spawn_lights(mut commands: Commands) {
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

fn spawn_text(mut commands: Commands) {
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

fn move_player(
    mut mouse_motion: EventReader<MouseMotion>,
    mut player: Query<&mut Transform, With<Player>>,
) {
    let mut transform = player.single_mut();
    for motion in mouse_motion.read() {
        let yaw = -motion.delta.x * 0.003;
        let pitch = -motion.delta.y * 0.002;
        // Order of rotations is important, see <https://gamedev.stackexchange.com/a/136175/103059>
        transform.rotate_y(yaw);
        transform.rotate_local_x(pitch);
    }
}

fn player_position_control(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    time: Res<Time>,
    player_query: Query<&Transform, With<Player>>,
    mut proposed_position: ResMut<ProposedPlayerPosition>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        let speed = 10.0;
        let delta = time.delta_seconds();

        let mut movement = Vec3::ZERO;

        if keyboard_input.pressed(KeyCode::ArrowUp) {
            movement += player_transform.forward().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowDown) {
            movement += player_transform.back().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowLeft) {
            movement += player_transform.left().as_vec3();
        }
        if keyboard_input.pressed(KeyCode::ArrowRight) {
            movement += player_transform.right().as_vec3();
        }

        if movement != Vec3::ZERO {
            movement = movement.normalize();
        }

        proposed_position.0 = player_transform.translation + movement * speed * delta;
        proposed_position.0.y = 1.0;
    }
}
fn check_collision_system(
    mut query: ParamSet<(
        Query<&Collider, With<Player>>,
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
fn apply_movement(
    mut query: Query<&mut Transform, With<Player>>,
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
    }
}
fn spawn_crosshair(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let crosshair :Handle<Image> = asset_server.load("crosshair.png");
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
fn cursor_grab(
    mut q_windows: Query<&mut Window, With<PrimaryWindow>>,
) {
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
