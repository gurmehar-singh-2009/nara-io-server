#![feature(let_chains)]
#![feature(try_blocks)]
#![feature(yeet_expr)] // not even syntax highlighting supports this :(
#![feature(adt_const_params)]
#![feature(const_param_ty_trait)]
#![feature(stmt_expr_attributes)]
#![allow(incomplete_features)]

use std::sync::Arc;

use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::{accept_async_with_config, tungstenite::Result};

use dashmap::DashMap;
use ed25519_dalek::SigningKey;
use shared::ServerPacket;
use tokio_tungstenite::tungstenite::protocol::WebSocketConfig;
use x25519_dalek::PublicKey;

mod game;
mod scripting;

use crate::{
    fs::tank_defs::aaa,
    game::game_state::GameState,
    net::{ClientConnection, ConnectionState},
};

mod anti_cheat;
mod entities;
mod errors;
mod fs;
mod net;

const PUBLIC_KEY_BYTES: &[u8; 32] = include_bytes!("../server_key.bin");

pub fn get_server_signing_key() -> SigningKey {
    SigningKey::from_bytes(PUBLIC_KEY_BYTES)
}

#[tokio::main]
async fn main() -> Result<(), String> {
    let addr = "127.0.0.1:8080".to_string();
    let socket_config = WebSocketConfig::default()
        .max_message_size(Some(4096))
        .max_frame_size(Some(1024));
    let signing_key = Arc::new(get_server_signing_key());

    let id_to_conn: DashMap<u32, ClientConnection<{ ConnectionState::Authenticated }>> =
        DashMap::new();

    let game_state = GameState::new().unwrap();

    let try_socket = TcpListener::bind(addr).await;
    let listener = try_socket.expect("Failed to bind");

    let mut current_id = 0;

    // Read incoming socket requests.
    while let Ok((stream, addr)) = listener.accept().await {
        let ws_stream = accept_async_with_config(stream, Some(socket_config))
            .await
            .unwrap();
        let (mut write, mut read) = ws_stream.split();

        // Initialze Read/Write channels for the socket.
        // We need to pass this into the `ClientConnection` struct.
        let (socket_send_tx, mut socket_recv_rx) =
            tokio::sync::mpsc::unbounded_channel::<Vec<u8>>();
        // Which is defined here.
        let client_connection = ClientConnection::new(current_id, socket_send_tx);

        // Setup channel read/write system.
        tokio::spawn(async move {
            while let Some(msg) = socket_recv_rx.recv().await {
                if write.send(msg.into()).await.is_err() {
                    break;
                }
            }
        });

        // Increment ID. I don't care about reusing.
        current_id += 1;

        let signing_key = signing_key.clone();
        tokio::spawn(async move {
            // Create an authenticated connection instance.
            let authenticated_connection = match read.next().await {
                Some(Ok(msg)) => {
                    if !msg.is_binary() {
                        return;
                    }

                    let msg = msg.into_data();

                    let decoded = match bitcode::decode::<ServerPacket>(&msg) {
                        Ok(data) => data,

                        // Decoding should never fail, disconnect the client.
                        Err(_) => return,
                    };

                    match decoded {
                        // We expect the first packet to be the handshake initiation.
                        // Otherwise we kaboom connection.
                        // TODO: Blacklist IPs that attempt to connect without handshaking first.
                        ServerPacket::Handshake(payload) => client_connection
                            .respond_handshake(
                                &PublicKey::from(<[u8; 32]>::try_from(&payload.0[..32]).unwrap()),
                                &signing_key,
                            )
                            .unwrap(),

                        _ => return,
                    }
                }

                _ => return,
            };

            // Now that the handshake has been established previously
            // we can continously read the socket stream.
            while let Some(Ok(msg)) = read.next().await {
                if msg.is_binary() {
                    let decoded = bitcode::decode::<ServerPacket>(&msg.into_data()).unwrap();
                }
            }
        });
    }

    aaa();

    do yeet "wow!".to_string()
}
