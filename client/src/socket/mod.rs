use std::{cell::RefCell, rc::Rc};

use chacha20poly1305::{ChaCha20Poly1305, Key, KeyInit, Nonce, aead::Aead};
use futures::{SinkExt, StreamExt, channel::mpsc::unbounded};
use gloo_net::websocket::{Message, futures::WebSocket};
use hkdf::Hkdf;
use js_sys::Uint8Array;
use sha2::Sha256;
use shared::packets::{
    PACKET_SEED, Packet,
    client_bound::AddEntityPacket,
    handshake::{HandShake, HandshakePacket},
    server_bound::SpawnReqPacket,
};
use wasm_bindgen::JsCast;
use wasm_bindgen_futures::spawn_local;
use web_sys::{Crypto, HtmlInputElement, window};
use x25519_dalek::{PublicKey, StaticSecret};

pub struct Socket {
    pub secret_key: StaticSecret,
}

pub fn get_random_bytes(crypto: &Crypto, buf: &mut [u8]) -> Result<(), js_sys::Error> {
    let temp_js_array = Uint8Array::new_with_length(buf.len() as u32);

    crypto
        .get_random_values_with_js_u8_array(&temp_js_array)
        .map_err(|_| js_sys::Error::new("Crypto error"))?;

    temp_js_array.copy_to(buf);

    Ok(())
}

fn nonce(counter: u32) -> Nonce {
    let mut bytes = [0u8; 12];

    bytes[..4].copy_from_slice(&counter.to_be_bytes());

    Nonce::from(bytes)
}

fn derive_ciphers(
    secret_key: &StaticSecret,
    server_public_key: &PublicKey,
) -> (ChaCha20Poly1305, ChaCha20Poly1305) {
    let shared_secret = secret_key.diffie_hellman(server_public_key);

    let hkdf = Hkdf::<Sha256>::new(None, shared_secret.as_bytes());

    let mut client_to_server = [0u8; 32];
    let mut server_to_client = [0u8; 32];

    hkdf.expand(b"client-to-server", &mut client_to_server)
        .expect("HKDF expansion failed");

    hkdf.expand(b"server-to-client", &mut server_to_client)
        .expect("HKDF expansion failed");

    let send_cipher = ChaCha20Poly1305::new(Key::from_slice(&client_to_server));

    let recv_cipher = ChaCha20Poly1305::new(Key::from_slice(&server_to_client));

    (send_cipher, recv_cipher)
}

impl Socket {
    pub async fn new(url: String) -> Self {
        let socket = WebSocket::open(&url).expect("Failed to connect to WebSocket server");

        let (mut send, mut recv) = socket.split();
        let (tx, mut rx) = unbounded::<Vec<u8>>();

        let window = window().expect("no window");
        let crypto = window.crypto().expect("crypto unavailable");

        let mut secret_bytes = [0u8; 32];

        get_random_bytes(&crypto, &mut secret_bytes).expect("crypto failed");

        let secret_key = StaticSecret::from(secret_bytes);
        let public_key = PublicKey::from(&secret_key);

        let mut handshake_bytes: HandShake = [0u8; 96];
        handshake_bytes[..32].copy_from_slice(public_key.as_bytes());

        get_random_bytes(&crypto, &mut handshake_bytes[32..]).expect("padding failed");

        let handshake = HandshakePacket::new(handshake_bytes, PACKET_SEED as u64);

        tx.unbounded_send(handshake.encode().to_vec())
            .expect("send failed");

        // still cleaner than cpp ._.
        let send_cipher: Rc<RefCell<Option<ChaCha20Poly1305>>> = Rc::new(RefCell::new(None));
        let send_nonce_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));
        let recv_cipher: Rc<RefCell<Option<ChaCha20Poly1305>>> = Rc::new(RefCell::new(None));
        let recv_nonce_count: Rc<RefCell<u32>> = Rc::new(RefCell::new(0));

        spawn_local(async move {
            while let Some(packet) = rx.next().await {
                if let Err(err) = send.send(Message::Bytes(packet)).await {
                    web_sys::console::error_1(&format!("Failed sending packet: {err:?}").into());

                    break;
                }
            }
        });

        let tx_clone = tx.clone();
        let send_cipher_clone = Rc::clone(&send_cipher);
        let send_nonce_count_clone = Rc::clone(&send_nonce_count);

        let button = window
            .document()
            .unwrap()
            .get_element_by_id("playButton")
            .expect("playButton not found");

        let closure = wasm_bindgen::closure::Closure::<dyn FnMut()>::new(move || {
            let cipher_ref = send_cipher_clone.borrow();

            let cipher = match cipher_ref.as_ref() {
                Some(cipher) => cipher,
                None => {
                    web_sys::console::error_1(&"Cannot spawn: handshake not completed".into());

                    return;
                }
            };

            let name = web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.get_element_by_id("nameInput"))
                .and_then(|e| e.dyn_into::<HtmlInputElement>().ok())
                .map(|input| input.value())
                .unwrap_or_else(|| "Unnamed".into());

            let spawn = SpawnReqPacket::new(name, js_sys::Date::now() as u64);

            let plaintext = spawn.encode();

            let nonce_counter = {
                let mut counter = send_nonce_count_clone.borrow_mut();

                let value = *counter;
                *counter += 1;
                value
            };

            let encrypted = match cipher.encrypt(&nonce(nonce_counter), plaintext.as_ref()) {
                Ok(data) => data,

                Err(err) => {
                    web_sys::console::error_1(
                        &format!("Failed encrypting SpawnReqPacket: {err:?}").into(),
                    );

                    return;
                }
            };

            if let Err(err) = tx_clone.unbounded_send(encrypted) {
                web_sys::console::error_1(
                    &format!("Failed queueing SpawnReqPacket: {err:?}").into(),
                );

                return;
            }

            web_sys::console::log_1(
                &format!(
                    "sent SpawnReqPacket logical_id=1 wire_id={}",
                    SpawnReqPacket::ID
                )
                .into(),
            );
        });

        button
            .add_event_listener_with_callback("click", closure.as_ref().unchecked_ref())
            .unwrap();

        closure.forget();

        let send_cipher_for_reader = Rc::clone(&send_cipher);
        let recv_cipher_for_reader = Rc::clone(&recv_cipher);
        let recv_nonce_count_for_reader = Rc::clone(&recv_nonce_count);
        let secret_key_for_reader = secret_key.clone();

        spawn_local(async move {
            while let Some(result) = recv.next().await {
                match result {
                    Ok(Message::Bytes(bytes)) => {
                        if bytes.is_empty() {
                            continue;
                        }

                        if recv_cipher_for_reader.borrow().is_none() {
                            match HandshakePacket::decode(&bytes) {
                                Ok(handshake) => {
                                    let server_public_key = PublicKey::from(
                                        <[u8; 32]>::try_from(&handshake.handshake[..32])
                                            .expect("invalid server public key"),
                                    );

                                    let (new_send_cipher, new_recv_cipher) =
                                        derive_ciphers(&secret_key_for_reader, &server_public_key);

                                    *send_cipher_for_reader.borrow_mut() = Some(new_send_cipher);

                                    *recv_cipher_for_reader.borrow_mut() = Some(new_recv_cipher);

                                    *recv_nonce_count_for_reader.borrow_mut() = 0;

                                    web_sys::console::log_1(
                                        &"Handshake accepted! Encryption established.".into(),
                                    );
                                }

                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("Handshake decode failed: {e:?}").into(),
                                    );
                                }
                            }

                            continue;
                        }

                        let plaintext = {
                            let cipher_ref = recv_cipher_for_reader.borrow();

                            let cipher = match cipher_ref.as_ref() {
                                Some(cipher) => cipher,

                                None => {
                                    web_sys::console::error_1(
                                        &"Received encrypted packet before cipher was ready".into(),
                                    );

                                    continue;
                                }
                            };

                            let counter = {
                                let mut counter = recv_nonce_count_for_reader.borrow_mut();

                                let value = *counter;
                                *counter += 1;
                                value
                            };

                            match cipher.decrypt(&nonce(counter), bytes.as_ref()) {
                                Ok(data) => data,

                                Err(err) => {
                                    web_sys::console::error_1(
                                        &format!(
                                            "Packet decryption failed (nonce={}): {err:?}",
                                            counter
                                        )
                                        .into(),
                                    );

                                    continue;
                                }
                            }
                        };

                        if plaintext.is_empty() {
                            continue;
                        }

                        let wire_id = plaintext[0];

                        let id = wire_id ^ PACKET_SEED;

                        web_sys::console::log_1(
                            &format!("received packet wire_id={} logical_id={}", wire_id, id)
                                .into(),
                        );

                        match id {
                            0 => match HandshakePacket::decode(&plaintext) {
                                Ok(_) => {
                                    web_sys::console::log_1(
                                        &"Unexpected encrypted handshake packet".into(),
                                    );
                                }

                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("Handshake decode failed: {e:?}").into(),
                                    );
                                }
                            },

                            2 => match AddEntityPacket::decode(&plaintext) {
                                Ok(data) => {
                                    web_sys::console::log_1(
                                        &format!("got add entity packet {:?}", data).into(),
                                    );
                                }

                                Err(e) => {
                                    web_sys::console::error_1(
                                        &format!("AddEntityPacket decode failed: {e:?}").into(),
                                    );
                                }
                            },

                            id => {
                                web_sys::console::warn_1(
                                    &format!(
                                        "Unknown packet logical_id={} wire_id={}",
                                        id, wire_id
                                    )
                                    .into(),
                                );
                            }
                        }
                    }

                    Ok(Message::Text(_)) => {
                        web_sys::console::warn_1(
                            &"Received unexpected text WebSocket message".into(),
                        );
                    }

                    Err(e) => {
                        web_sys::console::warn_1(&format!("WebSocket closed: {e:?}").into());

                        break;
                    }
                }
            }
        });

        Self { secret_key }
    }
}
