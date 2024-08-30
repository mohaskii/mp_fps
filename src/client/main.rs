use std::{
    collections::HashMap,
    net::{SocketAddrV4, UdpSocket},
    time::SystemTime,
};

use bevy::{
    app::{App, Startup, Update},
    log::info,
    DefaultPlugins,
};
use bevy_renet::{transport::NetcodeClientPlugin, RenetClientPlugin};
use renet::{
    transport::{ClientAuthentication, NetcodeClientTransport},
    ClientId, ConnectionConfig, RenetClient,
};
use states::GameState;

use crate::{
    resources::{MyClientId, PlayerEntities},
    systems::{
        handle_lobby_sync_event_system, handle_player_spawn_event_system, receive_message_system,
        send_message_system, setup_system, update_player_movement_system,
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
        DefaultPlugins,
        MapPlugin,
        UiPlugin,
    ));
    // Get the server address from the command line arguments
    let server_address = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:5000".to_string());
    let server_address: SocketAddrV4 = server_address.parse().expect("Invalid server address");

    // renet client
    let client = RenetClient::new(ConnectionConfig::default());
    app.insert_resource(client);

    let client_id = rand::random::<u64>();
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
    
    let  projectile_buffer = ProjectileBuffer::default();
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
            handle_player_spawn_event_system,
            handle_lobby_sync_event_system,
            shoot_system, // Système pour tirer
            projectile_movement_system,
            handle_shoot_event_system,
            player_animation
        )
            .chain()
            .run_if(in_state(GameState::Playing)),
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

    info!(
        "Client {} started with server address {}",
        client_id, server_address
    );

    app.run();
}
