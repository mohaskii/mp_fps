use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    math::Dir3,
    prelude::Query,
};
use mp_fps::PlayerAttributes;
use renet::{DefaultChannel, RenetServer, ServerEvent};

use crate::{resources::PlayerLobby, SERVER_ADDR};

pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>) {
    let chanel = DefaultChannel::Unreliable;
    let lobby = player_lobby.0.clone();
    let event = mp_fps::ServerMessage::LobbySync(lobby);
    let message = bincode::serialize(&event).unwrap();
    // print_lobby(&player_lobby);
    server.broadcast_message(chanel, message);
}

pub fn print_lobby(lobby: Res<PlayerLobby>) {
    if lobby.0.is_empty() {
        info!("Empty");
        return;
    }

    info!("Lobby:");
    info!("------");

    for (client_id, player) in lobby.0.iter() {
        info!("Client {}: {:?}", client_id, player);
    }
}

pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
) {
    for client_id in server.clients_id() {
        let message = server.receive_message(client_id, DefaultChannel::Unreliable);
        if let Some(message) = message {
            let player: PlayerAttributes = bincode::deserialize(&message).unwrap();
            player_lobby.0.insert(client_id, player);
        }
    }
}

pub fn handle_events_system(
    mut server: ResMut<RenetServer>,
    mut server_events: EventReader<ServerEvent>,
    mut player_lobby: ResMut<PlayerLobby>,
) {
    for event in server_events.read() {
        match event {
            ServerEvent::ClientConnected { client_id } => {
                println!("Client {client_id} connected");
                player_lobby.0.insert(
                    *client_id,
                    PlayerAttributes {
                        position: [0.0, 0.0, 0.0],
                        forward: Dir3::Z.to_array(),
                    },
                );
                let message =
                    bincode::serialize(&mp_fps::ServerMessage::PlayerJoin(*client_id)).unwrap();
                server.broadcast_message_except(
                    *client_id,
                    DefaultChannel::ReliableOrdered,
                    message,
                );
            }
            ServerEvent::ClientDisconnected { client_id, reason } => {
                println!("Client {client_id} disconnected: {reason}");
                player_lobby.0.remove(client_id);
                let message =
                    bincode::serialize(&mp_fps::ServerMessage::PlayerLeave(*client_id)).unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
        }
    }
}
