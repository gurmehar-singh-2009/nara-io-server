use client::init_environment;
use wasm_bindgen::prelude::*;
use web_sys::{HtmlCanvasElement, KeyboardEvent, Worker, window};

fn main() {
    init_environment();

    let document = window().unwrap().document().unwrap();
    let canvas: HtmlCanvasElement = document
        .get_element_by_id("gameCanvas")
        .unwrap()
        .dyn_into()
        .unwrap();

    // 1. Transfer Canvas control off the DOM thread
    let offscreen_canvas = canvas
        .transfer_control_to_offscreen()
        .expect("Failed to transfer canvas to offscreen");

    // 2. Spawn the isolated Web Worker binary
    let worker = Worker::new("./worker.js").expect("Failed to spawn worker.js");

    // 3. Send OffscreenCanvas handle to the worker via postMessage transfer
    let transfer_array = js_sys::Array::new();
    transfer_array.push(&offscreen_canvas);

    worker
        .post_message_with_transfer(&offscreen_canvas, &transfer_array)
        .expect("Failed to send canvas to worker");

    // 4. Forward Keyboard input events to the Web Worker
    let worker_clone = worker.clone();
    let on_keydown = Closure::<dyn FnMut(_)>::new(move |e: KeyboardEvent| {
        // Send key code to worker
        let payload = JsValue::from_str(&format!("KEY_DOWN:{}", e.code()));
        let _ = worker_clone.post_message(&payload);
    });

    window()
        .unwrap()
        .add_event_listener_with_callback("keydown", on_keydown.as_ref().unchecked_ref())
        .unwrap();
    on_keydown.forget();

    log::info!("App shell initialized. Canvas transferred to Web Worker.");
}
