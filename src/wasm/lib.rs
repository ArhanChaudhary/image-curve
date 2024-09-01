use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod gilbert;
mod renderer;
mod worker;

#[wasm_bindgen(start)]
fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Deserialize)]
#[serde(tag = "action", content = "payload")]
enum ReceivedWorkerMessage {
    #[serde(rename = "loadImage")]
    LoadImage,
    #[serde(rename = "canvasInit")]
    CanvasInit(CanvasInitMessage),
    #[serde(rename = "step")]
    Step,
    #[serde(rename = "start")]
    Start,
    #[serde(rename = "stop")]
    Stop,
}

#[derive(Deserialize)]
struct CanvasInitMessage {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    canvas: HtmlCanvasElement,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            Self::LoadImage => {
                renderer::load_image();
            }
            Self::CanvasInit(canvas_init_message) => {
                renderer::canvas_init(canvas_init_message);
            }
            Self::Stop => {
                renderer::stop();
            }

            Self::Step => {
                worker::step();
            }
            Self::Start => {
                worker::start();
            }
        }
    }
}
