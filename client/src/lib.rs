use ed25519_dalek::VerifyingKey;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::Message;
use gloo_net::websocket::futures::WebSocket;
use std::time::Duration;
use wasm_bindgen::prelude::*;
use winit::event_loop::EventLoop;
use winit::platform::web::EventLoopExtWebSys;

mod render;

#[cfg(target_arch = "wasm32")]
use tokio_with_wasm::alias as tokio;

use crate::render::renderer::Renderer;

const PUBLIC_KEY_BYTES: &[u8; 32] = include_bytes!("../public_key.bin");

pub fn get_server_verifying_key() -> VerifyingKey {
    VerifyingKey::from_bytes(PUBLIC_KEY_BYTES).expect("Invalid verifying key length")
}

mod structs;
mod entities;

#[wasm_bindgen(start)]
pub fn start() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Info).unwrap_throw();

    let event_loop = EventLoop::with_user_event().build().unwrap();
    let render = Renderer::new(&event_loop);
    event_loop.spawn_app(render);
}
