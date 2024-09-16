use bevy::{
    ecs::{
        event::EventReader,
        system::{Res, ResMut},
    },
    log::info,
    math::Quat,
    prelude::{EventWriter, NextState},
};
use mp_fps::{ClientMessage, PlayerAttributes, ServerMessage};
use renet::{ClientId, DefaultChannel, RenetServer, ServerEvent};

use crate::{
    event::ShootEvent,
    resources::{PlayerInTheGame, PlayerLobby},
    states::ServerState,
    SERVER_ADDR,
};

pub fn setup_system() {
    info!("Server started on {}", SERVER_ADDR);
}

pub fn send_message_system(mut server: ResMut<RenetServer>, player_lobby: Res<PlayerLobby>) {
    let chanel = DefaultChannel::Unreliable;
    let lobby = player_lobby.0.clone();
    let event = mp_fps::ServerMessage::LobbySync(lobby);
    let message = bincode::serialize(&event).unwrap();
    server.broadcast_message(chanel, message);
}

// fn print_lobby(lobby: &PlayerLobby) {
//     info!("Lobby:");
//     info!("------");

//     if lobby.0.is_empty() {
//         info!("Empty");
//         return;
//     }

//     for (client_id, player) in lobby.0.iter() {
//         info!("Client {}: {:?}", client_id, player);
//     }
// }

pub fn receive_message_system(
    mut server: ResMut<RenetServer>,
    mut player_lobby: ResMut<PlayerLobby>,
    // mut shoot_events: EventWriter<ShootEvent>,
) {
    for client_id in server.clients_id() {
        let message = server.receive_message(client_id, DefaultChannel::Unreliable);
        if let Some(message) = message {
            let message_type = bincode::deserialize(&message).unwrap();
            match message_type {
                // ClientMessage::Shoot(projectile_properties) => {
                //     shoot_events.send(ShootEvent(client_id, projectile_properties));
                // },
                ClientMessage::PlayerAttributes(player_attributes) => {
                    player_lobby.0.insert(client_id, player_attributes);
                }
                _ => {}
            }
            // let player: PlayerAttributes = bincode::deserialize(&message).unwrap();
            // player_lobby.0.insert(client_id, player);
        }
    }
}
pub fn receive_message_system_reliable(
    mut server: ResMut<RenetServer>,
    mut shoot_events: EventWriter<ShootEvent>,
    mut player_in_game: ResMut<PlayerInTheGame>,
    // mut next_state: ResMut<NextState<ServerState>>,
    // mut server: ResMut<RenetServer>
) {
    for client_id in server.clients_id() {
        let message = server.receive_message(client_id, DefaultChannel::ReliableOrdered);
        if let Some(message) = message {
            let message_type = bincode::deserialize(&message).unwrap();
            match message_type {
                ClientMessage::Shoot(projectile_properties) => {
                    shoot_events.send(ShootEvent(client_id, projectile_properties));
                }
                ClientMessage::ImDead => {
                    player_in_game.0 -= 1;
                    let message = ServerMessage::DaNiggaDie(client_id);
                    let message = bincode::serialize(&message).unwrap();
                    server.disconnect(client_id);
                    server.broadcast_message_except(
                        client_id,
                        DefaultChannel::ReliableOrdered,
                        message,
                    );
                    if player_in_game.0 != 1 {
                        return;
                    }

                    let message = ServerMessage::YouWon;
                    let message = bincode::serialize(&message).unwrap();
                    server.broadcast_message_except(
                        client_id,
                        DefaultChannel::ReliableOrdered,
                        message,
                    );
                    // server.disconnect_all()
                    // server.disconnect(client_id);
                }
                ClientMessage::PlayerStartTheGame => {
                    player_in_game.0 += 1;

                    info!("a player started the game");
                    if player_in_game.0 > 1 {
                        let message = ServerMessage::GameStarted;
                        let message = bincode::serialize(&message).unwrap();
                        server.broadcast_message(DefaultChannel::ReliableOrdered, message);
                    }
                }

                _ => {}
            }
            // let player: PlayerAttributes = bincode::deserialize(&message).unwrap();
            // player_lobby.0.insert(client_id, player);
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
                let mut r = [0.0; 4];
                Quat::default().write_to_slice(&mut r);
                player_lobby.0.insert(
                    *client_id,
                    PlayerAttributes {
                        position: [0.0, 0.0, 0.0],
                        rotation: r,
                        pitch: 0.0,
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
            ServerEvent::ClientDisconnected {
                client_id,
                reason: _,
            } => {
                player_lobby.0.remove(client_id);
                let message =
                    bincode::serialize(&mp_fps::ServerMessage::PlayerLeave(*client_id)).unwrap();
                server.broadcast_message(DefaultChannel::ReliableOrdered, message);
            }
        }
    }
}
pub fn handle_shoot_event_system(
    mut server: ResMut<RenetServer>,
    mut shoot_events: EventReader<ShootEvent>,
) {
    let event = shoot_events.read().next();
    if let Some(event) = event {
        let message =
            bincode::serialize(&mp_fps::ServerMessage::Shoot(event.0, event.1.clone())).unwrap();
        server.broadcast_message(DefaultChannel::ReliableOrdered, message);
    }
}
