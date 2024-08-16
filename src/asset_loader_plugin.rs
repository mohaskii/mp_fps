use bevy::gltf::Gltf;
use bevy::{prelude::*, utils::HashMap};
use bevy_asset_loader::prelude::*;

#[derive(States, Clone, Eq, PartialEq, Default, Hash, Debug)]
pub enum AssetLoaderState {
    #[default]
    Loading,
    Done,
}

pub struct AssetLoaderPlugin;
impl Plugin for AssetLoaderPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MyAssets>()
            .init_state::<AssetLoaderState>()
            .add_loading_state(
                LoadingState::new(AssetLoaderState::Loading)
                    .continue_to_state(AssetLoaderState::Done)
                    .load_collection::<MyAssets>(),
            );
    }
}

// pub fn load(ass: Res<AssetServer>, mut command: Commands) {
//     let m: Handle<Gltf> = ass.load("player1.glb");
//     let mut gltf = HashMap::new();
//     gltf.insert("player1".to_string(), m);
//     command.insert_resource(MyAssets { gltf_files: gltf });
    
// }

#[derive(AssetCollection, Resource, Default)]
pub struct MyAssets {
    #[asset(paths("player1.glb"), collection(typed, mapped))]
    pub gltf_files: HashMap<String, Handle<Gltf>>,
}
