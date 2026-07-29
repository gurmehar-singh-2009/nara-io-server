use client::get_server_verifying_key;
use futures::{SinkExt, StreamExt};
use gloo_net::websocket::Message;
use gloo_net::websocket::futures::WebSocket;
use std::cell::RefCell;
use std::rc::Rc;
use wasm_bindgen::prelude::*;
use web_sys::{DedicatedWorkerGlobalScope, MessageEvent, OffscreenCanvas};

use client::init_environment;
use client::render::renderer::RenderState;

fn main() {
    init_environment();

    let global: DedicatedWorkerGlobalScope = web_sys::js_sys::global()
        .dyn_into()
        .expect("worker.rs must run inside a Web Worker environment");

    // Listen for messages from the Main Thread
    let on_message = Closure::<dyn FnMut(_)>::new(move |e: MessageEvent| {
        let data = e.data();
        let data2 = data.clone();

        // 1. Check if payload is the transferred OffscreenCanvas
        if let Ok(canvas) = data.dyn_into::<OffscreenCanvas>() {
            log::info!("OffscreenCanvas received in Web Worker. Launching Game Engine...");

            // Spawn async task for Renderer and gloo_net WebSockets
            wasm_bindgen_futures::spawn_local(async move {
                run_worker_engine(canvas).await;
            });
        }
        // 2. Handle incoming input events forwarded from Main Thread
        else if let Some(msg_str) = data2.as_string() {
            if msg_str.starts_with("KEY_DOWN:") {
                let key_code = msg_str.trim_start_matches("KEY_DOWN:");
                log::debug!("Worker received keypress: {}", key_code);
                // Update local input state here
            }
        }
    });

    global.set_onmessage(Some(on_message.as_ref().unchecked_ref()));
    on_message.forget();
}

async fn run_worker_engine(canvas: OffscreenCanvas) {
    let verifying_key = get_server_verifying_key();
    log::info!(
        "Server Verifying Key loaded inside Worker: {:?}",
        verifying_key
    );

    // Initialize wgpu RenderState targeting the OffscreenCanvas
    let render_state = Rc::new(RefCell::new(RenderState::new_offscreen(canvas).await));

    // Start Worker Render Loop (runs independently via requestAnimationFrame)
    start_render_loop(render_state.clone());

    // Connect WebSocket securely inside WorkerGlobalScope
    match WebSocket::open("wss://yourgame.com/ws") {
        Ok(mut ws) => {
            log::info!("WebSocket connected natively inside Web Worker.");

            while let Some(msg) = ws.next().await {
                match msg {
                    Ok(Message::Bytes(_bytes)) => {
                        // Decrypt ChaCha20 payload & update physics state
                    }
                    Ok(Message::Text(text)) => {
                        log::info!("Received text message: {}", text);
                    }
                    Err(err) => {
                        log::error!("WebSocket error: {:?}", err);
                    }
                }
            }
        }
        Err(err) => {
            log::error!("Failed to open WebSocket: {:?}", err);
        }
    }
}

fn start_render_loop(render_state: Rc<RefCell<RenderState>>) {
    let global: DedicatedWorkerGlobalScope = web_sys::js_sys::global()
        .dyn_into()
        .expect("Failed to cast to DedicatedWorkerGlobalScope");

    let loop_closure: Rc<RefCell<Option<Closure<dyn FnMut()>>>> = Rc::new(RefCell::new(None));
    let loop_closure_clone = loop_closure.clone();

    *loop_closure.borrow_mut() = Some(Closure::<dyn FnMut()>::new(move || {
        // Draw frame
        render_state.borrow_mut().render();

        // Schedule next frame in Web Worker
        let global: DedicatedWorkerGlobalScope = web_sys::js_sys::global().dyn_into().unwrap();
        if let Some(ref closure) = *loop_closure_clone.borrow() {
            let _ = global.request_animation_frame(closure.as_ref().unchecked_ref());
        }
    }));

    // Kick off first frame
    let _ = global.request_animation_frame(
        loop_closure
            .borrow()
            .as_ref()
            .unwrap()
            .as_ref()
            .unchecked_ref(),
    );
}
