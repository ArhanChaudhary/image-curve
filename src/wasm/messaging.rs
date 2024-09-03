use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;

use crate::{renderer, worker};

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum ReceivedWorkerMessage {
    // #[serde(rename = "loadImage")]
    // LoadImage,
    #[serde(rename = "step")]
    Step,
    #[serde(rename = "start")]
    Start,
    // #[serde(rename = "stop")]
    // Stop,
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

#[derive(Serialize, Deserialize)]
pub struct ChangeSpeedMessage {
    pub new_speed_percentage: usize,
}

#[derive(Serialize, Deserialize)]
pub struct ChangeStepMessage {
    pub new_step_percentage: usize,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            // Self::LoadImage => {
            //     renderer::load_image();
            // }
            // Self::Stop => {
            //     renderer::stop();
            // }
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
