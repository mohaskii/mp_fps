#![allow(warnings)]
use std::{
    collections::HashMap,
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};
use bevy_rapier3d::prelude::*;

use bevy::{
    app::{App, Startup, Update},
    log::{info, LogPlugin},
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use mp_fps::{PlayerAttributes, P};
// use mp_fps::YP;
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
use states::GameState;

use crate::{
    resources::{MyClientId, PlayerEntities},
    systems::{
        handle_lobby_sync_event_system, handle_player_spawn_event_system, receive_message_system,
        send_message_system, setup_system, update_player_movement_system, prompt
    },
};

mod systems;
use systems::*;
mod components;
mod events;
mod resources;
use resources::*;
mod map_plugin;
use bevy::prelude::*;
use map_plugin::*;
mod states;
mod ui_plugin;
use bevy_dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin};
use ui_plugin::*;
fn main() {
    let mut app = App::new();

    // base plugins
    // app.add_plugins(RenetClientPlugin);
    // app.add_plugins(NetcodeClientPlugin);
    // app.add_plugins(DefaultPlugins);
    // app.add_plugins(MapPlugin);
    // app.add_plugins(UiPlugin);
    app.add_plugins((
        RenetClientPlugin,
        NetcodeClientPlugin,
        DefaultPlugins.set(LogPlugin {
            filter: "off".into(),
            level: bevy::log::Level::DEBUG,
            ..Default::default()
        }),
        MapPlugin,
        UiPlugin,
        RapierPhysicsPlugin::<NoUserData>::default(),
        RapierDebugRenderPlugin::default()
    ));

    app.add_plugins(FpsOverlayPlugin {
        config: FpsOverlayConfig {
            text_config: TextStyle {
                font_size: 50.0,
                color: Color::srgb(0.0, 1.0, 0.0),
                font: default(),
            },
        },
    });

    app.init_resource::<PlayerAttributes>();
    // Get the server address from the command line arguments
    let server_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5000".to_string());
    let server_address: SocketAddrV4 = server_address.parse().expect("Invalid server address");
    let username = prompt("Enter your username: ");

    // renet client
    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    let starting_lives = Live(5);
    let client_id = rand::random::<u64>();
    app.insert_resource(starting_lives);
    app.insert_resource(MyClientId(ClientId::from_raw(client_id)));
    app.insert_resource(PlayerEntities(HashMap::new()));
    app.insert_resource(ProposedPlayerPosition(Vec3::ZERO)); // Assuming ProposedPlayerPosition is a struct with a Vec3 field initialized to Vec3::ZERO
    app.insert_resource(HasCollision(false));

    let authentication = ClientAuthentication::Unsecure {
        server_addr: std::net::SocketAddr::V4(server_address),
        client_id,
        user_data: None,
        protocol_id: 0,
    };
    let socket = UdpSocket::bind("127.0.0.1:0").unwrap();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let transport = NetcodeClientTransport::new(current_time, authentication, socket).unwrap();

    app.insert_resource(transport);

    let projectile_buffer = ProjectileBuffer::default();
    app.insert_resource(projectile_buffer);

    // game events
    app.add_event::<events::PlayerSpawnEvent>();
    app.add_event::<events::PlayerDespawnEvent>();
    app.add_event::<events::PlayerMoveEvent>();
    app.add_event::<events::LobbySyncEvent>();
    app.add_event::<events::ShootEvent>();

    // game systems
    app.add_systems(
        Update,
        (
            move_player,
            player_position_control,
            check_collision_system,
            apply_movement,
            send_message_system,
            receive_message_system,
            shoot_system, // Système pour tirer
            projectile_movement_system,
            handle_shoot_event_system,
            detect_collision, 
            // handle_camera,           
        )
            .chain()
            .run_if(in_state(GameState::Playing).or_else(in_state(GameState::GameStarted))),
    );
    app.add_systems(
        Update,
        (
            handle_player_spawn_event_system,
            handle_lobby_sync_event_system,
            player_animation,
        )
            .chain()    
            .run_if(in_state(GameState::GameStarted)),
    );

    app.add_systems(
        OnEnter(GameState::Playing),
        (
            setup_system,
            spawn_lights,
            spawn_text,
            spawn_crosshair,
            // cursor_grab,
        ),
    );
    // app.add
    let map: ClientEntity = ClientEntity(HashMap::new());
    app.insert_resource(map);
    app.add_systems(OnEnter(GameState::Game0ver), spawn_game_over_image);
    app.add_systems(OnEnter(GameState::IWon), spawn_i_won_image);
    app.init_resource::<WaitingEntity>();
    app.init_resource::<GameAlreadyStarted>();

    info!(
        "Client {} started with server address {}",
        client_id, server_address
    );

    app.run();
}
