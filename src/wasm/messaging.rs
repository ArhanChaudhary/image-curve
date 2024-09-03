use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

use crate::worker;

#[wasm_bindgen(js_name = handleMessage)]
pub fn handle_message(message: JsValue) {
    let received_worker_message: ReceivedWorkerMessage =
        serde_wasm_bindgen::from_value(message).unwrap();
    received_worker_message.process();
}

#[derive(Serialize, Deserialize)]
#[serde(tag = "action", content = "payload")]
pub enum ReceivedWorkerMessage {
    Step,
    Start,
}

impl ReceivedWorkerMessage {
    pub fn process(self) {
        match self {
            Self::Step => {
                worker::step();
            }
            Self::Start => {
                worker::start();
            }
        }
    }
}
