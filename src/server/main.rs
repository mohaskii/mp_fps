#![allow(warnings)]
use std::{
    collections::HashMap,
    net::UdpSocket,
    time::{Duration, SystemTime},
};
// use bevy_utils::default;

use bevy::{
    app::{App, Startup, Update},
    log::LogPlugin,
    prelude::AppExtStates,
    MinimalPlugins,
};
use bevy_renet::{transport::NetcodeServerPlugin, RenetServerPlugin};
use mp_fps::ProjectileProperties;
use renet::{
    transport::{NetcodeServerTransport, ServerAuthentication, ServerConfig},
    ChannelConfig, ConnectionConfig, RenetServer, SendType,
};
use resources::{PlayerInTheGame, PlayerLobby, ProjectileBuffer};
use systems::{
    handle_events_system, handle_shoot_event_system, receive_message_system,
    receive_message_system_reliable, send_message_system, setup_system,
};

mod event;
mod resources;
mod states;
mod systems;
use event::ShootEvent;
use states::ServerState;
const SERVER_ADDR: &str = "127.0.0.1:5000";

fn main() {
    let mut app = App::new();
    let channel_configs_server = vec![
        ChannelConfig {
            channel_id: 0,
            max_memory_usage_bytes: 20 * 1024 * 1024,
            send_type: SendType::Unreliable,
        },
        ChannelConfig {
            channel_id: 1,
            max_memory_usage_bytes: 20 * 1024 * 1024,
            send_type: SendType::ReliableUnordered {
                resend_time: Duration::from_millis(50),
            },
        },
        ChannelConfig {
            channel_id: 2,
            max_memory_usage_bytes: 20 * 1024 * 1024,
            send_type: SendType::ReliableOrdered {
                resend_time: Duration::from_millis(50),
            },
        },
    ];

    // base plugins
    app.add_plugins(MinimalPlugins);
    app.add_plugins(LogPlugin::default());
    app.add_plugins(RenetServerPlugin);
    // app.init_state::<ServerState>();

    // renet server
    let server = RenetServer::new(ConnectionConfig {
        available_bytes_per_tick: 60_000,
        server_channels_config: channel_configs_server.clone(),
        client_channels_config: channel_configs_server,
    });

    let projectile_buffer = ProjectileBuffer::default();
    app.insert_resource(projectile_buffer);
    app.insert_resource(server);

    app.add_plugins(NetcodeServerPlugin);
    let server_addr = SERVER_ADDR.parse().unwrap();
    let socket = UdpSocket::bind(server_addr).unwrap();
    let server_config = ServerConfig {
        current_time: SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap(),
        max_clients: 64,
        protocol_id: 0,
        public_addresses: vec![server_addr],
        authentication: ServerAuthentication::Unsecure,
    };
    let player_in_the_game = 0;
    app.insert_resource(PlayerInTheGame(player_in_the_game));

    // let c = ChannelConfig {
    //     // The id for the channel, must be unique within its own list,
    //     // but it can be repeated between the server and client lists.
    //     channel_id: 0,
    //     // Maximum number of bytes that the channel may hold without acknowledgement of messages before becoming full.
    //     max_memory_usage_bytes: 5 * 1024 * 1024, // 5 megabytes
    //     send_type
    // };
    let transport = NetcodeServerTransport::new(server_config, socket).unwrap();
    app.insert_resource(transport);
    app.add_event::<ShootEvent>();
    // game systems
    app.insert_resource(PlayerLobby(HashMap::default()));

    app.add_systems(Startup, setup_system);
    app.add_systems(Update, send_message_system);
    app.add_systems(Update, receive_message_system);
    app.add_systems(Update, handle_events_system);
    app.add_systems(Update, handle_shoot_event_system);
    app.add_systems(Update, receive_message_system_reliable);

    app.run();
}
