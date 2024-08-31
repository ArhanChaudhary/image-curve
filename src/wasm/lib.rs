use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::console;

mod gilbert;
mod renderer;

#[wasm_bindgen(start)]
pub fn start() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    console::log_1(&"entry".into());
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "payload")]
enum ReceivedWorkerMessage {
    #[serde(rename = "canvasInit")]
    CanvasInit(CanvasInitMessage),
    #[serde(rename = "step")]
    Step,
}
#[derive(Deserialize)]
struct CanvasInitMessage {
    #[serde(rename = "pixelData")]
    pixel_data: Vec<u8>,
    width: usize,
    height: usize,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            Self::CanvasInit(canvas_init_message) => {
                renderer::canvas_init(canvas_init_message);
            }
            Self::Step => {
                renderer::step();
            }
        }
    }
}
