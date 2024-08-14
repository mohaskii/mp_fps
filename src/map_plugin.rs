use bevy::{prelude::*, sprite::MaterialMesh2dBundle};
pub struct MapPlugin;

#[derive(Component)]
pub struct Wall;
#[derive(Component)]
pub struct MiniMapPlayer;

#[derive(Component, Clone)]
pub struct Collider {
    pub size: Vec3,
}
    impl Plugin for MapPlugin {
        fn build(&self, app: &mut App) {
            app.add_systems(Startup, spawn_world_model);
        }
    }
fn spawn_world_model(
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
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
        vec![1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 1, 0, 0, 1, 0, 0, 0, 0, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 1, 1, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 0, 1, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 1, 0, 1, 0, 1, 1, 1, 0, 1, 0, 1, 1, 0, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 0, 1, 0, 0, 1, 1, 1, 1, 1, 1, 1, 0, 0, 1, 1, 0, 1, 1, 0, 0, 1],
        vec![1, 0, 0, 1, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1, 0, 0, 0, 1],
        vec![1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 1],
        vec![1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1],
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
    let mini_map_parent = commands.spawn(TransformBundle::default()).id();
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
