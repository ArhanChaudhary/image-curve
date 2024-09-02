use serde::Deserialize;
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

mod gilbert;
mod renderer;
mod worker;
mod utils;

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
    #[serde(rename = "changeSpeed")]
    ChangeSpeed(ChangeSpeedMessage),
    #[serde(rename = "changeStep")]
    ChangeStep(ChangeStepMessage),
}

#[derive(Deserialize)]
struct CanvasInitMessage {
    #[serde(with = "serde_wasm_bindgen::preserve")]
    canvas: HtmlCanvasElement,
}

#[derive(Deserialize)]
struct ChangeSpeedMessage {
    new_speed_percentage: usize,
}

#[derive(Deserialize)]
struct ChangeStepMessage {
    new_step_percentage: usize,
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
            Self::ChangeSpeed(change_speed_message) => {
                renderer::change_speed(change_speed_message);
            }
            Self::ChangeStep(change_step_message) => {
                renderer::change_step(change_step_message);
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
