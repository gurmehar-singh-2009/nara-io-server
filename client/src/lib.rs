use std::time::Duration;

use ed25519_dalek::VerifyingKey;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::{Message, futures::WebSocket};
use shared::packets::PACKET_SEED;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::spawn_local;
use web_sys::window;
use winit::{event_loop::EventLoop, platform::web::EventLoopExtWebSys};

mod render;
mod socket;

#[cfg(target_arch = "wasm32")]
use tokio_with_wasm::alias as tokio;

use crate::{render::renderer::Renderer, socket::Socket};

const PUBLIC_KEY_BYTES: &[u8; 32] = include_bytes!("../public_key.bin");

pub fn get_server_verifying_key() -> VerifyingKey {
    VerifyingKey::from_bytes(PUBLIC_KEY_BYTES).expect("Invalid verifying key length")
}

mod entities;
mod structs;

#[wasm_bindgen(start)]
pub async fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    // let socket = Socket::new("ws://127.0.0.1:8080".to_string()).await;
    let window = window().unwrap();

    let closure = Closure::<dyn FnMut()>::new(|| {
        spawn_local(async {
            Socket::new("ws://127.0.0.1:8080".into()).await;
        });
    });

    web_sys::console::log_1(&format!("PACKET SEED: {}", PACKET_SEED).into());

    window
        .add_event_listener_with_callback("load", closure.as_ref().unchecked_ref())
        .unwrap();

    closure.forget();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let render = Renderer::new(&event_loop);
    event_loop.spawn_app(render);
}
