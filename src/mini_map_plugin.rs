use bevy::{color::palettes::basic::PURPLE, prelude::*, sprite::MaterialMesh2dBundle};
use crate::Player;
pub struct MiniMapPlugin;

impl Plugin for MiniMapPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_mini_map)
           .add_systems(Update, update_mini_map);
    }
}

#[derive(Component)]
pub struct MiniMap;

#[derive(Component)]
pub struct MiniMapPlayer;

fn setup_mini_map(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Spawn the mini-map camera
    commands.spawn(Camera2dBundle::default());

    // Spawn the mini-map background
    // commands.spawn((
    //     MaterialMesh2dBundle {
    //         mesh: meshes.add(Rectangle::default()).into(),
    //         transform: Transform::from_xyz(-100., 0., 1.).with_scale(Vec3::splat(128.)),
    //         material: materials.add(Color::from(PURPLE)),
    //         ..default()
    //     },
    //     MiniMap,
    // ));

    // Spawn the player icon on the mini-map
   
}
fn update_mini_map(
    player_query: Query<&Transform, (With<Player>, Without<MiniMapPlayer>)>,
    mut mini_map_player_query: Query<&mut Transform, (With<MiniMapPlayer>, Without<Player>)>,
) {
    if let Ok(player_transform) = player_query.get_single() {
        if let Ok(mut mini_map_player_transform) = mini_map_player_query.get_single_mut() {
            // Mettre à jour la position de l'icône du joueur sur la mini-carte
            mini_map_player_transform.translation.x = player_transform.translation.x / 10.0;
            mini_map_player_transform.translation.y = player_transform.translation.z / 10.0;
        }
    }
}